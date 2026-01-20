// A hook to build a Nix derivation and update the cargoHash in package.nix.
//
// It works for nmrs, and probably for other projects I can't gurantee that.

use anyhow::Error;
use fns::{build_nix_derivation, parse_and_replace_hash};
use std::path::Path;

fn main() -> Result<(), Error> {
    // Build the Nix derivation
    let nix_build = build_nix_derivation()?;

    // If a new hash is found, update package.nix
    // Otherwise, if the build succeeded, do nothing.
    // If the build failed and no hash was found, panic.
    if let Some(got) = &nix_build.new_hash {
        println!("new hash found: {got}. Replacing now...");
        parse_and_replace_hash(Path::new("."), "package.nix", "cargoHash", got)?;
        println!("Done. package.nix has been updated, cargoHash is now {got}");
    } else if nix_build.status.success() {
        println!("build succeeded, no need to update hash.");
    } else {
        panic!("build failed and no hash was found");
    }

    Ok(())
}
