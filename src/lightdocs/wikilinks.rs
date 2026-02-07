//! Wikilinks transformer [[Article Name]] -> [Article Name](./article-name.html)

use regex::Regex;
use std::collections::HashMap;

/// Transforms wikilinks to standard markdown links
pub struct WikilinksTransformer {
    /// Map of document titles/aliases to their slugs
    title_map: HashMap<String, String>,
}

impl WikilinksTransformer {
    /// Create new transformer
    pub fn new() -> Self {
        Self {
            title_map: HashMap::new(),
        }
    }
    
    /// Register a document title -> slug mapping
    pub fn register(&mut self, title: &str, slug: &str) {
        self.title_map.insert(title.to_lowercase(), slug.to_string());
    }
    
    /// Register document with aliases
    pub fn register_with_aliases(&mut self, title: &str, aliases: &[String], slug: &str) {
        self.register(title, slug);
        for alias in aliases {
            self.title_map.insert(alias.to_lowercase(), slug.to_string());
        }
    }
    
    /// Transform all wikilinks in content to standard links
    pub fn transform(&self, content: &str) -> String {
        // Match [[Title]] or [[Title|Display Text]]
        let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
        
        re.replace_all(content, |caps: &regex::Captures| {
            let title = &caps[1];
            let display = caps.get(2)
                .map(|m| m.as_str())
                .unwrap_or(title);
            
            // Look up slug in map, or create from title
            let slug = self.title_map
                .get(&title.to_lowercase())
                .map(|s| s.clone())
                .unwrap_or_else(|| Self::title_to_slug(title));
            
            format!("[{}](./{}.html)", display, slug)
        }).to_string()
    }
    
    /// Convert title to URL-safe slug
    pub fn title_to_slug(title: &str) -> String {
        let slug: String = title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c == ' ' || c == '-' || c == '_' {
                    '-'
                } else if c.is_alphabetic() {
                    // Handle Cyrillic and other non-ASCII
                    c
                } else {
                    '-'
                }
            })
            .collect();
            
        let re = Regex::new(r"-+").unwrap();
        re.replace_all(&slug, "-")
            .trim_matches('-')
            .to_string()
    }
    
    /// Extract all wikilinks from content
    pub fn extract_links(content: &str) -> Vec<String> {
        let re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]+)?\]\]").unwrap();
        re.captures_iter(content)
            .map(|c| c[1].to_string())
            .collect()
    }
    
    /// Find broken links (links to non-existent documents)
    pub fn find_broken_links(&self, content: &str) -> Vec<String> {
        Self::extract_links(content)
            .into_iter()
            .filter(|title| !self.title_map.contains_key(&title.to_lowercase()))
            .collect()
    }
}

impl Default for WikilinksTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_wikilink() {
        let mut transformer = WikilinksTransformer::new();
        transformer.register("Руководство", "руководство");
        
        let input = "Смотрите [[Руководство]] для деталей.";
        let output = transformer.transform(input);
        
        assert_eq!(output, "Смотрите [Руководство](./руководство.html) для деталей.");
    }
    
    #[test]
    fn test_wikilink_with_display() {
        let mut transformer = WikilinksTransformer::new();
        transformer.register("FAQ", "faq");
        
        let input = "Читайте [[FAQ|Частые вопросы]] здесь.";
        let output = transformer.transform(input);
        
        assert_eq!(output, "Читайте [Частые вопросы](./faq.html) здесь.");
    }
    
    #[test]
    fn test_title_to_slug() {
        assert_eq!(WikilinksTransformer::title_to_slug("Hello World"), "hello-world");
        assert_eq!(WikilinksTransformer::title_to_slug("Руководство"), "руководство");
        assert_eq!(WikilinksTransformer::title_to_slug("Test -- Page"), "test-page");
    }
    
    #[test]
    fn test_extract_links() {
        let content = "See [[Page1]] and [[Page2|Alias]] for more.";
        let links = WikilinksTransformer::extract_links(content);
        assert_eq!(links, vec!["Page1", "Page2"]);
    }
}
