//! LightDocs - Portable Knowledge Base
//! 
//! An Obsidian-like local knowledge base for offline environments.
//! Features:
//! - Markdown parsing with live preview
//! - Wikilinks support [[Article]] 
//! - Draft/Public publishing
//! - Full-text search
//! - Static HTML export

pub mod parser;
pub mod wikilinks;
pub mod document;
pub mod server;
pub mod search;

use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::info;

pub use parser::MarkdownParser;
pub use wikilinks::WikilinksTransformer;
pub use document::{Document, DocumentStatus};
pub use server::LightDocsServer;

use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use std::sync::mpsc::channel;
use std::time::Duration;

/// LightDocs configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LightDocsConfig {
    /// Root directory for documents
    pub docs_root: PathBuf,
    /// Output directory for generated HTML
    pub output_dir: PathBuf,
    /// Server port
    pub port: u16,
    /// Site title
    pub title: String,
    /// Enable live reload
    pub live_reload: bool,
}

impl Default for LightDocsConfig {
    fn default() -> Self {
        Self {
            docs_root: PathBuf::from("knowledge"),
            output_dir: PathBuf::from("_site"),
            port: 8090,
            title: "LightDocs".to_string(),
            live_reload: true,
        }
    }
}

impl LightDocsConfig {
    /// Load config from root directory
    pub fn load(root: &Path) -> Result<Self> {
        let config_path = root.join("lightdocs.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: LightDocsConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save config to root directory
    pub fn save(&self, root: &Path) -> Result<()> {
        let config_path = root.join("lightdocs.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    /// Get absolute docs root path
    pub fn docs_root_abs(&self, root: &Path) -> PathBuf {
        if self.docs_root.is_absolute() {
            self.docs_root.clone()
        } else {
            root.join(&self.docs_root)
        }
    }
    
    /// Get absolute output path
    pub fn output_dir_abs(&self, root: &Path) -> PathBuf {
        if self.output_dir.is_absolute() {
            self.output_dir.clone()
        } else {
            root.join(&self.output_dir)
        }
    }
}

/// Main LightDocs engine
pub struct LightDocs {
    root: PathBuf,
    config: LightDocsConfig,
    parser: MarkdownParser,
}

impl LightDocs {
    /// Create new LightDocs instance
    pub fn new(root: &Path) -> Result<Self> {
        let config = LightDocsConfig::load(root)?;
        Ok(Self {
            root: root.to_path_buf(),
            config,
            parser: MarkdownParser::new(),
        })
    }
    
    /// Initialize LightDocs directory structure
    pub fn init(&self) -> Result<()> {
        let docs_root = self.config.docs_root_abs(&self.root);
        
        // Create directories
        std::fs::create_dir_all(&docs_root)?;
        std::fs::create_dir_all(self.config.output_dir_abs(&self.root))?;
        
        // Create sample document
        let sample_path = docs_root.join("index.md");
        if !sample_path.exists() {
            let sample_content = r#"---
title: Добро пожаловать в LightDocs
status: public
created: 2026-01-28
---

# Добро пожаловать в LightDocs! 🎉

Это ваша локальная база знаний, работающая полностью оффлайн.

## Возможности

- ✍️ **Markdown редактор** с поддержкой всех стандартных функций
- 🔗 **Wikilinks** — создавайте ссылки через `[[Название статьи]]`
- 📤 **Публикация** — статьи бывают `draft` или `public`
- 🔍 **Поиск** — полнотекстовый поиск по всей базе

## Быстрый старт

1. Создайте файл `.md` в папке `knowledge/`
2. Добавьте frontmatter с `title` и `status`
3. Запустите `lightdocs serve` для просмотра

## Ссылки

- [[Руководство]] — подробная документация
- [[FAQ]] — частые вопросы

---

> Создано с ❤️ для работы в закрытых контурах
"#;
            std::fs::write(&sample_path, sample_content)?;
            info!("Created sample document: {}", sample_path.display());
        }
        
        // Save config
        self.config.save(&self.root)?;
        
        info!("LightDocs initialized at: {}", docs_root.display());
        Ok(())
    }
    
    /// Build static site from markdown files
    pub fn build(&self) -> Result<Vec<Document>> {
        let docs_root = self.config.docs_root_abs(&self.root);
        let output_dir = self.config.output_dir_abs(&self.root);
        
        // Ensure output dir exists
        std::fs::create_dir_all(&output_dir)?;
        
        let mut documents = Vec::new();
        
        // Walk through all markdown files
        for entry in walkdir::WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let path = entry.path();
            let doc = Document::load(path)?;
            
            // Only process public documents
            if doc.status == DocumentStatus::Public {
                let html = self.parser.render(&doc)?;
                
                // Calculate output path
                let rel_path = path.strip_prefix(&docs_root)?;
                let html_path = output_dir.join(rel_path).with_extension("html");
                
                // Ensure parent directory exists
                if let Some(parent) = html_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                
                std::fs::write(&html_path, &html)?;
                info!("Built: {} -> {}", path.display(), html_path.display());
            }
            
            documents.push(doc);
        }
        
        // Generate index page
        self.generate_index(&output_dir, &documents)?;
        
        info!("Built {} documents", documents.len());
        Ok(documents)
    }
    
    /// Generate index.html with list of all public documents
    fn generate_index(&self, output_dir: &Path, documents: &[Document]) -> Result<()> {
        let public_docs: Vec<_> = documents.iter()
            .filter(|d| d.status == DocumentStatus::Public)
            .collect();
        
        let mut html = format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        :root {{
            --bg: #1a1a2e;
            --surface: #16213e;
            --primary: #0f3460;
            --accent: #e94560;
            --text: #eee;
            --text-muted: #888;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: 'Segoe UI', system-ui, sans-serif;
            background: var(--bg);
            color: var(--text);
            line-height: 1.6;
            padding: 2rem;
        }}
        .container {{ max-width: 800px; margin: 0 auto; }}
        h1 {{ 
            color: var(--accent); 
            margin-bottom: 1rem;
            font-size: 2rem;
        }}
        .search {{
            width: 100%;
            padding: 0.75rem 1rem;
            border: 2px solid var(--primary);
            background: var(--surface);
            color: var(--text);
            border-radius: 8px;
            font-size: 1rem;
            margin-bottom: 1.5rem;
        }}
        .search:focus {{ outline: none; border-color: var(--accent); }}
        .doc-list {{ list-style: none; }}
        .doc-item {{
            background: var(--surface);
            padding: 1rem;
            margin-bottom: 0.5rem;
            border-radius: 8px;
            border-left: 3px solid var(--accent);
        }}
        .doc-item:hover {{ background: var(--primary); }}
        .doc-title {{ 
            color: var(--text); 
            text-decoration: none;
            font-weight: 600;
        }}
        .doc-title:hover {{ color: var(--accent); }}
        .doc-meta {{ color: var(--text-muted); font-size: 0.875rem; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>📚 {}</h1>
        <input type="text" class="search" placeholder="Поиск..." id="search">
        <ul class="doc-list" id="docs">
"#, self.config.title, self.config.title);
        
        for doc in public_docs {
            let link = doc.path.file_stem()
                .unwrap_or_default()
                .to_string_lossy();
            html.push_str(&format!(
                r#"            <li class="doc-item" data-title="{}">
                <a href="{}.html" class="doc-title">{}</a>
                <div class="doc-meta">{}</div>
            </li>
"#,
                doc.title.to_lowercase(),
                link,
                doc.title,
                doc.created.map_or("".to_string(), |d| d.format("%d.%m.%Y").to_string())
            ));
        }
        
        html.push_str(r#"        </ul>
    </div>
    <script>
        document.getElementById('search').addEventListener('input', function(e) {
            const query = e.target.value.toLowerCase();
            document.querySelectorAll('.doc-item').forEach(item => {
                const title = item.dataset.title;
                item.style.display = title.includes(query) ? '' : 'none';
            });
        });
    </script>
</body>
</html>"#);
        
        std::fs::write(output_dir.join("index.html"), html)?;
        Ok(())
    }
    
    /// Get all documents
    pub fn list_documents(&self) -> Result<Vec<Document>> {
        let docs_root = self.config.docs_root_abs(&self.root);
        let mut documents = Vec::new();
        
        for entry in walkdir::WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let doc = Document::load(entry.path())?;
            documents.push(doc);
        }
        
        Ok(documents)
    }
    
    /// Watch for changes and rebuild
    pub fn watch(&self) -> Result<()> {
        let (tx, rx) = channel();
        
        let mut watcher = notify::recommended_watcher(move |res: NotifyResult<notify::Event>| {
            match res {
                Ok(event) => {
                    // Only react to content modification
                    if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                        let _ = tx.send(event);
                    }
                },
                Err(e) => info!("Watch error: {:?}", e),
            }
        })?;

        let docs_root = self.config.docs_root_abs(&self.root);
        watcher.watch(&docs_root, RecursiveMode::Recursive)?;
        
        info!("👀 Watching for changes in: {}", docs_root.display());
        
        loop {
            match rx.recv() {
                Ok(_) => {
                    // Debounce slightly
                    std::thread::sleep(Duration::from_millis(100));
                    // Drain other events
                    while let Ok(_) = rx.try_recv() {}
                    
                    info!("🔄 File changed, rebuilding...");
                    if let Err(e) = self.build() {
                        info!("❌ Build failed: {}", e);
                    } else {
                        // Re-index search
                        if let Ok(index) = search::SearchIndex::open(&self.root) {
                            if let Ok(docs) = self.list_documents() {
                                for doc in docs {
                                    let _ = index.index_document(&doc.slug(), &doc.title, &doc.content);
                                }
                            }
                        }
                    }
                }
                Err(e) => info!("Watch error: {}", e),
            }
        }
    }
}
