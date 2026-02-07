//! Full-text search using sled embedded database

use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Search index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub score: f32,
}

/// Full-text search index
pub struct SearchIndex {
    db: sled::Db,
    /// Word -> document slugs mapping
    index_tree: sled::Tree,
    /// Document metadata
    docs_tree: sled::Tree,
}

impl SearchIndex {
    /// Open or create search index
    pub fn open(root: &Path) -> Result<Self> {
        let db_path = root.join(".lightdocs_search");
        let db = sled::open(&db_path)?;
        
        let index_tree = db.open_tree("word_index")?;
        let docs_tree = db.open_tree("documents")?;
        
        Ok(Self {
            db,
            index_tree,
            docs_tree,
        })
    }
    
    /// Index a document
    pub fn index_document(&self, slug: &str, title: &str, content: &str) -> Result<()> {
        // Store document metadata
        let doc_data = serde_json::json!({
            "title": title,
            "excerpt": Self::create_excerpt(content),
        });
        self.docs_tree.insert(slug.as_bytes(), doc_data.to_string().as_bytes())?;
        
        // Tokenize and index words
        let words = Self::tokenize(content);
        for word in words {
            // Get existing doc list for this word
            let key = word.to_lowercase();
            let mut slugs: Vec<String> = self.index_tree
                .get(key.as_bytes())?
                .map(|v| serde_json::from_slice(&v).unwrap_or_default())
                .unwrap_or_default();
            
            if !slugs.contains(&slug.to_string()) {
                slugs.push(slug.to_string());
                let value = serde_json::to_vec(&slugs)?;
                self.index_tree.insert(key.as_bytes(), value)?;
            }
        }
        
        self.db.flush()?;
        Ok(())
    }
    
    /// Search for documents matching query
    pub fn search(&self, query: &str) -> Result<Vec<SearchEntry>> {
        let query_words = Self::tokenize(query);
        let mut doc_scores: HashMap<String, f32> = HashMap::new();
        
        // Find documents containing query words
        for word in &query_words {
            let key = word.to_lowercase();
            if let Some(value) = self.index_tree.get(key.as_bytes())? {
                let slugs: Vec<String> = serde_json::from_slice(&value)?;
                for slug in slugs {
                    *doc_scores.entry(slug).or_insert(0.0) += 1.0;
                }
            }
        }
        
        // Normalize scores
        let max_score = query_words.len() as f32;
        for score in doc_scores.values_mut() {
            *score /= max_score;
        }
        
        // Build result list
        let mut results: Vec<SearchEntry> = doc_scores
            .into_iter()
            .filter_map(|(slug, score)| {
                self.docs_tree.get(slug.as_bytes()).ok()?.map(|v| {
                    let doc: serde_json::Value = serde_json::from_slice(&v).ok()?;
                    Some(SearchEntry {
                        slug,
                        title: doc["title"].as_str()?.to_string(),
                        excerpt: doc["excerpt"].as_str()?.to_string(),
                        score,
                    })
                })?
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(results)
    }
    
    /// Clear the index
    pub fn clear(&self) -> Result<()> {
        self.index_tree.clear()?;
        self.docs_tree.clear()?;
        self.db.flush()?;
        Ok(())
    }
    
    /// Tokenize text into words
    fn tokenize(text: &str) -> Vec<String> {
        text.split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| w.to_lowercase())
            .collect()
    }
    
    /// Create short excerpt from content
    fn create_excerpt(content: &str) -> String {
        let clean: String = content
            .lines()
            .filter(|l| !l.starts_with('#'))
            .take(3)
            .collect::<Vec<_>>()
            .join(" ");
        
        if clean.len() > 150 {
            format!("{}...", &clean[..150])
        } else {
            clean
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_search_index() {
        let dir = tempdir().unwrap();
        let index = SearchIndex::open(dir.path()).unwrap();
        
        index.index_document("test", "Test Document", "Hello world from Rust").unwrap();
        
        let results = index.search("world").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "test");
    }
}
