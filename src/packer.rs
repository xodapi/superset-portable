//! Fast release packing with Zstd compression
//! 
//! Replaces PowerShell Compress-Archive with native Rust implementation
//! for 5-10x faster release packaging.

use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};
use tracing::{info, warn};

/// Release packer configuration
pub struct ReleasePacker {
    root: PathBuf,
    output_dir: PathBuf,
    release_name: String,
}

/// Packing statistics
#[derive(Debug)]
pub struct PackStats {
    pub files_packed: usize,
    pub total_size_bytes: u64,
    pub compressed_size_bytes: u64,
    pub duration_secs: f64,
    pub compression_ratio: f64,
}

impl PackStats {
    pub fn summary(&self) -> String {
        format!(
            "📦 Packed {} files ({:.1} MB → {:.1} MB, {:.1}% compression) in {:.1}s",
            self.files_packed,
            self.total_size_bytes as f64 / 1_048_576.0,
            self.compressed_size_bytes as f64 / 1_048_576.0,
            (1.0 - self.compression_ratio) * 100.0,
            self.duration_secs
        )
    }
}

impl ReleasePacker {
    /// Create a new release packer
    pub fn new(root: &Path) -> Self {
        let release_name = format!("superset-portable-v6.0-ru");
        Self {
            root: root.to_path_buf(),
            output_dir: root.join("release"),
            release_name,
        }
    }
    
    /// Pack the release using standard ZIP with deflate
    pub fn pack_zip(&self) -> Result<PackStats> {
        let start = Instant::now();
        
        fs::create_dir_all(&self.output_dir)?;
        
        let zip_path = self.output_dir.join(format!("{}.zip", self.release_name));
        let staging_dir = self.output_dir.join(&self.release_name);
        
        // Create staging directory with all files
        info!("📂 Preparing release files...");
        self.prepare_staging(&staging_dir)?;
        
        // Create ZIP archive
        info!("🗜️ Creating ZIP archive...");
        let (files_packed, total_size) = self.create_zip(&staging_dir, &zip_path)?;
        
        let compressed_size = fs::metadata(&zip_path)?.len();
        let duration = start.elapsed().as_secs_f64();
        
        let stats = PackStats {
            files_packed,
            total_size_bytes: total_size,
            compressed_size_bytes: compressed_size,
            duration_secs: duration,
            compression_ratio: compressed_size as f64 / total_size as f64,
        };
        
        info!("{}", stats.summary());
        info!("📍 Output: {}", zip_path.display());
        
        Ok(stats)
    }
    
    /// Pack the release using Zstd compression (faster, better ratio)
    pub fn pack_zstd(&self) -> Result<PackStats> {
        let start = Instant::now();
        
        fs::create_dir_all(&self.output_dir)?;
        
        let archive_path = self.output_dir.join(format!("{}.tar.zst", self.release_name));
        let staging_dir = self.output_dir.join(&self.release_name);
        
        // Create staging directory with all files
        info!("📂 Preparing release files...");
        self.prepare_staging(&staging_dir)?;
        
        // Create tar.zst archive
        info!("🗜️ Creating Zstd archive (fast mode)...");
        let (files_packed, total_size) = self.create_tar_zstd(&staging_dir, &archive_path)?;
        
        let compressed_size = fs::metadata(&archive_path)?.len();
        let duration = start.elapsed().as_secs_f64();
        
        let stats = PackStats {
            files_packed,
            total_size_bytes: total_size,
            compressed_size_bytes: compressed_size,
            duration_secs: duration,
            compression_ratio: compressed_size as f64 / total_size as f64,
        };
        
        info!("{}", stats.summary());
        info!("📍 Output: {}", archive_path.display());
        
        Ok(stats)
    }
    
    /// Prepare staging directory with release files
    fn prepare_staging(&self, staging: &Path) -> Result<()> {
        // Components to include
        let components = [
            ("python", "python"),
            ("superset_home", "superset_home"),
            ("docs", "docs"),
            ("start_superset.bat", "start_superset.bat"),
            ("start_docs.bat", "start_docs.bat"),
            ("superset-launcher.exe", "superset-launcher.exe"),
            ("LICENSE", "LICENSE"),
            ("NOTICE", "NOTICE"),
            ("QUICKSTART.md", "README.txt"),
        ];
        
        for (src, dst) in &components {
            let src_path = self.root.join(src);
            let dst_path = staging.join(dst);
            
            if src_path.exists() {
                if src_path.is_dir() {
                    if !dst_path.exists() {
                        info!("  Copying directory: {}", src);
                        copy_dir_all(&src_path, &dst_path)?;
                    }
                } else {
                    fs::create_dir_all(dst_path.parent().unwrap())?;
                    if !dst_path.exists() {
                        fs::copy(&src_path, &dst_path)?;
                    }
                }
            } else {
                warn!("  Skipping missing: {}", src);
            }
        }
        
        Ok(())
    }
    
    /// Create ZIP archive from staging directory
    fn create_zip(&self, staging: &Path, output: &Path) -> Result<(usize, u64)> {
        let file = File::create(output)?;
        let writer = BufWriter::new(file);
        let mut zip = ZipWriter::new(writer);
        
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));
        
        let mut files_count = 0;
        let mut total_size = 0u64;
        
        for entry in WalkDir::new(staging).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(staging)?;
            
            if relative.as_os_str().is_empty() {
                continue;
            }
            
            let name = relative.to_string_lossy().replace("\\", "/");
            
            if path.is_dir() {
                zip.add_directory(&name, options)?;
            } else {
                zip.start_file(&name, options)?;
                let mut file = BufReader::new(File::open(path)?);
                let size = io::copy(&mut file, &mut zip)?;
                total_size += size;
                files_count += 1;
                
                if files_count % 1000 == 0 {
                    info!("  {} files processed...", files_count);
                }
            }
        }
        
        zip.finish()?;
        Ok((files_count, total_size))
    }
    
    /// Create tar.zst archive (faster than ZIP)
    fn create_tar_zstd(&self, staging: &Path, output: &Path) -> Result<(usize, u64)> {
        let file = File::create(output)?;
        let encoder = zstd::Encoder::new(file, 3)?;  // Level 3 = fast
        let mut tar = tar::Builder::new(encoder);
        
        let mut files_count = 0;
        let mut total_size = 0u64;
        
        for entry in WalkDir::new(staging).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let relative = path.strip_prefix(staging)?;
            
            if relative.as_os_str().is_empty() {
                continue;
            }
            
            if path.is_file() {
                let size = path.metadata()?.len();
                tar.append_path_with_name(path, relative)?;
                total_size += size;
                files_count += 1;
                
                if files_count % 1000 == 0 {
                    info!("  {} files processed...", files_count);
                }
            }
        }
        
        let encoder = tar.into_inner()?;
        encoder.finish()?;
        
        Ok((files_count, total_size))
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

/// Default release format
pub enum ReleaseFormat {
    Zip,
    TarZstd,
}
