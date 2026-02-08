//! Web search tool using DuckDuckGo.
//!
//! Provides web search capabilities without requiring any API keys.
//! Uses DuckDuckGo's HTML search endpoint to retrieve results.
//!
//! # Usage
//!
//! ```rust
//! use saorsa_agent::{WebSearchTool, Tool};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let tool = WebSearchTool::new();
//! let result = tool.execute(serde_json::json!({
//!     "query": "Rust programming language"
//! })).await?;
//! println!("{}", result);
//! # Ok(())
//! # }
//! ```

use tracing::debug;

use crate::error::{Result, SaorsaAgentError};
use crate::tool::Tool;

/// Default maximum number of search results to return.
const DEFAULT_MAX_RESULTS: usize = 5;

/// Maximum allowed results to prevent excessive output.
const MAX_RESULTS_LIMIT: usize = 20;

/// Maximum response body size in bytes (1 MB).
const MAX_RESPONSE_BYTES: usize = 1_048_576;

/// User agent string for HTTP requests.
const USER_AGENT: &str = "Mozilla/5.0 (compatible; saorsa/0.1)";

/// Web search tool that queries DuckDuckGo for current information.
///
/// No API key is required -- DuckDuckGo's HTML search endpoint is free to use.
/// Results include titles, URLs, and text snippets.
pub struct WebSearchTool {
    /// HTTP client for making requests.
    client: reqwest::Client,
}

impl WebSearchTool {
    /// Create a new web search tool.
    pub fn new() -> Self {
        let client = match reqwest::Client::builder().user_agent(USER_AGENT).build() {
            Ok(c) => c,
            Err(_) => reqwest::Client::new(),
        };
        Self { client }
    }
}

impl Default for WebSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// A single search result extracted from DuckDuckGo HTML.
#[derive(Debug)]
struct SearchResult {
    /// The result title.
    title: String,
    /// The result URL.
    url: String,
    /// The result text snippet.
    snippet: String,
}

/// Parse DuckDuckGo HTML response to extract search results.
///
/// Looks for result links with `class="result__a"` and snippets with
/// `class="result__snippet"`. Uses simple string searching rather than
/// a full HTML parser to keep dependencies minimal.
fn parse_ddg_html(html: &str, max_results: usize) -> Vec<SearchResult> {
    let mut results = Vec::new();
    let mut search_pos = 0;

    while results.len() < max_results {
        // Find the next result link: <a rel="nofollow" class="result__a" href="URL">TITLE</a>
        let link_marker = "class=\"result__a\"";
        let Some(marker_pos) = html[search_pos..].find(link_marker) else {
            break;
        };
        let abs_marker = search_pos + marker_pos;
        search_pos = abs_marker + link_marker.len();

        // Extract href from the anchor tag.
        // Look backwards first (href before class), then forward (href after class).
        let tag_start_region = abs_marker.saturating_sub(200);
        let backward_region = &html[tag_start_region..abs_marker];
        let url = extract_href(backward_region).unwrap_or_else(|| {
            // Look forward from the marker to the closing > of the tag
            let forward_end = (search_pos + 500).min(html.len());
            let forward_region = &html[search_pos..forward_end];
            let tag_end = forward_region.find('>').unwrap_or(forward_region.len());
            extract_href(&forward_region[..tag_end]).unwrap_or_default()
        });

        // Extract title text between > and </a>
        let title = match html[search_pos..].find('>') {
            Some(gt_pos) => {
                let text_start = search_pos + gt_pos + 1;
                match html[text_start..].find("</a>") {
                    Some(end_pos) => strip_html_tags(&html[text_start..text_start + end_pos]),
                    None => String::new(),
                }
            }
            None => String::new(),
        };

        // Find the snippet: <a class="result__snippet" ...>SNIPPET</a>
        // or <td class="result__snippet">SNIPPET</td>
        let snippet = extract_snippet(html, search_pos);

        // Skip results with no useful content
        if url.is_empty() && title.is_empty() {
            continue;
        }

        // Decode the DuckDuckGo redirect URL if needed
        let decoded_url = decode_ddg_url(&url);

        results.push(SearchResult {
            title: clean_text(&title),
            url: decoded_url,
            snippet: clean_text(&snippet),
        });
    }

    results
}

/// Extract href value from an HTML tag fragment.
///
/// Looks for `href="..."` pattern and returns the URL content.
fn extract_href(tag_html: &str) -> Option<String> {
    let href_marker = "href=\"";
    let href_pos = tag_html.rfind(href_marker)?;
    let url_start = href_pos + href_marker.len();
    let remaining = &tag_html[url_start..];
    let url_end = remaining.find('"')?;
    Some(remaining[..url_end].to_string())
}

