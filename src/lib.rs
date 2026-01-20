// Utility functions to parse and replace cargoHash in package.nix
// We also build the Nix derivation and capture output here to extract the new hash.

use anyhow::{Context, Result};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

/// Parse package.nix from the root of the repo and extract the cargoHash value.
pub fn parse_and_replace_hash(
    repo_root: &Path,
    filename: &str,
    hash_key: &str,
    new_hash: &str,
) -> Result<()> {
    let path = repo_root.join(filename);
    let contents =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let mut found = false;

    let updated = contents
        .lines()
        .map(|line| {
            let trimmed = line.trim();

            if let Some((k, _)) = trimmed.split_once('=')
                && k.trim() == hash_key
            {
                found = true;
                // Preserve indentation
                // This could be more elegant but it's fine for me
                let indent = line
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .collect::<String>();

                format!(r#"{indent}{hash_key} = "{new_hash}";"#)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !found {
        anyhow::bail!("hash `{hash_key}` not found in {}", path.display());
    }

    fs::write(&path, updated).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Build Nix derivation and capture output.
pub fn build_nix_derivation() -> std::io::Result<NixBuildResult> {
    let mut child = Command::new("nix")
        .args(["build", "-f", "default.nix"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("stdout unavailable");
    let stderr = child.stderr.take().expect("stderr unavailable");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    /// Here, we use channels to collect output from both stdout and stderr concurrently.
    /// This allows us to process lines as they come in, rather than waiting for one stream to
    /// finish.
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();

    enum StreamEvent {
        Stdout(String),
        Stderr(String),
    }

    let tx_err = tx.clone();
    let err_handle = std::thread::spawn(move || -> std::io::Result<()> {
        for line in stderr_reader.lines() {
            let line = line?;
            tx_err.send(StreamEvent::Stderr(line)).unwrap();
        }
        Ok(())
    });

    let out_handle = std::thread::spawn(move || -> std::io::Result<()> {
        for line in stdout_reader.lines() {
            let line = line?;
            tx.send(StreamEvent::Stdout(line)).unwrap();
        }
        Ok(())
    });

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();
    let mut correct_hash = None;

    // Collect lines from both stdout and stderr
    // until both threads have finished sending.
    for event in rx {
        match event {
            StreamEvent::Stdout(line) => {
                stdout_lines.push(line);
            }
            StreamEvent::Stderr(line) => {
                if let Some(got) = classify(&line) {
                    correct_hash = Some(got.to_owned());
                }
                stderr_lines.push(line);
            }
        }
    }

    // TODO: not this
    err_handle.join().unwrap()?;
    out_handle.join().unwrap()?;
    let status = child.wait()?;

    Ok(NixBuildResult {
        status,
        stdout_lines,
        stderr_lines,
        new_hash: correct_hash,
    })
}

/// Result of Nix build.
#[derive(Debug)]
pub struct NixBuildResult {
    pub status: std::process::ExitStatus,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub new_hash: Option<String>,
}

/// Helper to extract 'got: ' error.
fn classify(line: &str) -> Option<&str> {
    let line = line.trim_start();
    line.strip_prefix("got:").map(str::trim)
}
