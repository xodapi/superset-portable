//! Environment validation module
//! 
//! Validates the portable Superset environment before startup.

use anyhow::Result;
use std::net::TcpListener;
use std::path::Path;
use tracing::info;

use crate::python::PythonEnv;

/// Validation result for a single check
#[derive(Debug)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

impl CheckResult {
    fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            message: message.to_string(),
        }
    }
    
    fn fail(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            message: message.to_string(),
        }
    }
}

/// Environment validator
pub struct Validator {
    root: std::path::PathBuf,
}

impl Validator {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }
    
    /// Run all validation checks
    pub fn validate_all(&self) -> Vec<CheckResult> {
        let mut results = Vec::new();
        
        results.push(self.check_python());
        results.push(self.check_superset_installed());
        results.push(self.check_database());
        results.push(self.check_config());
        results.push(self.check_port(8088, "Superset"));
        results.push(self.check_port(8089, "Docs"));
        
        results
    }
    
    /// Check if Python exists
    fn check_python(&self) -> CheckResult {
        let python_env = match PythonEnv::new(&self.root) {
            Ok(env) => env,
            Err(_) => return CheckResult::fail("Python", "Не удалось инициализировать"),
        };
        
        if python_env.is_valid() {
            CheckResult::pass("Python", &format!("Найден: {}", python_env.python_path().display()))
        } else {
            CheckResult::fail("Python", &format!("Не найден: {}", python_env.python_path().display()))
        }
    }
    
    /// Check if Superset is installed
    fn check_superset_installed(&self) -> CheckResult {
        let python_env = match PythonEnv::new(&self.root) {
            Ok(env) => env,
            Err(_) => return CheckResult::fail("Superset", "Python не найден"),
        };
        
        if python_env.is_superset_installed() {
            CheckResult::pass("Superset", "Установлен")
        } else {
            CheckResult::fail("Superset", "Не установлен. Запустите setup\\install_superset.bat")
        }
    }
    
    /// Check if database exists
    fn check_database(&self) -> CheckResult {
        let db_path = self.root.join("superset_home").join("superset.db");
        
        if db_path.exists() {
            let size = std::fs::metadata(&db_path)
                .map(|m| m.len())
                .unwrap_or(0);
            CheckResult::pass("База данных", &format!("Найдена ({:.1} MB)", size as f64 / 1_048_576.0))
        } else {
            CheckResult::fail("База данных", "Не найдена. Запустите инициализацию")
        }
    }
    
    /// Check if config exists
    fn check_config(&self) -> CheckResult {
        let config_path = self.root.join("superset_home").join("superset_config.py");
        
        if config_path.exists() {
            CheckResult::pass("Конфигурация", "superset_config.py найден")
        } else {
            CheckResult::fail("Конфигурация", "superset_config.py не найден")
        }
    }
    
    /// Check if port is available
    fn check_port(&self, port: u16, service: &str) -> CheckResult {
        match TcpListener::bind(format!("127.0.0.1:{}", port)) {
            Ok(_) => CheckResult::pass(
                &format!("Порт {} ({})", port, service),
                "Свободен"
            ),
            Err(_) => {
                // Port might be in use by our own service - check if it responds
                if is_port_responding(port) {
                    CheckResult::pass(
                        &format!("Порт {} ({})", port, service),
                        "Занят (сервис работает)"
                    )
                } else {
                    CheckResult::fail(
                        &format!("Порт {} ({})", port, service),
                        "Занят другим процессом"
                    )
                }
            }
        }
    }
}

/// Check if a port is responding to HTTP requests
fn is_port_responding(port: u16) -> bool {
    use std::net::TcpStream;
    use std::time::Duration;
    
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(500)
    ).is_ok()
}

/// Print validation results to console
pub fn print_validation_report(results: &[CheckResult]) {
    println!();
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║           Проверка окружения Superset                  ║");
    println!("╠════════════════════════════════════════════════════════╣");
    
    let mut passed = 0;
    let mut failed = 0;
    
    for result in results {
        let status = if result.passed {
            passed += 1;
            "✅"
        } else {
            failed += 1;
            "❌"
        };
        
        println!("║ {} {:<20} │ {:<30} ║", 
            status, 
            truncate(&result.name, 20),
            truncate(&result.message, 30)
        );
    }
    
    println!("╠════════════════════════════════════════════════════════╣");
    
    if failed == 0 {
        println!("║ ✅ Все проверки пройдены ({}/{})                        ║", passed, passed);
    } else {
        println!("║ ⚠️  Пройдено: {}, Ошибок: {}                             ║", passed, failed);
    }
    
    println!("╚════════════════════════════════════════════════════════╝");
    println!();
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}
