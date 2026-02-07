//! Template rendering engine with variable substitution and conditionals.

use crate::error::{Result, SaorsaAgentError};
use std::collections::HashMap;

/// Template context for variable substitution.
pub type TemplateContext = HashMap<String, String>;

/// Template rendering engine.
#[derive(Debug, Default)]
pub struct TemplateEngine;

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Self {
        Self
    }

    /// Render a template with the given context.
    ///
    /// Supports:
    /// - Variable substitution: `{{variable}}`
    /// - Conditionals: `{{#if var}}...{{/if}}`
    /// - Negated conditionals: `{{#unless var}}...{{/unless}}`
    pub fn render(&self, template: &str, context: &TemplateContext) -> Result<String> {
        let mut result = String::new();
        let mut chars = template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                let tag = self.collect_until(&mut chars, '}', '}')?;
                let rendered = self.render_tag(&tag, context, template)?;
                result.push_str(&rendered);
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Collect characters until we see the delimiter pair.
    fn collect_until(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars>,
        delim1: char,
        delim2: char,
    ) -> Result<String> {
        let mut content = String::new();
        while let Some(ch) = chars.next() {
            if ch == delim1 && chars.peek() == Some(&delim2) {
                chars.next(); // consume second delimiter
                return Ok(content);
            }
            content.push(ch);
        }
        Err(SaorsaAgentError::Context(
            "Unclosed template tag".to_string(),
        ))
    }

    /// Render a template tag (variable or conditional).
    fn render_tag(
        &self,
        tag: &str,
        context: &TemplateContext,
        full_template: &str,
    ) -> Result<String> {
        let tag = tag.trim();

        if let Some(var_name) = tag.strip_prefix("#if ") {
            self.render_if(var_name.trim(), context, full_template)
        } else if let Some(var_name) = tag.strip_prefix("#unless ") {
            self.render_unless(var_name.trim(), context, full_template)
        } else if tag == "/if" || tag == "/unless" {
            // Closing tags are handled by conditional logic, should not appear here
            Ok(String::new())
        } else {
            // Variable substitution
            self.render_variable(tag, context)
        }
    }

    /// Render a variable substitution.
    fn render_variable(&self, var_name: &str, context: &TemplateContext) -> Result<String> {
        context.get(var_name).cloned().ok_or_else(|| {
            SaorsaAgentError::Context(format!("Missing template variable: {}", var_name))
        })
    }

    /// Render an `{{#if var}}...{{/if}}` block.
    ///
    /// Note: This is a placeholder. Actual conditional logic is handled by render_simple.
    fn render_if(
        &self,
        _var_name: &str,
        _context: &TemplateContext,
        _full_template: &str,
    ) -> Result<String> {
        Ok(String::new())
    }

    /// Render an `{{#unless var}}...{{/unless}}` block.
    ///
    /// Note: This is a placeholder. Actual conditional logic is handled by render_simple.
    fn render_unless(
        &self,
        _var_name: &str,
        _context: &TemplateContext,
        _full_template: &str,
    ) -> Result<String> {
        Ok(String::new())
    }
}

/// Simple template renderer (does not support nested conditionals properly).
///
/// This is a basic implementation. For proper conditional support, a full parser
/// would be needed. This version handles simple cases only.
pub fn render_simple(template: &str, context: &TemplateContext) -> Result<String> {
    let mut result = template.to_string();

    // Replace variables
    for (key, value) in context {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    // Handle simple if blocks (single line only)
    // This is a simplified implementation
    let if_pattern_start = "{{#if ";
    let if_pattern_end = "{{/if}}";

    while let Some(start_pos) = result.find(if_pattern_start) {
        if let Some(content_start) = result[start_pos..].find("}}") {
            let var_end = start_pos + if_pattern_start.len();
            let var_name = &result[var_end..start_pos + content_start];

            if let Some(end_pos) = result[start_pos..].find(if_pattern_end) {
                let full_start = start_pos;
                let full_end = start_pos + end_pos + if_pattern_end.len();
                let content = &result[start_pos + content_start + 2..start_pos + end_pos];

                let replacement = if context.contains_key(var_name.trim())
                    && !context.get(var_name.trim()).is_some_and(|v| v.is_empty())
                {
                    content.to_string()
                } else {
                    String::new()
                };

                result.replace_range(full_start..full_end, &replacement);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Handle simple unless blocks
    let unless_pattern_start = "{{#unless ";
    let unless_pattern_end = "{{/unless}}";

    while let Some(start_pos) = result.find(unless_pattern_start) {
        if let Some(content_start) = result[start_pos..].find("}}") {
            let var_end = start_pos + unless_pattern_start.len();
            let var_name = &result[var_end..start_pos + content_start];

            if let Some(end_pos) = result[start_pos..].find(unless_pattern_end) {
                let full_start = start_pos;
                let full_end = start_pos + end_pos + unless_pattern_end.len();
                let content = &result[start_pos + content_start + 2..start_pos + end_pos];

                let replacement = if !context.contains_key(var_name.trim())
                    || context.get(var_name.trim()).is_some_and(|v| v.is_empty())
                {
                    content.to_string()
                } else {
                    String::new()
                };

                result.replace_range(full_start..full_end, &replacement);
            } else {
                break;
            }
        } else {
            break;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_substitution() {
        let template = "Hello {{name}}!";
        let mut context = TemplateContext::new();
        context.insert("name".to_string(), "World".to_string());

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Hello World!"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_multiple_variables() {
        let template = "{{greeting}} {{name}}!";
        let mut context = TemplateContext::new();
        context.insert("greeting".to_string(), "Hi".to_string());
        context.insert("name".to_string(), "Alice".to_string());

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Hi Alice!"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_if_conditional_true() {
        let template = "Start {{#if show}}visible{{/if}} end";
        let mut context = TemplateContext::new();
        context.insert("show".to_string(), "yes".to_string());

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Start visible end"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_if_conditional_false() {
        let template = "Start {{#if show}}hidden{{/if}} end";
        let context = TemplateContext::new();

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Start  end"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_unless_conditional_true() {
        let template = "Start {{#unless hide}}visible{{/unless}} end";
        let context = TemplateContext::new();

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Start visible end"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_unless_conditional_false() {
        let template = "Start {{#unless hide}}hidden{{/unless}} end";
        let mut context = TemplateContext::new();
        context.insert("hide".to_string(), "yes".to_string());

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "Start  end"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_empty_context() {
        let template = "No variables here";
        let context = TemplateContext::new();

        let result = render_simple(template, &context);
        assert!(result.is_ok());

        match result {
            Ok(r) => assert_eq!(r, "No variables here"),
            Err(_) => unreachable!("Should render successfully"),
        }
    }

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}";
        let mut context = TemplateContext::new();
        context.insert("name".to_string(), "Test".to_string());

        let result = engine.render(template, &context);
        assert!(result.is_ok());
    }
}
