use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use hashlink::LruCache as HashlinkLruCache;
use lru::LruCache;
use mini_moka::unsync::Cache as MokaCache;
use std::time::Duration;

// 导入项目模块
use cache_bench::cache::{CacheRunner, WorkloadGenerator, WarmupManager, Op};
use cache_bench::config::{CACHE_CAPACITY, WORKLOAD_SIZE, bench, messages};
use cache_bench::error::{AppError, ErrorContext};

pub struct CompioExecutor;

impl criterion::async_executor::AsyncExecutor for CompioExecutor {
    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> T {
        // 为每个 bench iteration 创建一个新的轻量级 runtime
        let runtime = compio::runtime::Runtime::new()
            .expect("Failed to create Compio runtime in executor");
        runtime.block_on(future)
    }
}

// 操作类型已在 cache.rs 中定义，直接使用

// 使用导入的函数和类型

// ----------------------------------------------------------------
// 各缓存实现
// ----------------------------------------------------------------
async fn run_hashlink(ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    let cache = HashlinkLruCache::new(CACHE_CAPACITY as usize);
    CacheRunner::run_cache(cache, ops).await
}

async fn run_lru(ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    let cache = LruCache::new(std::num::NonZeroUsize::new(CACHE_CAPACITY as usize).unwrap());
    CacheRunner::run_cache(cache, ops).await
}

async fn run_mini_moka(ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    let cache = cache_bench::cache::OptimizedMokaCacheBuilder::build_high_performance_cache();
    CacheRunner::run_cache(cache, ops).await
}

// ----------------------------------------------------------------
// 带预热的缓存函数（用于 benchmark）
// ----------------------------------------------------------------
async fn run_hashlink_with_cache(cache: HashlinkLruCache<usize, usize>, ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    CacheRunner::run_cache(cache, ops).await
}

async fn run_lru_with_cache(cache: LruCache<usize, usize>, ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    CacheRunner::run_cache(cache, ops).await
}

async fn run_mini_moka_with_cache(cache: MokaCache<usize, usize>, ops: &[Op]) -> std::result::Result<(u64, u64), AppError> {
    CacheRunner::run_cache(cache, ops).await
}

