//! Built-in prompt templates for common tasks.

/// Built-in template for code review.
pub const CODE_REVIEW: &str = r#"# Code Review

Please review the following code for:
- Correctness and logic errors
- Code style and best practices
- Performance issues
- Security vulnerabilities
- Test coverage

{{#if file}}
File: {{file}}
{{/if}}

{{#if language}}
Language: {{language}}
{{/if}}

Code:
```
{{code}}
```

Provide detailed feedback and suggestions for improvement.
"#;

/// Built-in template for debugging.
pub const DEBUG: &str = r#"# Debug Assistance

Help me debug this issue:

{{#if error}}
Error message:
```
{{error}}
```
{{/if}}

{{#if code}}
Relevant code:
```
{{code}}
```
{{/if}}

{{#if context}}
Context: {{context}}
{{/if}}

Please:
1. Identify the root cause
2. Explain what's happening
3. Suggest a fix
4. Recommend preventive measures
"#;

/// Built-in template for documentation.
pub const DOCUMENT: &str = r#"# Documentation Request

Please write comprehensive documentation for:

{{#if type}}
Type: {{type}}
{{/if}}

{{#if name}}
Name: {{name}}
{{/if}}

{{#if code}}
Code:
```
{{code}}
```
{{/if}}

Include:
- Purpose and overview
- Parameters and return values
- Usage examples
- Edge cases and limitations
{{#if api}}
- API contract
{{/if}}
"#;

/// Built-in template for test generation.
pub const TEST: &str = r#"# Test Generation

Generate comprehensive tests for:

{{#if function}}
Function: {{function}}
{{/if}}

{{#if code}}
Code:
```
{{code}}
```
{{/if}}

Include:
- Unit tests for normal cases
- Edge cases and boundary conditions
- Error handling tests
{{#if integration}}
- Integration tests
{{/if}}
- Test data setup and teardown

{{#if framework}}
Use testing framework: {{framework}}
{{/if}}
"#;

/// Built-in template for refactoring.
pub const REFACTOR: &str = r#"# Refactoring Request

Please refactor this code to improve:

{{#if focus}}
Focus: {{focus}}
{{/if}}

Code:
```
{{code}}
```

Goals:
- Improve readability and maintainability
- Reduce complexity
- Follow best practices
{{#if patterns}}
- Apply design patterns: {{patterns}}
{{/if}}
- Preserve existing functionality

Provide the refactored code with explanations of changes.
"#;

/// Get a built-in template by name.
pub fn get_builtin(name: &str) -> Option<&'static str> {
    match name {
        "code_review" => Some(CODE_REVIEW),
        "debug" => Some(DEBUG),
        "document" => Some(DOCUMENT),
        "test" => Some(TEST),
        "refactor" => Some(REFACTOR),
        _ => None,
    }
}

/// List all built-in template names.
pub fn list_builtins() -> Vec<&'static str> {
    vec!["code_review", "debug", "document", "test", "refactor"]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::engine::{TemplateContext, render_simple};

    #[test]
    fn test_code_review_template() {
        let mut context = TemplateContext::new();
        context.insert("file".to_string(), "test.rs".to_string());
        context.insert("language".to_string(), "Rust".to_string());
        context.insert("code".to_string(), "fn main() {}".to_string());

        let result = render_simple(CODE_REVIEW, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => {
                assert!(r.contains("test.rs"));
                assert!(r.contains("Rust"));
                assert!(r.contains("fn main() {}"));
            }
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_debug_template() {
        let mut context = TemplateContext::new();
        context.insert("error".to_string(), "null pointer".to_string());
        context.insert("code".to_string(), "let x = None;".to_string());

        let result = render_simple(DEBUG, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => {
                assert!(r.contains("null pointer"));
                assert!(r.contains("let x = None;"));
            }
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_document_template() {
        let mut context = TemplateContext::new();
        context.insert("type".to_string(), "function".to_string());
        context.insert("name".to_string(), "process_data".to_string());
        context.insert("code".to_string(), "fn process_data() {}".to_string());

        let result = render_simple(DOCUMENT, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => {
                assert!(r.contains("function"));
                assert!(r.contains("process_data"));
            }
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_test_template() {
        let mut context = TemplateContext::new();
        context.insert("function".to_string(), "add".to_string());
        context.insert(
            "code".to_string(),
            "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
        );
        context.insert("framework".to_string(), "pytest".to_string());

        let result = render_simple(TEST, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => {
                assert!(r.contains("add"));
                assert!(r.contains("pytest"));
            }
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_refactor_template() {
        let mut context = TemplateContext::new();
        context.insert("focus".to_string(), "performance".to_string());
        context.insert("code".to_string(), "fn slow() {}".to_string());
        context.insert("patterns".to_string(), "caching".to_string());

        let result = render_simple(REFACTOR, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => {
                assert!(r.contains("performance"));
                assert!(r.contains("caching"));
            }
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_get_builtin_valid() {
        assert!(get_builtin("code_review").is_some());
        assert!(get_builtin("debug").is_some());
        assert!(get_builtin("document").is_some());
        assert!(get_builtin("test").is_some());
        assert!(get_builtin("refactor").is_some());
    }

    #[test]
    fn test_get_builtin_invalid() {
        assert!(get_builtin("nonexistent").is_none());
    }

    #[test]
    fn test_list_builtins() {
        let builtins = list_builtins();
        assert_eq!(builtins.len(), 5);
        assert!(builtins.contains(&"code_review"));
        assert!(builtins.contains(&"debug"));
        assert!(builtins.contains(&"document"));
        assert!(builtins.contains(&"test"));
        assert!(builtins.contains(&"refactor"));
    }
}
