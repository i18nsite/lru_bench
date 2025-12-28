//! 缓存基准测试库
//! 
//! 这个项目提供了多种LRU缓存实现的性能基准测试，
//! 包括Hashlink、LRU和Mini-Moka三种实现。
//! 
//! 特性：
//! - 使用Compio异步运行时
//! - Zipf分布模拟真实访问模式
//! - 增强的预热策略
//! - 详细的性能报告

pub mod config;
pub mod error;
pub mod cache;