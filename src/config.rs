//! 配置模块
//! 定义所有应用级别的常量和配置

/// 缓存容量配置
pub const CACHE_CAPACITY: u64 = 7500;

/// 总Key数量
pub const TOTAL_KEYS: usize = 10_000;

/// 每次测试的操作数量
pub const WORKLOAD_SIZE: usize = 1_000;

/// Zipf分布参数
pub const ZIPF_S: f64 = 1.6;

/// 读操作比例
pub const READ_RATIO: f64 = 0.95;

/// 后端延迟范围（微秒）
pub const MIN_DELAY_US: u64 = 1000;
pub const MAX_DELAY_US: u64 = 2000;

/// 基准测试配置
pub mod bench {
    use super::*;
    
    /// 采样数量
    pub const SAMPLE_SIZE: usize = 20;
    
    /// 测量时间（秒）
    pub const MEASUREMENT_TIME_SECS: u64 = 10;
    
    /// 预热操作数量
    pub const WARMUP_SIZE: u64 = CACHE_CAPACITY;
    
    /// 预热种子
    pub const WARMUP_SEED: u64 = 123;
    
    /// 工作负载种子
    pub const WORKLOAD_SEED: u64 = 42;
    
    /// 最小命中率目标（百分比）
    pub const MIN_HIT_RATE_TARGET: f64 = 85.0;
}

/// 错误消息常量
pub mod messages {
    pub const RUNTIME_CREATE_FAILED: &str = "Failed to create Compio runtime";
    pub const WORKLOAD_GEN_FAILED: &str = "Failed to generate workload";
    pub const WARMUP_FAILED: &str = "Warmup operation failed";
    pub const CACHE_OPERATION_FAILED: &str = "Cache operation failed";
    pub const ZIPF_CREATE_FAILED: &str = "Failed to create Zipf distribution";
}