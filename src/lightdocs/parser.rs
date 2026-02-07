//! Markdown parser with HTML generation

use anyhow::Result;
use pulldown_cmark::{Parser, Options, html};

use super::document::Document;
use super::wikilinks::WikilinksTransformer;

/// Markdown to HTML parser
pub struct MarkdownParser {
    wikilinks: WikilinksTransformer,
}

impl MarkdownParser {
    /// Create new parser
    pub fn new() -> Self {
        Self {
            wikilinks: WikilinksTransformer::new(),
        }
    }
    
    /// Register document for wikilink resolution
    pub fn register_document(&mut self, title: &str, aliases: &[String], slug: &str) {
        self.wikilinks.register_with_aliases(title, aliases, slug);
    }
    
    /// Render document to full HTML page
    pub fn render(&self, doc: &Document) -> Result<String> {
        let content_html = self.render_content(&doc.content)?;
        
        Ok(format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        :root {{
            --bg: #1a1a2e;
            --surface: #16213e;
            --primary: #0f3460;
            --accent: #e94560;
            --text: #eee;
            --text-muted: #888;
            --code-bg: #0d1117;
            --link: #58a6ff;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.7;
            padding: 2rem;
            max-width: 800px;
            margin: 0 auto;
        }}
        a {{ color: var(--link); text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
        h1, h2, h3, h4 {{ margin: 1.5rem 0 0.75rem; color: var(--accent); }}
        h1 {{ font-size: 2rem; border-bottom: 2px solid var(--primary); padding-bottom: 0.5rem; }}
        h2 {{ font-size: 1.5rem; }}
        h3 {{ font-size: 1.25rem; }}
        p {{ margin: 0.75rem 0; }}
        ul, ol {{ margin: 0.75rem 0; padding-left: 1.5rem; }}
        li {{ margin: 0.25rem 0; }}
        code {{
            font-family: 'Cascadia Code', 'Consolas', monospace;
            background: var(--code-bg);
            padding: 0.125rem 0.375rem;
            border-radius: 4px;
            font-size: 0.875rem;
        }}
        pre {{
            background: var(--code-bg);
            padding: 1rem;
            border-radius: 8px;
            overflow-x: auto;
            margin: 1rem 0;
        }}
        pre code {{ padding: 0; background: none; }}
        blockquote {{
            border-left: 3px solid var(--accent);
            padding-left: 1rem;
            margin: 1rem 0;
            color: var(--text-muted);
            font-style: italic;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 1rem 0;
        }}
        th, td {{
            border: 1px solid var(--primary);
            padding: 0.5rem;
            text-align: left;
        }}
        th {{ background: var(--primary); }}
        hr {{ border: none; border-top: 1px solid var(--primary); margin: 2rem 0; }}
        img {{ max-width: 100%; border-radius: 8px; }}
        .breadcrumb {{
            margin-bottom: 1rem;
            color: var(--text-muted);
        }}
        .breadcrumb a {{ color: var(--text-muted); }}
        .meta {{
            color: var(--text-muted);
            font-size: 0.875rem;
            margin-bottom: 1.5rem;
        }}
        .tags {{ display: flex; gap: 0.5rem; flex-wrap: wrap; margin-top: 0.5rem; }}
        .tag {{
            background: var(--primary);
            padding: 0.125rem 0.5rem;
            border-radius: 4px;
            font-size: 0.75rem;
        }}
    </style>
</head>
<body>
    <nav class="breadcrumb">
        <a href="index.html">← Главная</a>
    </nav>
    <article>
        <h1>{title}</h1>
        <div class="meta">
            {meta}
        </div>
        {content}
    </article>
</body>
</html>"#,
            title = doc.title,
            meta = self.render_meta(doc),
            content = content_html,
        ))
    }
    
    /// Render just the content (markdown -> HTML)
    pub fn render_content(&self, markdown: &str) -> Result<String> {
        // First transform wikilinks
        let content = self.wikilinks.transform(markdown);
        
        // Parse markdown with extensions
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        
        let parser = Parser::new_ext(&content, options);
        
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        Ok(html_output)
    }
    
    /// Render document metadata
    fn render_meta(&self, doc: &Document) -> String {
        let mut parts = Vec::new();
        
        if let Some(created) = doc.created {
            parts.push(format!("📅 {}", created.format("%d.%m.%Y")));
        }
        
        if !doc.tags.is_empty() {
            let tags_html = doc.tags.iter()
                .map(|t| format!("<span class=\"tag\">{}</span>", t))
                .collect::<Vec<_>>()
                .join("");
            parts.push(format!("<div class=\"tags\">{}</div>", tags_html));
        }
        
        parts.join(" ")
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_content() {
        let parser = MarkdownParser::new();
        let html = parser.render_content("# Hello\n\nWorld").unwrap();
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }
    
    #[test]
    fn test_wikilinks_in_render() {
        let mut parser = MarkdownParser::new();
        parser.register_document("FAQ", &[], "faq");
        
        let html = parser.render_content("See [[FAQ]] for help.").unwrap();
        assert!(html.contains("href=\"./faq.html\""));
    }
}
