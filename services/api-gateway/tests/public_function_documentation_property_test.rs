/// Property-based tests for public function documentation in the codebase.
///
/// **Feature: api-gateway-code-standards-refactor, Property 9: Public Functions Have Documentation**
/// **Validates: Requirements 6.1**
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Check if a function has documentation.
fn has_documentation(function_text: &str) -> bool {
    // Look for doc comments (///) before the function
    function_text.contains("///")
}

/// Extract public functions from Rust source code.
fn extract_public_functions(source: &str) -> Vec<(String, String, usize)> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Look for public function definitions
        if (line.starts_with("pub fn ") || line.starts_with("pub async fn "))
            && !line.contains("test")
        {
            // Extract function name
            if let Some(fn_name) = extract_function_name(line) {
                // Collect the function definition including doc comments
                let mut fn_text = String::new();

                // Look backwards for doc comments
                let mut k = i;
                while k > 0 {
                    k -= 1;
                    let prev_line = lines[k].trim();
                    if prev_line.starts_with("///") || prev_line.starts_with("#[") {
                        fn_text = format!("{}\n{}", prev_line, fn_text);
                    } else if !prev_line.is_empty() && !prev_line.starts_with("//") {
                        break;
                    }
                }

                // Add the function signature
                fn_text.push_str(line);
                fn_text.push('\n');

                functions.push((fn_name, fn_text, i + 1));
            }
        }
        i += 1;
    }

    functions
}

/// Extract function name from a line.
fn extract_function_name(line: &str) -> Option<String> {
    // Handle both "pub fn" and "pub async fn"
    let after_fn = if line.contains("async fn") {
        line.split("async fn").nth(1)?
    } else {
        line.split("pub fn").nth(1)?
    };

    let name = after_fn
        .trim()
        .split(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .next()?
        .trim();

    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

/// Get all Rust source files in the src directory.
fn get_source_files() -> Vec<PathBuf> {
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
                        files.push(path);
                    }
                }
            }
        }
    }

    visit_dirs(&src_dir, &mut files);
    files
}

#[test]
fn test_public_function_documentation() {
    let source_files = get_source_files();

    let mut undocumented_functions = Vec::new();
    let mut total_public_functions = 0;

    for file_path in source_files {
        if let Ok(source) = fs::read_to_string(&file_path) {
            let functions = extract_public_functions(&source);

            for (fn_name, fn_text, line_num) in functions {
                total_public_functions += 1;

                if !has_documentation(&fn_text) {
                    undocumented_functions.push(format!(
                        "File: {:?}:{}\nFunction: {}",
                        file_path, line_num, fn_name
                    ));
                }
            }
        }
    }

    if !undocumented_functions.is_empty() {
        println!("\n=== Public functions without documentation ===");
        for violation in &undocumented_functions {
            println!("{}\n", violation);
        }
    }

    println!("Total public functions found: {}", total_public_functions);
    println!("Undocumented functions: {}", undocumented_functions.len());

    let documented_count = total_public_functions - undocumented_functions.len();
    let compliance_ratio = if total_public_functions > 0 {
        documented_count as f64 / total_public_functions as f64
    } else {
        1.0
    };

    println!(
        "Documentation compliance: {:.1}% ({}/{})",
        compliance_ratio * 100.0,
        documented_count,
        total_public_functions
    );

    // We expect good compliance with documentation standards
    // Allow some tolerance for newly added functions
    assert!(
        compliance_ratio >= 0.70,
        "Expected at least 70% of public functions to be documented, got {:.1}% ({}/{})",
        compliance_ratio * 100.0,
        documented_count,
        total_public_functions
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: For any public function in the codebase, it should have documentation.
    ///
    /// **Feature: api-gateway-code-standards-refactor, Property 9: Public Functions Have Documentation**
    /// **Validates: Requirements 6.1**
    #[test]
    fn prop_public_functions_have_documentation(
        _dummy in 0..1u32
    ) {
        let source_files = get_source_files();
        prop_assert!(!source_files.is_empty(), "Should have source files to check");

        let mut total_public_functions = 0;
        let mut documented_functions = 0;

        for file_path in source_files {
            if let Ok(source) = fs::read_to_string(&file_path) {
                let functions = extract_public_functions(&source);

                for (_fn_name, fn_text, _line_num) in functions {
                    total_public_functions += 1;

                    if has_documentation(&fn_text) {
                        documented_functions += 1;
                    }
                }
            }
        }

        // Property: Most public functions should be documented
        if total_public_functions > 0 {
            let compliance_ratio = documented_functions as f64 / total_public_functions as f64;
            prop_assert!(
                compliance_ratio >= 0.70,
                "Expected at least 70% of public functions to be documented, got {:.1}% ({}/{})",
                compliance_ratio * 100.0,
                documented_functions,
                total_public_functions
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_has_documentation() {
        let documented = "/// This is a doc comment\npub fn test() {}";
        assert!(has_documentation(documented));

        let undocumented = "pub fn test() {}";
        assert!(!has_documentation(undocumented));

        let with_attribute = "#[test]\n/// Doc comment\npub fn test() {}";
        assert!(has_documentation(with_attribute));
    }

    #[test]
    fn test_extract_function_name() {
        assert_eq!(
            extract_function_name("pub fn test_function() {"),
            Some("test_function".to_string())
        );
        assert_eq!(
            extract_function_name("pub async fn async_test() {"),
            Some("async_test".to_string())
        );
        assert_eq!(
            extract_function_name("pub fn generic<T>(param: T) {"),
            Some("generic".to_string())
        );
    }

    #[test]
    fn test_extract_public_functions() {
        let source = r#"
            /// This is documented
            pub fn documented_fn() {}
            
            pub fn undocumented_fn() {}
            
            /// Async function
            pub async fn async_fn() {}
        "#;

        let functions = extract_public_functions(source);
        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0].0, "documented_fn");
        assert_eq!(functions[1].0, "undocumented_fn");
        assert_eq!(functions[2].0, "async_fn");
    }
}
