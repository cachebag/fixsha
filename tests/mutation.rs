use fixsha::{NixBuildResult, parse_and_replace_hash};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_and_replace_hash_basic() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = r#"{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "test-package";
  version = "0.1.0";
  
  cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  
  meta = {
    description = "A test package";
  };
}
"#;

    fs::write(&test_file, original_content).unwrap();

    let new_hash = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=";
    parse_and_replace_hash(temp_dir.path(), "package.nix", "cargoHash", new_hash).unwrap();

    let updated_content = fs::read_to_string(&test_file).unwrap();
    assert!(updated_content.contains(&format!(r#"cargoHash = "{}";"#, new_hash)));
    assert!(!updated_content.contains("sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
}

#[test]
fn test_parse_and_replace_hash_preserves_indentation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = r#"{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage {
  pname = "test-package";
  
    cargoHash = "sha256-OLD_HASH";
  
  meta = {
    description = "A test package";
  };
}
"#;

    fs::write(&test_file, original_content).unwrap();

    let new_hash = "sha256-NEW_HASH";
    parse_and_replace_hash(temp_dir.path(), "package.nix", "cargoHash", new_hash).unwrap();

    let updated_content = fs::read_to_string(&test_file).unwrap();

    // Check that the indentation (4 spaces) is preserved
    assert!(updated_content.contains(&format!(r#"    cargoHash = "{}";"#, new_hash)));
}

#[test]
fn test_parse_and_replace_hash_with_tabs() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = "{\n\tcargoHash = \"sha256-OLD\";\n}\n";
    fs::write(&test_file, original_content).unwrap();

    let new_hash = "sha256-NEW";
    parse_and_replace_hash(temp_dir.path(), "package.nix", "cargoHash", new_hash).unwrap();

    let updated_content = fs::read_to_string(&test_file).unwrap();
    assert!(updated_content.contains(&format!("\tcargoHash = \"{}\";", new_hash)));
}

#[test]
fn test_parse_and_replace_hash_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = r#"{
  pname = "test-package";
  version = "0.1.0";
}
"#;

    fs::write(&test_file, original_content).unwrap();

    let result = parse_and_replace_hash(temp_dir.path(), "package.nix", "cargoHash", "sha256-NEW");

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("hash `cargoHash` not found"));
}

#[test]
fn test_parse_and_replace_hash_file_not_found() {
    let temp_dir = TempDir::new().unwrap();

    let result = parse_and_replace_hash(
        temp_dir.path(),
        "nonexistent.nix",
        "cargoHash",
        "sha256-NEW",
    );

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("failed to read"));
}

#[test]
fn test_parse_and_replace_hash_different_key() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = r#"{
  vendorHash = "sha256-OLD_VENDOR_HASH";
  cargoHash = "sha256-OLD_CARGO_HASH";
}
"#;

    fs::write(&test_file, original_content).unwrap();

    let new_hash = "sha256-NEW_VENDOR_HASH";
    parse_and_replace_hash(temp_dir.path(), "package.nix", "vendorHash", new_hash).unwrap();

    let updated_content = fs::read_to_string(&test_file).unwrap();

    // vendorHash should be updated
    assert!(updated_content.contains(&format!(r#"vendorHash = "{}";"#, new_hash)));
    // cargoHash should remain unchanged
    assert!(updated_content.contains(r#"cargoHash = "sha256-OLD_CARGO_HASH";"#));
}

#[test]
fn test_parse_and_replace_hash_with_spaces_around_equals() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("package.nix");

    let original_content = r#"{
  cargoHash   =   "sha256-OLD";
}
"#;

    fs::write(&test_file, original_content).unwrap();

    let new_hash = "sha256-NEW";
    parse_and_replace_hash(temp_dir.path(), "package.nix", "cargoHash", new_hash).unwrap();

    let updated_content = fs::read_to_string(&test_file).unwrap();
    assert!(updated_content.contains(&format!(r#"cargoHash = "{}";"#, new_hash)));
}

#[test]
fn test_parse_and_replace_hash_multiple_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create two different files
    let file1 = temp_dir.path().join("package1.nix");
    let file2 = temp_dir.path().join("package2.nix");

    fs::write(&file1, "{\n  cargoHash = \"sha256-OLD1\";\n}\n").unwrap();
    fs::write(&file2, "{\n  cargoHash = \"sha256-OLD2\";\n}\n").unwrap();

    // Update first file
    parse_and_replace_hash(temp_dir.path(), "package1.nix", "cargoHash", "sha256-NEW1").unwrap();

    // Update second file
    parse_and_replace_hash(temp_dir.path(), "package2.nix", "cargoHash", "sha256-NEW2").unwrap();

    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(content1.contains(r#"  cargoHash = "sha256-NEW1";"#));
    assert!(content2.contains(r#"  cargoHash = "sha256-NEW2";"#));
}

#[test]
fn test_nix_build_result_structure() {
    // Test that NixBuildResult can be constructed and accessed
    // This is a basic structure test to ensure the public API is accessible
    let result = NixBuildResult {
        status: std::process::Command::new("true").status().unwrap(),
        stdout_lines: vec!["line1".to_string(), "line2".to_string()],
        stderr_lines: vec!["error1".to_string()],
        new_hash: Some("sha256-HASH".to_string()),
    };

    assert!(result.status.success());
    assert_eq!(result.stdout_lines.len(), 2);
    assert_eq!(result.stderr_lines.len(), 1);
    assert_eq!(result.new_hash, Some("sha256-HASH".to_string()));
}