// ----------------------------------------------------------------
// Criterion Benchmark 设置
// ----------------------------------------------------------------
fn bench_caches(c: &mut Criterion) {
    // 生成工作负载和预热操作
    let mut workload_gen = WorkloadGenerator::new(bench::WORKLOAD_SEED);
    let ops = workload_gen.generate()
        .with_context(messages::WORKLOAD_GEN_FAILED)
        .expect("Failed to generate workload");
    
    let mut warmup_mgr = WarmupManager::new();
    let warmup_ops = warmup_mgr.generate_warmup_ops()
        .with_context(messages::WARMUP_FAILED)
        .expect("Failed to generate warmup ops");

    // 1. 验证并打印命中率 (只跑一次作为检查)
    let runtime = compio::runtime::Runtime::new()
        .map_err(|e| AppError::RuntimeCreate(e.to_string()))
        .expect(messages::RUNTIME_CREATE_FAILED);
    
    runtime.block_on(async {
        println!("=== Warmup & Calibration Check ===");
        let (hashlink_hits, hashlink_misses) = run_hashlink(&ops).await
            .with_context(messages::CACHE_OPERATION_FAILED)
            .expect("Failed to run hashlink");
        let hashlink_rate = CacheRunner::calculate_hit_rate(hashlink_hits, hashlink_misses);
        println!(
            "Hashlink Hit Rate: {:.2}% (Hits: {}, Misses: {})",
            hashlink_rate, hashlink_hits, hashlink_misses
        );

        let (lru_hits, lru_misses) = run_lru(&ops).await
            .with_context(messages::CACHE_OPERATION_FAILED)
            .expect("Failed to run lru");
        let lru_rate = CacheRunner::calculate_hit_rate(lru_hits, lru_misses);
        println!(
            "LRU Hit Rate: {:.2}% (Hits: {}, Misses: {})",
            lru_rate, lru_hits, lru_misses
        );

        let (moka_hits, moka_misses) = run_mini_moka(&ops).await
            .with_context(messages::CACHE_OPERATION_FAILED)
            .expect("Failed to run mini moka");
        let moka_rate = CacheRunner::calculate_hit_rate(moka_hits, moka_misses);
        println!(
            "Mini-Moka Hit Rate: {:.2}% (Hits: {}, Misses: {})",
            moka_rate, moka_hits, moka_misses
        );

        if hashlink_rate < bench::MIN_HIT_RATE_TARGET 
            || lru_rate < bench::MIN_HIT_RATE_TARGET 
            || moka_rate < bench::MIN_HIT_RATE_TARGET {
            println!("WARNING: Hit rate is below target. Adjust ZIPF_S or CACHE_CAPACITY.");
        }
        println!("==================================");
    });

    let mut group = c.benchmark_group("Single-Thread Cache + Compio Async IO");
    // 设置采样参数
    group.sample_size(bench::SAMPLE_SIZE);
    group.measurement_time(Duration::from_secs(bench::MEASUREMENT_TIME_SECS));
    group.throughput(Throughput::Elements(WORKLOAD_SIZE as u64));

    // 测试 Hashlink
    group.bench_function("hashlink_lru", |b| {
        b.iter_batched(
            || {
                let cache = HashlinkLruCache::new(CACHE_CAPACITY as usize);
                let runtime = compio::runtime::Runtime::new()
                    .expect(messages::RUNTIME_CREATE_FAILED);
                (cache, runtime)
            },
            |(mut cache, runtime)| {
                // 预热
                let warmup_result = runtime.block_on(
                    warmup_mgr.warmup_cache(&mut cache, &warmup_ops)
                );
                if let Err(e) = warmup_result {
                    eprintln!("Warning: Warmup failed for hashlink: {}", e);
                }
                // 运行基准测试
                runtime.block_on(run_hashlink_with_cache(cache, &ops))
                    .with_context(messages::CACHE_OPERATION_FAILED)
                    .expect("Benchmark failed")
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // 测试 LRU
    group.bench_function("lru", |b| {
        b.iter_batched(
            || {
                let cache = LruCache::new(std::num::NonZeroUsize::new(CACHE_CAPACITY as usize).unwrap());
                let runtime = compio::runtime::Runtime::new()
                    .expect(messages::RUNTIME_CREATE_FAILED);
                (cache, runtime)
            },
            |(mut cache, runtime)| {
                // 预热
                let warmup_result = runtime.block_on(
                    warmup_mgr.warmup_cache(&mut cache, &warmup_ops)
                );
                if let Err(e) = warmup_result {
                    eprintln!("Warning: Warmup failed for lru: {}", e);
                }
                // 运行基准测试
                runtime.block_on(run_lru_with_cache(cache, &ops))
                    .with_context(messages::CACHE_OPERATION_FAILED)
                    .expect("Benchmark failed")
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // 测试 Mini-Moka (高性能版本)
    group.bench_function("mini_moka_unsync_optimized", |b| {
        b.iter_batched(
            || {
                let cache = cache_bench::cache::OptimizedMokaCacheBuilder::build_high_performance_cache();
                let runtime = compio::runtime::Runtime::new()
                    .expect(messages::RUNTIME_CREATE_FAILED);
                (cache, runtime)
            },
            |(mut cache, runtime)| {
                // 预热
                let warmup_result = runtime.block_on(
                    warmup_mgr.warmup_cache(&mut cache, &warmup_ops)
                );
                if let Err(e) = warmup_result {
                    eprintln!("Warning: Warmup failed for mini moka optimized: {}", e);
                }
                // 运行基准测试
                runtime.block_on(run_mini_moka_with_cache(cache, &ops))
                    .with_context(messages::CACHE_OPERATION_FAILED)
                    .expect("Benchmark failed")
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // 测试 Mini-Moka (权重感知版本)
    group.bench_function("mini_moka_unsync_weigher", |b| {
        b.iter_batched(
            || {
                let cache = cache_bench::cache::OptimizedMokaCacheBuilder::build_optimized_cache();
                let runtime = compio::runtime::Runtime::new()
                    .expect(messages::RUNTIME_CREATE_FAILED);
                (cache, runtime)
            },
            |(mut cache, runtime)| {
                // 预热
                let warmup_result = runtime.block_on(
                    warmup_mgr.warmup_cache(&mut cache, &warmup_ops)
                );
                if let Err(e) = warmup_result {
                    eprintln!("Warning: Warmup failed for mini moka weigher: {}", e);
                }
                // 运行基准测试
                runtime.block_on(run_mini_moka_with_cache(cache, &ops))
                    .with_context(messages::CACHE_OPERATION_FAILED)
                    .expect("Benchmark failed")
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_caches);
criterion_main!(benches);
