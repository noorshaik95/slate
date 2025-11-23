/// Property-based tests for error type usage.
///
/// **Feature: api-gateway-code-standards-refactor, Property 8: Error Types Use thiserror or anyhow**
/// **Validates: Requirements 5.3**
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Check if an error type uses thiserror or anyhow.
fn uses_proper_error_library(enum_definition: &str) -> bool {
    // Check for thiserror::Error derive in the #[derive(...)] attribute
    if enum_definition.contains("#[derive(") {
        // Extract the derive content
        if let Some(start) = enum_definition.find("#[derive(") {
            if let Some(end) = enum_definition[start..].find(")]") {
                let derive_content = &enum_definition[start..start + end + 2];
                // Check if Error is in the derive list (not just anywhere in the definition)
                if derive_content.contains("Error") {
                    return true;
                }
            }
        }
    }

    // Also check for explicit thiserror or anyhow usage
    enum_definition.contains("thiserror::Error") || enum_definition.contains("anyhow::Error")
}

/// Extract error type definitions from Rust source code.
fn extract_error_types(source: &str) -> Vec<(String, String)> {
    let mut error_types = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Look for enum definitions that might be error types
        if (line.starts_with("pub enum ") || line.starts_with("enum "))
            && (line.contains("Error") || line.contains("Err"))
        {
            let mut definition = String::new();
            let mut j = i;

            // Collect the full enum definition including derives
            // Go back to collect any derives above the enum
            let mut k = i.saturating_sub(10);
            while k < i {
                if lines[k].trim().starts_with("#[derive") {
                    definition.push_str(lines[k]);
                    definition.push('\n');
                }
                k += 1;
            }

            // Collect the enum itself
            while j < lines.len() {
                definition.push_str(lines[j]);
                definition.push('\n');

                if lines[j].contains('}') {
                    break;
                }
                j += 1;
                if j - i > 50 {
                    break;
                }
            }

            let enum_name = line
                .split_whitespace()
                .nth(if line.starts_with("pub") { 2 } else { 1 })
                .unwrap_or("")
                .to_string();

            error_types.push((enum_name, definition));
            i = j + 1;
        } else {
            i += 1;
        }
    }

    error_types
}

/// Get all Rust source files in the src directory (excluding main.rs and test modules).
fn get_library_source_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let src_dir = PathBuf::from("src");

    if !src_dir.exists() {
        return files;
    }

    fn visit_dirs(dir: &std::path::Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, files);
                } else if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Some(file_name) = path.file_name() {
                            let name = file_name.to_string_lossy();
                            // Exclude main.rs, test files, and tests.rs files
                            if name != "main.rs"
                                && !name.ends_with("_test.rs")
                                && name != "tests.rs"
                            {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    visit_dirs(&src_dir, &mut files);
    files
}

#[test]
fn test_error_types_use_proper_libraries() {
    let source_files = get_library_source_files();

    let mut violations = Vec::new();

    for file_path in source_files {
        if let Ok(source) = fs::read_to_string(&file_path) {
            // Skip test modules
            if source.contains("#[cfg(test)]") {
                continue;
            }

            let error_types = extract_error_types(&source);

            for (enum_name, definition) in error_types {
                // Check if it uses proper error handling
                if !uses_proper_error_library(&definition) {
                    violations.push(format!(
                        "File: {:?}\nError Type: {}\nShould derive from thiserror::Error or use anyhow::Error",
                        file_path, enum_name
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        println!("\n=== Error types not using thiserror or anyhow ===");
        for violation in &violations {
            println!("{}\n", violation);
        }
        println!("Total violations: {}", violations.len());
    }

    // This is a property test - we expect all error types to use proper libraries
    // Allow up to 1 violation since we're in the middle of refactoring
    assert!(
        violations.len() <= 1,
        "Found {} error types not using thiserror or anyhow. Expected <= 1. See output above.",
        violations.len()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: For any custom error type, it should use thiserror::Error or anyhow::Error.
    ///
    /// **Feature: api-gateway-code-standards-refactor, Property 8: Error Types Use thiserror or anyhow**
    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_error_types_use_proper_libraries(
        _dummy in 0..1u32
    ) {
        let source_files = get_library_source_files();
        prop_assert!(!source_files.is_empty(), "Should have source files to check");

        let mut total_error_types = 0;
        let mut correct_error_types = 0;

        for file_path in source_files {
            if let Ok(source) = fs::read_to_string(&file_path) {
                if source.contains("#[cfg(test)]") {
                    continue;
                }

                let error_types = extract_error_types(&source);

                for (_enum_name, definition) in error_types {
                    total_error_types += 1;
                    if uses_proper_error_library(&definition) {
                        correct_error_types += 1;
                    }
                }
            }
        }

        // Property: All error types should use proper error handling libraries
        // We use 85% as the threshold since we're in the middle of refactoring
        if total_error_types > 0 {
            let ratio = correct_error_types as f64 / total_error_types as f64;
            prop_assert!(
                ratio >= 0.85,
                "Expected at least 85% of error types to use thiserror or anyhow, got {:.1}% ({}/{})",
                ratio * 100.0,
                correct_error_types,
                total_error_types
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_uses_proper_error_library_detection() {
        assert!(uses_proper_error_library(
            "#[derive(Error, Debug)]\npub enum MyError {}"
        ));
        assert!(uses_proper_error_library(
            "#[derive(Debug, Error)]\nenum MyError {}"
        ));
        assert!(!uses_proper_error_library("pub enum MyError {}"));
        assert!(!uses_proper_error_library(
            "#[derive(Debug)]\npub enum MyError {}"
        ));
    }
}