/// Extract snippet text near the current search position.
///
/// Looks for `class="result__snippet"` within the next chunk of HTML.
fn extract_snippet(html: &str, search_pos: usize) -> String {
    let snippet_marker = "class=\"result__snippet\"";
    let search_end = (search_pos + 2000).min(html.len());
    let region = &html[search_pos..search_end];

    let Some(snip_pos) = region.find(snippet_marker) else {
        return String::new();
    };

    let after_marker = search_pos + snip_pos + snippet_marker.len();
    let remaining = &html[after_marker..];

    // Find the opening > after the class attribute
    let Some(gt_pos) = remaining.find('>') else {
        return String::new();
    };
    let text_start = gt_pos + 1;

    // Find the closing tag (</a> or </td>)
    let text_region = &remaining[text_start..];
    let end_pos = text_region
        .find("</a>")
        .or_else(|| text_region.find("</td>"))
        .unwrap_or(text_region.len().min(500));

    strip_html_tags(&text_region[..end_pos])
}

/// Decode DuckDuckGo redirect URLs.
///
/// DuckDuckGo wraps result URLs in redirects like:
/// `//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&rut=...`
///
/// This extracts and decodes the actual target URL.
fn decode_ddg_url(url: &str) -> String {
    if let Some(uddg_pos) = url.find("uddg=") {
        let encoded_start = uddg_pos + 5;
        let encoded = &url[encoded_start..];
        let encoded_end = encoded.find('&').unwrap_or(encoded.len());
        let encoded_url = &encoded[..encoded_end];
        url_decode(encoded_url)
    } else if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("//") {
        format!("https:{url}")
    } else {
        url.to_string()
    }
}

/// Simple percent-decoding for URLs.
fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.bytes();

    loop {
        let Some(b) = chars.next() else {
            break;
        };
        if b == b'%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(h), Some(l)) = (hi, lo) {
                let hex = [h, l];
                if let Ok(s) = std::str::from_utf8(&hex)
                    && let Ok(val) = u8::from_str_radix(s, 16)
                {
                    result.push(val as char);
                    continue;
                }
                // If decoding fails, keep the original sequence
                result.push('%');
                result.push(h as char);
                result.push(l as char);
            } else {
                result.push('%');
            }
        } else if b == b'+' {
            result.push(' ');
        } else {
            result.push(b as char);
        }
    }

    result
}

/// Strip HTML tags from a string, returning just the text content.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
}

