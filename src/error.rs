//! 错误处理模块
//! 定义了所有应用级别的错误类型

use std::fmt;

/// 应用主错误类型
#[derive(Debug, Clone)]
pub enum AppError {
    /// 运行时创建错误
    RuntimeCreate(String),
    /// Zipf分布创建错误
    ZipfCreate(String),
    /// 缓存操作错误
    CacheOperation(String),
    /// IO错误
    Io(String),
    /// 配置错误
    Config(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::RuntimeCreate(msg) => write!(f, "Runtime create error: {}", msg),
            AppError::ZipfCreate(msg) => write!(f, "Zipf distribution create error: {}", msg),
            AppError::CacheOperation(msg) => write!(f, "Cache operation error: {}", msg),
            AppError::Io(msg) => write!(f, "IO error: {}", msg),
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

/// 错误上下文扩展trait
pub trait ErrorContext<T> {
    /// 添加上下文信息
    fn with_context(self, context: &str) -> Result<T>;
}

impl<T> ErrorContext<T> for Result<T> {
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| match e {
            AppError::RuntimeCreate(msg) => AppError::RuntimeCreate(format!("{}: {}", context, msg)),
            AppError::ZipfCreate(msg) => AppError::ZipfCreate(format!("{}: {}", context, msg)),
            AppError::CacheOperation(msg) => AppError::CacheOperation(format!("{}: {}", context, msg)),
            AppError::Io(msg) => AppError::Io(format!("{}: {}", context, msg)),
            AppError::Config(msg) => AppError::Config(format!("{}: {}", context, msg)),
        })
    }
}