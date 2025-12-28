//! 缓存抽象模块
//! 定义缓存操作的统一接口

use crate::config::*;
use crate::error::{AppError, Result};
use hashlink::LruCache as HashlinkLruCache;
use lru::LruCache;
use mini_moka::unsync::Cache as MokaCache;
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::time::Duration;

/// 操作类型枚举
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Read(usize),
    Write(usize, usize),
}

/// 缓存操作trait，统一接口
pub trait CacheOps {
    /// 获取缓存值
    fn get(&mut self, key: &usize) -> Option<&usize>;
    
    /// 插入键值对
    fn insert(&mut self, key: usize, value: usize);
    
    /// 获取缓存名称（用于日志）
    fn name(&self) -> &'static str;
}

impl CacheOps for HashlinkLruCache<usize, usize> {
    #[inline]
    fn get(&mut self, key: &usize) -> Option<&usize> {
        self.get(key)
    }
    
    #[inline]
    fn insert(&mut self, key: usize, value: usize) {
        self.insert(key, value);
    }
    
    #[inline]
    fn name(&self) -> &'static str {
        "Hashlink LRU"
    }
}

impl CacheOps for LruCache<usize, usize> {
    #[inline]
    fn get(&mut self, key: &usize) -> Option<&usize> {
        self.get(key)
    }
    
    #[inline]
    fn insert(&mut self, key: usize, value: usize) {
        self.push(key, value);
    }
    
    #[inline]
    fn name(&self) -> &'static str {
        "LRU"
    }
}

impl CacheOps for MokaCache<usize, usize> {
    #[inline]
    fn get(&mut self, key: &usize) -> Option<&usize> {
        self.get(key)
    }
    
    #[inline]
    fn insert(&mut self, key: usize, value: usize) {
        self.insert(key, value);
    }
    
    #[inline]
    fn name(&self) -> &'static str {
        "Mini-Moka Unsync"
    }
}

/// 优化的 Mini-Moka 缓存构建器
pub struct OptimizedMokaCacheBuilder;

impl OptimizedMokaCacheBuilder {
    /// 创建优化的 Mini-Moka 缓存
    pub fn build_optimized_cache() -> MokaCache<usize, usize> {
        MokaCache::builder()
            // 预分配初始容量，减少动态扩容开销
            .initial_capacity((CACHE_CAPACITY / 2) as usize)
            // 使用权重感知，基于实际内存大小
            .weigher(|_key, _value: &usize| -> u32 {
                // 每个 usize 条目的权重（内存字节数）
                std::mem::size_of::<(usize, usize)>() as u32
            })
            // 设置最大权重容量（字节）
            .max_capacity((CACHE_CAPACITY as u32 * std::mem::size_of::<(usize, usize)>() as u32) as u64)
            .build()
    }
    
    /// 创建带过期策略的缓存（适用于长时间运行的场景）
    pub fn build_cache_with_expiration() -> MokaCache<usize, usize> {
        MokaCache::builder()
            .initial_capacity((CACHE_CAPACITY / 2) as usize)
            .weigher(|_key, _value: &usize| -> u32 {
                std::mem::size_of::<(usize, usize)>() as u32
            })
            .max_capacity((CACHE_CAPACITY as u32 * std::mem::size_of::<(usize, usize)>() as u32) as u64)
            // 设置TTL和TTI以优化内存使用
            .time_to_live(Duration::from_secs(600)) // 10分钟TTL
            .time_to_idle(Duration::from_secs(120)) // 2分钟TTI
            .build()
    }
    
    /// 创建高性能缓存（仅使用基本优化）
    pub fn build_high_performance_cache() -> MokaCache<usize, usize> {
        MokaCache::builder()
            // 预分配初始容量，减少动态扩容开销
            .initial_capacity((CACHE_CAPACITY / 2) as usize)
            // 保持简单的条目计数，避免权重计算开销
            .max_capacity(CACHE_CAPACITY)
            .build()
    }
}