/// Clean text by normalizing whitespace and decoding common HTML entities.
fn clean_text(text: &str) -> String {
    let decoded = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&#x27;", "'")
        .replace("&nbsp;", " ");

    // Normalize whitespace
    let mut result = String::with_capacity(decoded.len());
    let mut prev_space = false;

    for ch in decoded.chars() {
        if ch.is_whitespace() {
            if !prev_space && !result.is_empty() {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }

    // Trim trailing space
    if result.ends_with(' ') {
        result.pop();
    }

    result
}

/// Format search results as a readable string for the LLM.
fn format_results(query: &str, results: &[SearchResult]) -> String {
    if results.is_empty() {
        return format!("No results found for: {query}");
    }

    let mut output = format!("Search results for: {query}\n\n");

    for (i, result) in results.iter().enumerate() {
        output.push_str(&format!("{}. {}\n", i + 1, result.title));
        if !result.url.is_empty() {
            output.push_str(&format!("   URL: {}\n", result.url));
        }
        if !result.snippet.is_empty() {
            output.push_str(&format!("   {}\n", result.snippet));
        }
        output.push('\n');
    }

    output
}

#[async_trait::async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web using DuckDuckGo. Returns titles, URLs, and snippets. No API key required."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5, max: 20)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<String> {
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| SaorsaAgentError::Tool("missing 'query' field".into()))?;

        let max_results = input
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|n| (n as usize).min(MAX_RESULTS_LIMIT))
            .unwrap_or(DEFAULT_MAX_RESULTS);

        debug!(query = %query, max_results, "Executing web search");

        let response = self
            .client
            .post("https://html.duckduckgo.com/html/")
            .form(&[("q", query)])
            .send()
            .await
            .map_err(|e| SaorsaAgentError::Tool(format!("search request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            return Err(SaorsaAgentError::Tool(format!(
                "search returned HTTP {status}"
            )));
        }

        // Read the response body with a size limit
        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| SaorsaAgentError::Tool(format!("failed to read response: {e}")))?;

        if body_bytes.len() > MAX_RESPONSE_BYTES {
            return Err(SaorsaAgentError::Tool("search response too large".into()));
        }

        let html = String::from_utf8_lossy(&body_bytes);
        let results = parse_ddg_html(&html, max_results);

        Ok(format_results(query, &results))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn tool_metadata() {
        let tool = WebSearchTool::new();
        assert_eq!(tool.name(), "web_search");
        assert!(!tool.description().is_empty());
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"][0], "query");
    }

    #[test]
    fn default_creates_tool() {
        let tool = WebSearchTool::default();
        assert_eq!(tool.name(), "web_search");
    }

    #[tokio::test]
    async fn missing_query_field_returns_error() {
        let tool = WebSearchTool::new();
        let result = tool.execute(serde_json::json!({})).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("missing 'query' field"));
        }
    }

    #[test]
    fn parse_empty_html() {
        let results = parse_ddg_html("", 5);
        assert!(results.is_empty());
    }

    #[test]
    fn parse_html_with_results() {
        let html = r##"
        <div class="result">
            <a rel="nofollow" href="//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com&rut=abc" class="result__a">
                <span>Example Title</span>
            </a>
            <a class="result__snippet" href="#">This is a snippet about the result.</a>
        </div>
        "##;
        let results = parse_ddg_html(html, 5);
        assert_eq!(results.len(), 1);
        assert!(results[0].title.contains("Example Title"));
        assert_eq!(results[0].url, "https://example.com");
        assert!(results[0].snippet.contains("snippet about the result"));
    }

    #[test]
    fn parse_html_respects_max_results() {
        let html = r#"
        <a rel="nofollow" class="result__a" href="https://one.com">One</a>
        <a class="result__snippet">Snippet one</a>
        <a rel="nofollow" class="result__a" href="https://two.com">Two</a>
        <a class="result__snippet">Snippet two</a>
        <a rel="nofollow" class="result__a" href="https://three.com">Three</a>
        <a class="result__snippet">Snippet three</a>
        "#;
        let results = parse_ddg_html(html, 2);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn decode_ddg_redirect_url() {
        let url = "//duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpath&rut=abc";
        assert_eq!(decode_ddg_url(url), "https://example.com/path");
    }

    #[test]
    fn decode_direct_url() {
        assert_eq!(decode_ddg_url("https://example.com"), "https://example.com");
    }

    #[test]
    fn decode_protocol_relative_url() {
        assert_eq!(
            decode_ddg_url("//example.com/path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn url_decode_basic() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a%2Fb"), "a/b");
        assert_eq!(url_decode("no+encoding"), "no encoding");
        assert_eq!(url_decode("plain"), "plain");
    }

    #[test]
    fn strip_tags() {
        assert_eq!(strip_html_tags("<b>bold</b> text"), "bold text");
        assert_eq!(strip_html_tags("no tags"), "no tags");
        assert_eq!(strip_html_tags("<a href=\"x\">link</a>"), "link");
    }

    #[test]
    fn clean_text_entities() {
        assert_eq!(clean_text("a &amp; b"), "a & b");
        assert_eq!(clean_text("&lt;tag&gt;"), "<tag>");
        assert_eq!(clean_text("it&#39;s"), "it's");
    }

    #[test]
    fn clean_text_whitespace() {
        assert_eq!(clean_text("  hello   world  "), "hello world");
        assert_eq!(clean_text("line\n  break"), "line break");
    }

    #[test]
    fn format_no_results() {
        let output = format_results("test query", &[]);
        assert!(output.contains("No results found"));
        assert!(output.contains("test query"));
    }

    #[test]
    fn format_with_results() {
        let results = vec![SearchResult {
            title: "Test Title".into(),
            url: "https://example.com".into(),
            snippet: "A test snippet".into(),
        }];
        let output = format_results("test", &results);
        assert!(output.contains("Test Title"));
        assert!(output.contains("https://example.com"));
        assert!(output.contains("A test snippet"));
    }

    #[test]
    fn extract_href_from_tag() {
        let tag = r#"<a rel="nofollow" href="https://example.com" class="#;
        assert_eq!(extract_href(tag), Some("https://example.com".to_string()));
    }

    #[test]
    fn extract_href_missing() {
        assert_eq!(extract_href("<a class=\"test\""), None);
    }
}
