use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

pub struct DataWatcher {
    root: PathBuf,
    running: Arc<AtomicBool>,
}

impl DataWatcher {
    pub fn new(root: &PathBuf) -> Self {
        Self {
            root: root.clone(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            info!("Watcher already running");
            return;
        }

        let root = self.root.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            info!("Starting Data Watcher on {:?}", root.join("data"));

            let (tx, mut rx) = mpsc::channel(1);

            let mut watcher = match RecommendedWatcher::new(
                move |res| {
                    let _ = tx.blocking_send(res);
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    error!("Failed to create watcher: {}", e);
                    running.store(false, Ordering::SeqCst);
                    return;
                }
            };

            // Watch docs/demo_data (csv files) or data/
            // Based on previous checks, data seems to be in docs/demo_data, but README says "data/"
            // Let's watch both key locations to be safe, or just the one we know works.
            // implementation_plan says "docs/demo_data/".
            
            let watch_path = root.join("docs").join("demo_data");
            
            if !watch_path.exists() {
                 error!("Watch path does not exist: {:?}", watch_path);
                 // Try creating it or fallback?
            }

            if let Err(e) = watcher.watch(&watch_path, RecursiveMode::NonRecursive) {
                error!("Failed to watch path: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }

            info!("Watching for file changes in: {:?}", watch_path);

            while running.load(Ordering::SeqCst) {
                // Wait for event with simple debounce
                if let Some(res) = rx.recv().await {
                    match res {
                        Ok(event) => {
                            info!("File change detected: {:?}", event.paths);
                            
                            // Debounce
                            tokio::time::sleep(Duration::from_secs(2)).await;
                            // Drain other events that happened during sleep
                            while rx.try_recv().is_ok() {}

                            // Run update logic
                            info!("Triggering dashboard update...");
                            
                            // We run the binary we just built
                            // Assuming create_dashboard.exe is in the same dir as superset-launcher.exe (root)
                            // OR in target/release if dev.
                            // In portable release, it's in root.
                            // In dev, we might need to look in target/release.
                            
                            let exe_name = if cfg!(windows) { "create_dashboard.exe" } else { "create_dashboard" };
                            let mut exe_path = root.join(exe_name);
                            
                            if !exe_path.exists() {
                                // Try target/release for dev mode
                                exe_path = root.join("target").join("release").join(exe_name);
                            }

                            if exe_path.exists() {
                                match tokio::process::Command::new(&exe_path)
                                    .current_dir(&root)
                                    .output()
                                    .await 
                                {
                                    Ok(output) => {
                                        if output.status.success() {
                                            info!("Data updated successfully!");
                                        } else {
                                            error!("Data update failed: {}", String::from_utf8_lossy(&output.stderr));
                                        }
                                    },
                                    Err(e) => error!("Failed to execute updater: {}", e),
                                }
                            } else {
                                error!("Updater binary not found at {:?}", exe_path);
                            }
                        },
                        Err(e) => error!("Watch error: {}", e),
                    }
                }
            }
            
            info!("Watcher stopped");
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}
