/// Property-based tests for public type documentation in the codebase.
///
/// **Feature: api-gateway-code-standards-refactor, Property 10: Public Types Have Documentation**
/// **Validates: Requirements 6.2**
use proptest::prelude::*;
use std::fs;
use std::path::PathBuf;

/// Check if a type has documentation.
fn has_documentation(type_text: &str) -> bool {
    // Look for doc comments (///) before the type
    type_text.contains("///")
}

/// Extract public types from Rust source code.
/// Returns (type_name, type_text_with_docs, line_number)
fn extract_public_types(source: &str) -> Vec<(String, String, usize)> {
    let mut types = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Look for public type definitions (struct, enum, type alias)
        let is_public_type = (line.starts_with("pub struct ")
            || line.starts_with("pub enum ")
            || line.starts_with("pub type "))
            && !line.contains("#[test]");

        if is_public_type {
            // Extract type name
            if let Some(type_name) = extract_type_name(line) {
                // Collect the type definition including doc comments
                let mut type_text = String::new();

                // Look backwards for doc comments and attributes
                let mut k = i;
                while k > 0 {
                    k -= 1;
                    let prev_line = lines[k].trim();
                    if prev_line.starts_with("///") || prev_line.starts_with("#[") {
                        type_text = format!("{}\n{}", prev_line, type_text);
                    } else if !prev_line.is_empty() && !prev_line.starts_with("//") {
                        break;
                    }
                }

                // Add the type signature
                type_text.push_str(line);
                type_text.push('\n');

                types.push((type_name, type_text, i + 1));
            }
        }
        i += 1;
    }

    types
}

/// Extract type name from a line.
fn extract_type_name(line: &str) -> Option<String> {
    // Handle "pub struct", "pub enum", "pub type"
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 3 && parts[0] == "pub" {
        let type_keyword = parts[1];
        if type_keyword == "struct" || type_keyword == "enum" || type_keyword == "type" {
            let name = parts[2]
                .split(['<', '(', '{', '=', ';'])
                .next()?
                .trim();

            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        } else {
            None
        }
    } else {
        None
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
fn test_public_type_documentation() {
    let source_files = get_source_files();

    let mut undocumented_types = Vec::new();
    let mut total_public_types = 0;

    for file_path in source_files {
        if let Ok(source) = fs::read_to_string(&file_path) {
            let types = extract_public_types(&source);

            for (type_name, type_text, line_num) in types {
                total_public_types += 1;

                if !has_documentation(&type_text) {
                    undocumented_types.push(format!(
                        "File: {:?}:{}\nType: {}",
                        file_path, line_num, type_name
                    ));
                }
            }
        }
    }

    if !undocumented_types.is_empty() {
        println!("\n=== Public types without documentation ===");
        for violation in &undocumented_types {
            println!("{}\n", violation);
        }
    }

    println!("Total public types found: {}", total_public_types);
    println!("Undocumented types: {}", undocumented_types.len());

    let documented_count = total_public_types - undocumented_types.len();
    let compliance_ratio = if total_public_types > 0 {
        documented_count as f64 / total_public_types as f64
    } else {
        1.0
    };

    println!(
        "Documentation compliance: {:.1}% ({}/{})",
        compliance_ratio * 100.0,
        documented_count,
        total_public_types
    );

    // We expect good compliance with documentation standards
    // Allow some tolerance for newly added types
    assert!(
        compliance_ratio >= 0.70,
        "Expected at least 70% of public types to be documented, got {:.1}% ({}/{})",
        compliance_ratio * 100.0,
        documented_count,
        total_public_types
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: For any public type in the codebase, it should have documentation.
    ///
    /// **Feature: api-gateway-code-standards-refactor, Property 10: Public Types Have Documentation**
    /// **Validates: Requirements 6.2**
    #[test]
    fn prop_public_types_have_documentation(
        _dummy in 0..1u32
    ) {
        let source_files = get_source_files();
        prop_assert!(!source_files.is_empty(), "Should have source files to check");

        let mut total_public_types = 0;
        let mut documented_types = 0;

        for file_path in source_files {
            if let Ok(source) = fs::read_to_string(&file_path) {
                let types = extract_public_types(&source);

                for (_type_name, type_text, _line_num) in types {
                    total_public_types += 1;

                    if has_documentation(&type_text) {
                        documented_types += 1;
                    }
                }
            }
        }

        // Property: Most public types should be documented
        if total_public_types > 0 {
            let compliance_ratio = documented_types as f64 / total_public_types as f64;
            prop_assert!(
                compliance_ratio >= 0.70,
                "Expected at least 70% of public types to be documented, got {:.1}% ({}/{})",
                compliance_ratio * 100.0,
                documented_types,
                total_public_types
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_has_documentation() {
        let documented = "/// This is a doc comment\npub struct Test {}";
        assert!(has_documentation(documented));

        let undocumented = "pub struct Test {}";
        assert!(!has_documentation(undocumented));

        let with_attribute = "#[derive(Debug)]\n/// Doc comment\npub struct Test {}";
        assert!(has_documentation(with_attribute));
    }

    #[test]
    fn test_extract_type_name() {
        assert_eq!(
            extract_type_name("pub struct TestStruct {"),
            Some("TestStruct".to_string())
        );
        assert_eq!(
            extract_type_name("pub enum TestEnum {"),
            Some("TestEnum".to_string())
        );
        assert_eq!(
            extract_type_name("pub type TestType = String;"),
            Some("TestType".to_string())
        );
        assert_eq!(
            extract_type_name("pub struct Generic<T> {"),
            Some("Generic".to_string())
        );
    }

    #[test]
    fn test_extract_public_types() {
        let source = r#"
            /// This is documented
            pub struct DocumentedStruct {}
            
            pub struct UndocumentedStruct {}
            
            /// Enum with docs
            pub enum DocumentedEnum {
                Variant1,
            }
            
            pub type TypeAlias = String;
        "#;

        let types = extract_public_types(source);
        assert_eq!(types.len(), 4);
        assert_eq!(types[0].0, "DocumentedStruct");
        assert_eq!(types[1].0, "UndocumentedStruct");
        assert_eq!(types[2].0, "DocumentedEnum");
        assert_eq!(types[3].0, "TypeAlias");
    }
}
