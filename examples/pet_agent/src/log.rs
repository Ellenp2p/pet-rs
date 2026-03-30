//! 日志模块
//!
//! 将日志写入文件，方便调试和监控。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// 日志系统
pub struct Logger {
    log_path: PathBuf,
}

impl Logger {
    /// 创建新的日志器
    pub fn new() -> Self {
        let log_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".pet_agent")
            .join("log.txt");
        Self { log_path }
    }

    /// 获取日志文件路径
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }

    /// 写入日志
    pub fn log(&self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] [{}] {}\n", timestamp, level, message);

        // 确保目录存在
        if let Some(parent) = self.log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // 写入文件
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(log_line.as_bytes());
        }
    }

    /// 信息日志
    pub fn info(&self, message: &str) {
        self.log("INFO", message);
    }

    /// 警告日志
    pub fn warn(&self, message: &str) {
        self.log("WARN", message);
    }

    /// 错误日志
    pub fn error(&self, message: &str) {
        self.log("ERROR", message);
    }

    /// 成功日志
    pub fn success(&self, message: &str) {
        self.log("OK", message);
    }

    /// 读取最近的日志
    pub fn tail(&self, lines: usize) -> String {
        if let Ok(content) = std::fs::read_to_string(&self.log_path) {
            let all_lines: Vec<&str> = content.lines().collect();
            let start = if all_lines.len() > lines {
                all_lines.len() - lines
            } else {
                0
            };
            all_lines[start..].join("\n")
        } else {
            "日志文件不存在".to_string()
        }
    }

    /// 清空日志
    pub fn clear(&self) {
        let _ = std::fs::write(&self.log_path, "");
    }
}

/// 全局日志器
static mut LOGGER: Option<Logger> = None;

/// 初始化日志系统
pub fn init() {
    unsafe {
        LOGGER = Some(Logger::new());
    }
}

/// 获取日志器
pub fn logger() -> &'static Logger {
    unsafe {
        if LOGGER.is_none() {
            LOGGER = Some(Logger::new());
        }
        LOGGER.as_ref().unwrap()
    }
}

/// 记录信息
pub fn info(message: &str) {
    logger().info(message);
}

/// 记录警告
pub fn warn(message: &str) {
    logger().warn(message);
}

/// 记录错误
pub fn error(message: &str) {
    logger().error(message);
}

/// 记录成功
pub fn success(message: &str) {
    logger().success(message);
}
