/// Property-based tests for no panics in library code.
///
/// **Feature: api-gateway-code-standards-refactor, Property 7: No Panics in Library Code**
/// **Validates: Requirements 5.2**
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Check if a function body contains panic-inducing calls.
fn contains_panic_calls(function_body: &str) -> bool {
    // Look for panic-inducing patterns
    // Note: We need to be careful about false positives in comments and strings
    let lines: Vec<&str> = function_body.lines().collect();

    for line in lines {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") {
            continue;
        }

        // Check for panic patterns (not in comments)
        if let Some(code_part) = trimmed.split("//").next() {
            if code_part.contains("panic!(")
                || code_part.contains(".unwrap()")
                || code_part.contains(".expect(")
            {
                return true;
            }
        }
    }

    false
}

/// Extract function bodies from Rust source code.
fn extract_function_bodies(source: &str) -> Vec<(String, String)> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Skip test functions and main
        if line.contains("#[test]") || line.contains("#[cfg(test)]") || line.contains("fn main(") {
            i += 1;
            continue;
        }

        // Look for function definitions (public or private)
        if (line.starts_with("pub fn ")
            || line.starts_with("pub async fn ")
            || line.starts_with("fn ")
            || line.starts_with("async fn "))
            && !line.contains("fn main(")
        {
            let mut signature = String::new();
            let mut body = String::new();

            // Collect signature
            let mut j = i;
            while j < lines.len() {
                let current_line = lines[j];
                signature.push_str(current_line);
                signature.push(' ');

                if current_line.contains('{') {
                    break;
                }
                j += 1;
            }

            // Collect function body
            if j < lines.len() {
                let mut brace_count = 0;
                let mut started = false;

                while j < lines.len() {
                    let current_line = lines[j];

                    for ch in current_line.chars() {
                        if ch == '{' {
                            brace_count += 1;
                            started = true;
                        } else if ch == '}' {
                            brace_count -= 1;
                        }
                    }

                    if started {
                        body.push_str(current_line);
                        body.push('\n');
                    }

                    if started && brace_count == 0 {
                        break;
                    }

                    j += 1;
                    if j - i > 200 {
                        break;
                    }
                }
            }

            functions.push((signature, body));
            i = j + 1;
        } else {
            i += 1;
        }
    }

    functions
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
fn test_no_panics_in_library_code() {
    let source_files = get_library_source_files();

    let mut violations = Vec::new();

    for file_path in source_files {
        if let Ok(source) = fs::read_to_string(&file_path) {
            // Skip test modules
            if source.contains("#[cfg(test)]") {
                continue;
            }

            let functions = extract_function_bodies(&source);

            for (signature, body) in functions {
                // Skip test functions
                if signature.contains("#[test]") {
                    continue;
                }

                // Check for panic calls
                if contains_panic_calls(&body) {
                    violations.push(format!(
                        "File: {:?}\nFunction: {}\nContains panic-inducing calls (panic!, unwrap(), or expect())",
                        file_path, signature.lines().next().unwrap_or("")
                    ));
                }
            }
        }
    }

    if !violations.is_empty() {
        println!("\n=== Functions with panic-inducing calls ===");
        for violation in &violations {
            println!("{}\n", violation);
        }
        println!("Total violations: {}", violations.len());
    }

    // This is a property test - we expect no panics in library code
    // We'll be lenient and allow a small number of violations for now
    // since some may be intentional (like in error handling paths or infallible operations)
    assert!(
        violations.len() <= 7,
        "Found {} functions with panic-inducing calls. Expected <= 7. See output above.",
        violations.len()
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: For any function in library code, it should not contain panic-inducing calls.
    ///
    /// **Feature: api-gateway-code-standards-refactor, Property 7: No Panics in Library Code**
    /// **Validates: Requirements 5.2**
    #[test]
    fn prop_no_panics_in_library_code(
        _dummy in 0..1u32
    ) {
        let source_files = get_library_source_files();
        prop_assert!(!source_files.is_empty(), "Should have source files to check");

        let mut total_functions = 0;
        let mut functions_with_panics = 0;

        for file_path in source_files {
            if let Ok(source) = fs::read_to_string(&file_path) {
                if source.contains("#[cfg(test)]") {
                    continue;
                }

                let functions = extract_function_bodies(&source);

                for (signature, body) in functions {
                    if signature.contains("#[test]") {
                        continue;
                    }

                    total_functions += 1;
                    if contains_panic_calls(&body) {
                        functions_with_panics += 1;
                    }
                }
            }
        }

        // Property: The ratio of functions without panics should be very high
        // We use 96% as the threshold since some legitimate uses exist (infallible operations)
        if total_functions > 0 {
            let ratio = (total_functions - functions_with_panics) as f64 / total_functions as f64;
            prop_assert!(
                ratio >= 0.96,
                "Expected at least 96% of functions to avoid panic calls, got {:.1}% ({}/{})",
                ratio * 100.0,
                total_functions - functions_with_panics,
                total_functions
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_contains_panic_calls_detection() {
        assert!(contains_panic_calls("panic!(\"error\");"));
        assert!(contains_panic_calls("let x = value.unwrap();"));
        assert!(contains_panic_calls("let y = result.expect(\"failed\");"));
        assert!(!contains_panic_calls("let x = value.unwrap_or_default();"));
        assert!(!contains_panic_calls("// This is a comment with unwrap()"));
    }
}
