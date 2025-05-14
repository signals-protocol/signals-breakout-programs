use crate::RangeBetMath;
use std::time::{Duration, Instant};

const BENCH_ITERATIONS: usize = 100;

/// Simple benchmark function - calculates the average execution time and 95th percentile for a specific task.
fn benchmark<F>(name: &str, mut f: F)
where
    F: FnMut(),
{
    let mut durations = Vec::with_capacity(BENCH_ITERATIONS);
    
    // Warm-up runs
    for _ in 0..5 {
        f();
    }
    
    // Actual measurement
    for _ in 0..BENCH_ITERATIONS {
        let start = Instant::now();
        f();
        let elapsed = start.elapsed();
        durations.push(elapsed);
    }
    
    // Calculate results
    durations.sort();
    
    let total: Duration = durations.iter().sum();
    let avg = total / BENCH_ITERATIONS as u32;
    
    // 95th percentile
    let idx_95 = (BENCH_ITERATIONS as f64 * 0.95) as usize;
    let p95 = durations[idx_95];
    
    println!(
        "Benchmark {}: Average = {:?}, 95p = {:?}",
        name, avg, p95
    );
}

#[test]
fn bench_bin_buy_cost() {
    // Normal case
    benchmark("bin_buy_cost (normal)", || {
        let _ = RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap();
    });
    
    // Extreme case
    benchmark("bin_buy_cost (extreme)", || {
        let _ = RangeBetMath::calculate_bin_buy_cost(u64::MAX / 1_000_000, u64::MAX / 3, u64::MAX / 2).unwrap();
    });
}

#[test]
fn bench_bin_sell_cost() {
    // Normal case
    benchmark("bin_sell_cost (normal)", || {
        let _ = RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap();
    });
    
    // Extreme case (ensuring x < t to avoid errors)
    benchmark("bin_sell_cost (extreme)", || {
        let t = u64::MAX / 2;
        let x = t / 2;
        let q = t;
        let _ = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
    });
}

#[test]
fn bench_multi_bins_ops() {
    // Small bin list (5 bins)
    let small_bins: Vec<u64> = vec![100, 200, 300, 400, 500];
    
    // Medium bin list (20 bins)
    let medium_bins: Vec<u64> = (0..20).map(|i| (i + 1) * 100).collect();
    
    // Buy benchmark
    benchmark("multi_bins_buy_cost (5 bins)", || {
        let _ = RangeBetMath::calculate_multi_bins_buy_cost(100, &small_bins, 1000).unwrap();
    });
    
    benchmark("multi_bins_buy_cost (20 bins)", || {
        let _ = RangeBetMath::calculate_multi_bins_buy_cost(100, &medium_bins, 5000).unwrap();
    });
    
    // Sell benchmark
    benchmark("multi_bins_sell_cost (5 bins)", || {
        let _ = RangeBetMath::calculate_multi_bins_sell_cost(50, &small_bins, 1000).unwrap();
    });
    
    benchmark("multi_bins_sell_cost (20 bins)", || {
        let _ = RangeBetMath::calculate_multi_bins_sell_cost(50, &medium_bins, 5000).unwrap();
    });
}

#[test]
fn bench_x_for_multi_bins() {
    // Small bin list (5 bins)
    let small_bins: Vec<u64> = vec![100, 200, 300, 400, 500];
    
    // Medium bin list (20 bins)
    let medium_bins: Vec<u64> = (0..20).map(|i| (i + 1) * 100).collect();
    
    // Benchmark with various budgets
    benchmark("x_for_multi_bins (budget=1,000)", || {
        let _ = RangeBetMath::calculate_x_for_multi_bins(1_000, &small_bins, 5000).unwrap();
    });
    
    benchmark("x_for_multi_bins (budget=1,000,000)", || {
        let _ = RangeBetMath::calculate_x_for_multi_bins(1_000_000, &small_bins, 5000).unwrap();
    });
    
    benchmark("x_for_multi_bins (20 bins, budget=1,000,000)", || {
        let _ = RangeBetMath::calculate_x_for_multi_bins(1_000_000, &medium_bins, 5000).unwrap();
    });
} 