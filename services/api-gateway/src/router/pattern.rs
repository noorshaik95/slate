//! Route pattern parsing and representation.
//!
//! Handles parsing of route patterns with support for dynamic segments.

use std::collections::HashMap;

/// Represents a route pattern with support for dynamic segments.
#[derive(Debug, Clone)]
pub struct RoutePattern {
    pub(crate) segments: Vec<PathSegment>,
    pub(crate) method: String,
}

/// A segment in a route path.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    Static(String),
    Dynamic(String), // Parameter name
}

impl RoutePattern {
    /// Parse a path pattern into segments.
    pub fn parse(path: &str, method: &str) -> Self {
        let segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if let Some(param_name) = segment.strip_prefix(':') {
                    PathSegment::Dynamic(param_name.to_string())
                } else {
                    PathSegment::Static(segment.to_string())
                }
            })
            .collect();

        Self {
            segments,
            method: method.to_uppercase(),
        }
    }

    /// Check if this pattern contains only static segments.
    pub fn is_static(&self) -> bool {
        self.segments
            .iter()
            .all(|s| matches!(s, PathSegment::Static(_)))
    }

    /// Try to match a path against this pattern.
    ///
    /// Returns Some(params) if match succeeds, None otherwise.
    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Must have same number of segments
        if path_segments.len() != self.segments.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_seg, path_seg) in self.segments.iter().zip(path_segments.iter()) {
            match pattern_seg {
                PathSegment::Static(expected) => {
                    if expected != path_seg {
                        return None;
                    }
                }
                PathSegment::Dynamic(param_name) => {
                    params.insert(param_name.clone(), path_seg.to_string());
                }
            }
        }

        Some(params)
    }
}
