//! Document model with YAML frontmatter support

use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

/// Document publication status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Draft,
    Public,
}

impl Default for DocumentStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// YAML frontmatter structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    #[serde(default)]
    pub status: DocumentStatus,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created: Option<NaiveDate>,
    pub updated: Option<NaiveDate>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// A document in the knowledge base
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub title: String,
    pub status: DocumentStatus,
    pub tags: Vec<String>,
    pub created: Option<NaiveDate>,
    pub updated: Option<NaiveDate>,
    pub aliases: Vec<String>,
    pub content: String,
    pub raw_content: String,
}

impl Document {
    /// Load document from file path
    pub fn load(path: &Path) -> Result<Self> {
        let raw_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        
        let (frontmatter, content) = Self::parse_frontmatter(&raw_content)?;
        
        Ok(Self {
            path: path.to_path_buf(),
            title: frontmatter.title,
            status: frontmatter.status,
            tags: frontmatter.tags,
            created: frontmatter.created,
            updated: frontmatter.updated,
            aliases: frontmatter.aliases,
            content,
            raw_content,
        })
    }
    
    /// Parse YAML frontmatter from document content
    fn parse_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
        // Check for frontmatter delimiter
        if !content.starts_with("---") {
            // No frontmatter, use filename as title
            return Ok((
                Frontmatter {
                    title: "Untitled".to_string(),
                    status: DocumentStatus::Draft,
                    tags: Vec::new(),
                    created: None,
                    updated: None,
                    aliases: Vec::new(),
                },
                content.to_string(),
            ));
        }
        
        // Find closing delimiter
        let rest = &content[3..];
        if let Some(end_pos) = rest.find("\n---") {
            let yaml_content = &rest[..end_pos].trim();
            let body = &rest[end_pos + 4..].trim_start();
            
            let frontmatter: Frontmatter = serde_yaml::from_str(yaml_content)
                .with_context(|| "Failed to parse frontmatter YAML")?;
            
            Ok((frontmatter, body.to_string()))
        } else {
            anyhow::bail!("Invalid frontmatter: missing closing ---");
        }
    }
    
    /// Save document back to file
    pub fn save(&self) -> Result<()> {
        let frontmatter = Frontmatter {
            title: self.title.clone(),
            status: self.status,
            tags: self.tags.clone(),
            created: self.created,
            updated: self.updated,
            aliases: self.aliases.clone(),
        };
        
        let yaml = serde_yaml::to_string(&frontmatter)?;
        let content = format!("---\n{}---\n\n{}", yaml, self.content);
        
        std::fs::write(&self.path, content)?;
        Ok(())
    }
    
    /// Get document slug (URL-safe name)
    pub fn slug(&self) -> String {
        self.path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .replace(' ', "-")
    }
    
    /// Check if document matches search query
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.title.to_lowercase().contains(&query)
            || self.content.to_lowercase().contains(&query)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Test Document
status: public
tags:
  - rust
  - test
---

# Hello World

This is content."#;
        
        let (fm, body) = Document::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test Document");
        assert_eq!(fm.status, DocumentStatus::Public);
        assert_eq!(fm.tags, vec!["rust", "test"]);
        assert!(body.contains("# Hello World"));
    }
    
    #[test]
    fn test_no_frontmatter() {
        let content = "# Just content\n\nNo frontmatter here.";
        let (fm, body) = Document::parse_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Untitled");
        assert_eq!(fm.status, DocumentStatus::Draft);
        assert!(body.contains("Just content"));
    }
}