/// 模拟后端延迟
#[inline]
pub async fn simulate_backend_latency(rng: &mut SmallRng) {
    let delay_ns = rng.random_range(MIN_DELAY_US * 1000..=MAX_DELAY_US * 1000);
    compio::time::sleep(Duration::from_nanos(delay_ns)).await;
}

/// 工作负载生成器
pub struct WorkloadGenerator {
    rng: StdRng,
}

impl WorkloadGenerator {
    /// 创建新的工作负载生成器
    pub fn new(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    /// 生成工作负载操作序列
    pub fn generate(&mut self) -> Result<Vec<Op>> {
        let zipf = rand_distr::Zipf::new(TOTAL_KEYS as f64, ZIPF_S)
            .map_err(|e| AppError::ZipfCreate(e.to_string()))?;
        
        let mut ops = Vec::with_capacity(WORKLOAD_SIZE);
        
        for _ in 0..WORKLOAD_SIZE {
            let key = zipf.sample(&mut self.rng) as usize;
            let is_read = self.rng.random::<f64>() < READ_RATIO;
            
            if is_read {
                ops.push(Op::Read(key));
            } else {
                let value = self.rng.random::<u32>() as usize;
                ops.push(Op::Write(key, value));
            }
        }
        
        Ok(ops)
    }
}

/// 预热管理器
pub struct WarmupManager {
    rng: StdRng,
}

impl Default for WarmupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WarmupManager {
    /// 创建新的预热管理器
    pub fn new() -> Self {
        Self {
            rng: StdRng::seed_from_u64(bench::WARMUP_SEED),
        }
    }
    
    /// 生成预热操作序列
    pub fn generate_warmup_ops(&mut self) -> Result<Vec<Op>> {
        let zipf = rand_distr::Zipf::new((CACHE_CAPACITY * 2) as f64, ZIPF_S)
            .map_err(|e| AppError::ZipfCreate(e.to_string()))?;
        
        let mut ops = Vec::with_capacity(bench::WARMUP_SIZE as usize);
        
        for _ in 0..bench::WARMUP_SIZE {
            let key = zipf.sample(&mut self.rng) as usize;
            let value = self.rng.random::<u32>() as usize;
            ops.push(Op::Write(key, value));
            
            // 偶尔加入读操作
            if self.rng.random::<f64>() < 0.2 {
                ops.push(Op::Read(key));
            }
        }
        
        Ok(ops)
    }
    
    /// 执行缓存预热
    pub async fn warmup_cache<C: CacheOps>(
        &mut self,
        cache: &mut C,
        warmup_ops: &[Op],
    ) -> Result<()> {
        for op in warmup_ops {
            match op {
                Op::Read(key) => {
                    cache.get(key);
                    // 模拟读取后的访问模式
                    if *key % 10 == 0 {
                        cache.insert(*key + 1000, *key + 1000);
                    }
                }
                Op::Write(key, val) => {
                    cache.insert(*key, *val);
                }
            }
        }
        Ok(())
    }
}

/// 通用缓存运行器
pub struct CacheRunner;

impl CacheRunner {
    /// 运行缓存测试
    pub async fn run_cache<C: CacheOps>(
        mut cache: C,
        ops: &[Op],
    ) -> Result<(u64, u64)> {
        let mut backend_rng = SmallRng::from_seed(rand::random());
        let mut hits = 0u64;
        let mut misses = 0u64;
        
        for op in ops {
            match op {
                Op::Read(key) => {
                    if cache.get(key).is_some() {
                        hits += 1;
                    } else {
                        misses += 1;
                        simulate_backend_latency(&mut backend_rng).await;
                        cache.insert(*key, *key);
                    }
                }
                Op::Write(key, val) => {
                    simulate_backend_latency(&mut backend_rng).await;
                    cache.insert(*key, *val);
                }
            }
        }
        
        Ok((hits, misses))
    }
    
    /// 计算命中率
    #[inline]
    pub fn calculate_hit_rate(hits: u64, misses: u64) -> f64 {
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64 * 100.0
        }
    }
}