//! Markdown rendering utilities.

use pulldown_cmark::{html, Options, Parser};

/// Render Markdown to HTML.
pub fn render_markdown(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(input, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

/// Detect if a file is likely a README.
pub fn is_readme(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower == "readme.md"
        || lower == "readme.markdown"
        || lower == "readme.txt"
        || lower == "readme"
        || lower == "readme.rst"
}

/// Get syntax highlighting language from file extension.
pub fn get_language_from_extension(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "jsx" => "javascript",
        "tsx" => "typescript",
        "go" => "go",
        "java" => "java",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "sh" | "bash" | "zsh" => "bash",
        "yml" | "yaml" => "yaml",
        "json" => "json",
        "xml" => "xml",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" | "sass" => "scss",
        "sql" => "sql",
        "md" | "markdown" => "markdown",
        "toml" => "toml",
        "dockerfile" => "dockerfile",
        "makefile" => "makefile",
        _ => "plaintext",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown() {
        let md = "# Hello\n\nThis is **bold** and *italic*.";
        let html = render_markdown(md);
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn test_is_readme() {
        assert!(is_readme("README.md"));
        assert!(is_readme("readme.md"));
        assert!(is_readme("README"));
        assert!(!is_readme("main.rs"));
    }

    #[test]
    fn test_get_language() {
        assert_eq!(get_language_from_extension("main.rs"), "rust");
        assert_eq!(get_language_from_extension("app.py"), "python");
        assert_eq!(get_language_from_extension("index.js"), "javascript");
        assert_eq!(get_language_from_extension("unknown.xyz"), "plaintext");
    }
}
