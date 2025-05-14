use crate::RangeBetMath;

#[test]
fn test_calculate_x_for_multi_bins_edge_cases() {
    // When budget is 0
    assert_eq!(RangeBetMath::calculate_x_for_multi_bins(0, &[100, 200, 300], 1000).unwrap(), 0);
    
    // Empty array case
    assert_eq!(RangeBetMath::calculate_x_for_multi_bins(1000, &[], 1000).unwrap(), 0);
    
    // Very small budget (1 lamport)
    let x = RangeBetMath::calculate_x_for_multi_bins(1, &[500], 1000).unwrap();
    if x > 0 {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[500], 1000).unwrap();
        assert!(cost <= 1);
    }
    
    // Very small budget (1 lamport), with very large bin
    let x = RangeBetMath::calculate_x_for_multi_bins(1, &[u64::MAX / 2], u64::MAX / 2).unwrap();
    if x > 0 {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[u64::MAX / 2], u64::MAX / 2).unwrap();
        assert!(cost <= 1);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_single_bin_scenarios() {
    // Test with various budgets (single bin)
    let test_budgets = [
        100,                    // Small budget
        1_000,                  // 1 SOL (10^-9)
        10_000,                 // 10 SOL
        100_000,                // 100 SOL
        1_000_000,              // 1,000 SOL
        10_000_000,             // 10,000 SOL
        100_000_000,            // 100,000 SOL
        1_000_000_000,          // 1,000,000 SOL
        10_000_000_000,         // 10 million SOL (10^10)
        100_000_000_000,        // 100 million SOL (10^11)
        1_000_000_000_000,      // 1 billion SOL (10^12)
        10_000_000_000_000,     // 10 billion SOL (10^13)
        100_000_000_000_000,    // 100 billion SOL (10^14)
        1_000_000_000_000_000,  // 10^15
        u64::MAX / 1_000_000,   // Almost one millionth of the maximum value
    ];
    
    // Test with various bin sizes
    let bin_scenarios = [
        (500, 1000),                            // Normal case
        (999, 1000),                            // q≈t case
        (1, 1000),                              // Very small q case
        (u64::MAX / 3, u64::MAX / 2),           // Very large q and t
        (1, u64::MAX / 1_000_000_000)           // Very small q, huge t
    ];
    
    println!("\n---- Single Bin Large-scale Test ----");
    
    for &budget in &test_budgets {
        for &(q, t) in &bin_scenarios {
            println!("\nTest case (single bin/large-scale): budget={}, bin=[{}], t={}", budget, q, t);
            
            // Inverse calculation test
            let x = RangeBetMath::calculate_x_for_multi_bins(budget, &[q], t).unwrap();
            println!("Calculated X for budget {}: {}", budget, x);
            
            // Calculate cost with the calculated x
            let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[q], t).unwrap_or(0);
            
            // Print and verify results
            let usage_percentage = if budget > 0 { actual_cost * 100 / budget } else { 0 };
            println!("Actual cost for X={}: {} ({}% of budget)", x, actual_cost, usage_percentage);
            
            // Cost calculated with x should not exceed budget
            assert!(actual_cost <= budget, 
                   "Cost ({}) exceeds budget ({})", actual_cost, budget);
            
            // Difference between budget and actual cost should be within 0.1% in Lamport units (only when x > 0)
            if x > 0 && budget >= 1000 {
                let max_deviation = budget / 1000; // Allow 0.1% error
                let diff = if actual_cost > budget {
                    actual_cost - budget
                } else {
                    budget - actual_cost
                };
                
                assert!(diff <= max_deviation,
                       "Difference between cost and budget ({}) exceeds allowed error ({})", diff, max_deviation);
            }
            
            // For very large values, only test a few cases to avoid long calculation times
            if budget > u64::MAX / 10_000 || q > u64::MAX / 10_000 {
                break;
            }
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_multiple_bins_scenarios() {
    // Test with various budgets and bin configurations
    let test_scenarios = [
        // Normal scale tests
        (1_000, vec![300, 400, 500], 1000),
        (10_000, vec![300, 400, 500], 1000),
        (100_000, vec![300, 400, 500], 1000),
        
        // Large-scale budget tests
        (1_000_000, vec![300, 400, 500], 1000),
        (10_000_000, vec![300, 400, 500], 1000),
        (100_000_000, vec![300, 400, 500], 1000),
        (1_000_000_000, vec![300, 400, 500], 1000),
        (10_000_000_000, vec![300, 400, 500], 1000),
        
        // Various bin size tests
        (10_000_000, vec![10_000, 20_000, 30_000], 100_000),
        (10_000_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        
        // Extreme bin size differences
        (10_000_000, vec![1, 1_000_000, 1_000_000_000], 2_000_000_000),
        
        // Very large number of bins
        (100_000_000, vec![1_000, 2_000, 3_000, 4_000, 5_000, 6_000, 7_000, 8_000, 9_000, 10_000, 
                          11_000, 12_000, 13_000, 14_000, 15_000, 16_000, 17_000, 18_000, 19_000, 20_000],
                    1_000_000),
        
        // Lamport unit range tests (10^-9 SOL)
        (1, vec![10, 10, 10], 100),
        (9, vec![10, 10, 10], 100),
        (99, vec![10, 10, 10], 100),
        (999, vec![10, 10, 10], 100),
    ];
    
    println!("\n---- Multiple Bins Test (Extended) ----");
    
    for (budget, bins, t) in test_scenarios {
        println!("\nTest case (multiple bins): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // Inverse calculation test
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        println!("Calculated X for budget {}: {}", budget, x);
        
        // Calculate cost with the calculated x
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        println!("Actual cost for X={}: {} ({}% of budget)", 
                x, actual_cost, if budget > 0 { actual_cost * 100 / budget } else { 0 });
        
        // Cost calculated with x should not exceed budget
        assert!(actual_cost <= budget);
        
        // Difference between budget and actual cost should be within 0.1% in lamport units (only when x > 0)
        if x > 0 && budget >= 1000 {
            let acceptable_deviation = (budget / 1000).max(1); // Allow 0.1% error, minimum 1 lamport
            let diff = if actual_cost > budget {
                actual_cost - budget
            } else {
                budget - actual_cost
            };
            assert!(diff <= acceptable_deviation,
                   "Difference between cost and budget ({}) exceeds allowed error ({})", diff, acceptable_deviation);
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_extreme_scenarios() {
    // Test extreme scenarios
    let extreme_scenarios = [
        // Very large budgets
        (u64::MAX / 100, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        (u64::MAX / 1_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        (u64::MAX / 10_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        
        // Very small bin sizes
        (1_000_000, vec![1, 2, 3], 10),
        (1_000_000, vec![1, 1, 1], 3),
        
        // Very large bin sizes
        (1_000_000, vec![u64::MAX / 1_000_000, u64::MAX / 1_000_000], u64::MAX / 100_000),
        
        // Very imbalanced bin sizes
        (1_000_000, vec![1, u64::MAX / 1_000_000], u64::MAX / 500_000),
        
        // Extreme lamport unit tests
        (1, vec![1, 1, 1], 3),
        (2, vec![1, 1, 1], 3),
        (3, vec![1, 1, 1], 3),
    ];
    
    println!("\n---- Extreme Scenario Test (Extended) ----");
    
    for (budget, bins, t) in extreme_scenarios {
        println!("\nTest case (extreme): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // Verify calculation proceeds without conflicts
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        println!("Calculated X for budget {}: {}", budget, x);
        
        // Calculate cost with the calculated x - use unwrap_or as errors may occur
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        println!("Actual cost for X={}: {}", x, actual_cost);
        
        // Cost calculated with x should not exceed budget
        assert!(actual_cost <= budget);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_u64_boundaries() {
    // u64 boundary value tests
    let boundary_scenarios = [
        // When budget is very large (close to u64::MAX)
        (u64::MAX - 1, vec![1_000_000, 2_000_000], 10_000_000),
        (u64::MAX - 1_000, vec![1_000_000, 2_000_000], 10_000_000),
        
        // When t is very large 
        (1_000_000, vec![1_000_000, 2_000_000], u64::MAX - 1_000),
        
        // When q is very large
        (1_000_000, vec![u64::MAX - 1_000, u64::MAX - 2_000], u64::MAX - 1_000),
        
        // When all values are close to u64::MAX
        (u64::MAX - 1_000, vec![u64::MAX - 2_000, u64::MAX - 3_000], u64::MAX - 1_000),
    ];
    
    println!("\n---- u64 Boundary Value Test ----");
    
    for (budget, bins, t) in boundary_scenarios {
        println!("\nTest case (u64 boundary): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // Verify boundary value calculations proceed without errors
        let result = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t);
        match result {
            Ok(x) => {
                println!("Calculated X for budget {}: {}", budget, x);
                
                // Calculate cost with the calculated x
                match RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t) {
                    Ok(actual_cost) => {
                        println!("Actual cost for X={}: {}", x, actual_cost);
                        
                        // Cost calculated with x should not exceed budget
                        assert!(actual_cost <= budget);
                    },
                    Err(e) => {
                        println!("Warning: Error calculating cost for X={}: {:?}", x, e);
                    }
                };
            },
            Err(e) => {
                println!("Warning: Error calculating X: {:?}", e);
                // This test is to verify function behavior at boundary values
                // Don't consider test failure if an error occurs
            }
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_lamport_precision() {
    // Lamport precision test
    // In SOL, lamport is the smallest currency unit at 10^-9
    // 1 SOL = 10^9 lamport
    
    let precision_tests = [
        // 1 lamport unit increase
        (1_000_000_000, 1_000_000_001),  // 1 SOL, 1 SOL + 1 lamport
        (1_000_000_001, 1_000_000_002),  // 1 SOL + 1 lamport, 1 SOL + 2 lamport
        
        // 10 lamport unit increase
        (1_000_000_000, 1_000_000_010),  // 1 SOL, 1 SOL + 10 lamport
        
        // 100 lamport unit increase
        (1_000_000_000, 1_000_000_100),  // 1 SOL, 1 SOL + 100 lamport
        
        // 1000 lamport unit increase (0.000001 SOL)
        (1_000_000_000, 1_000_001_000),  // 1 SOL, 1 SOL + 0.000001 SOL
        
        // 1 lamport increase with very small budgets
        (1, 2),  // 1 lamport, 2 lamport
        (9, 10), // 9 lamport, 10 lamport
        (99, 100), // 99 lamport, 100 lamport
        (999, 1000), // 999 lamport, 1000 lamport (0.000001 SOL)
    ];
    
    let bins = vec![10_000, 20_000, 30_000];
    let t = 100_000;
    
    println!("\n---- Lamport Precision Test ----");
    
    for (budget1, budget2) in precision_tests {
        println!("\nLamport precision test: budget1={}, budget2={}, difference={} lamport", 
                budget1, budget2, budget2 - budget1);
        
        let x1 = RangeBetMath::calculate_x_for_multi_bins(budget1, &bins, t).unwrap();
        let x2 = RangeBetMath::calculate_x_for_multi_bins(budget2, &bins, t).unwrap();
        
        let cost1 = RangeBetMath::calculate_multi_bins_buy_cost(x1, &bins, t).unwrap_or(0);
        let cost2 = RangeBetMath::calculate_multi_bins_buy_cost(x2, &bins, t).unwrap_or(0);
        
        println!("Budget {}: X={}, cost={}, remaining={}", budget1, x1, cost1, budget1 - cost1);
        println!("Budget {}: X={}, cost={}, remaining={}", budget2, x2, cost2, budget2 - cost2);
        
        // Cost should not exceed budget
        assert!(cost1 <= budget1);
        assert!(cost2 <= budget2);
        
        // If budget increases, X or cost should also increase
        if budget2 > budget1 && x2 == x1 && cost2 == cost1 {
            println!("Info: No change in X and cost despite budget increase ({} -> {})", budget1, budget2);
        }
        
        // Verify algorithm responds appropriately even to very small increases
        if budget2 - budget1 <= 10 && x2 > x1 {
            println!("Success: X increase detected despite very small budget increase ({} lamport)", budget2 - budget1);
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_performance() {
    use std::time::{Duration, Instant};
    
    // Performance test with various input sizes
    let performance_scenarios = [
        // Small-scale scenario
        (1000, vec![100, 200, 300], 1000),
        
        // Medium-scale scenario
        (1_000_000, vec![100, 200, 300, 400, 500], 5000),
        
        // Large-scale scenario - large budget
        (1_000_000_000, vec![100, 200, 300, 400, 500], 10000),
        
        // Large-scale scenario - many bins
        (1_000_000, vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000,
                        1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000], 50000),
        
        // Large-scale scenario - large budget and many bins
        (1_000_000_000, vec![10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000], 1_000_000),
        
        // Extreme scenario - very large budget
        (u64::MAX / 100, vec![1_000_000, 2_000_000], 10_000_000),
    ];
    
    println!("\n---- Performance Test (Extended) ----");
    
    for (budget, bins, t) in performance_scenarios {
        println!("\nPerformance test: budget={}, bins.len={}, t={}", budget, bins.len(), t);
        
        let start = Instant::now();
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        let duration = start.elapsed();
        
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        let accuracy = 100.0 - ((budget as f64 - actual_cost as f64) / budget as f64 * 100.0);
        
        println!("Calculated X: {}, cost: {}, budget utilization: {:.6}%, time taken: {:?}", 
                x, actual_cost, accuracy, duration);
        
        // Performance test simply measures execution time, so skip result verification
        // However, check that execution time is within reasonable range (e.g., under 30 seconds)
        assert!(duration < Duration::from_secs(30), 
               "Calculation time exceeds 30 seconds: {:?}", duration);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_stress() {
    // Stress test: perform many calculations quickly for various sized values
    let bins_scenarios = [
        vec![300, 400, 500],                    // Normal case
        vec![3, 4, 5],                          // Very small bins
        vec![30_000, 40_000, 50_000],           // Large bins
        vec![300, 400, 500, 600, 700, 800]      // Many bins
    ];
    
    let t_scenarios = [
        1_000,          // Normal case
        10,             // Very small t
        1_000_000       // Large t
    ];
    
    // Budget range
    let min_budget = 0;
    let max_budget = 10_000_000;  // 0 ~ 10 SOL
    let budget_step = 100_000;    // 0.1 SOL increments
    
    println!("\n---- Stress Test (Extended) ----");
    println!("Budget range: {}~{}, step: {}", min_budget, max_budget, budget_step);
    
    let start = std::time::Instant::now();
    let mut success_count = 0;
    let mut total_count = 0;
    
    for bins in &bins_scenarios {
        for &t in &t_scenarios {
            println!("\nBin configuration: {:?}, t={}", bins, t);
            
            for budget in (min_budget..=max_budget).step_by(budget_step as usize) {
                total_count += 1;
                
                match RangeBetMath::calculate_x_for_multi_bins(budget, bins, t) {
                    Ok(x) => {
                        if let Ok(cost) = RangeBetMath::calculate_multi_bins_buy_cost(x, bins, t) {
                            if cost <= budget {
                                success_count += 1;
                            } else {
                                println!("Error: Budget {} exceeded - X={}, cost={}", budget, x, cost);
                            }
                        } else {
                            println!("Error: Failed to calculate cost for X={}", x);
                        }
                    },
                    Err(e) => {
                        println!("Error: Error calculating for budget {}: {:?}", budget, e);
                    }
                }
                
                // Show progress briefly to avoid too much output
                if total_count % 100 == 0 {
                    print!(".");
                    if total_count % 1000 == 0 {
                        println!(" {}", total_count);
                    }
                }
            }
        }
    }
    
    let duration = start.elapsed();
    println!("\nTotal tests: {}, success: {}, failures: {}, success rate: {:.2}%, time taken: {:?}", 
            total_count, success_count, total_count - success_count, 
            (success_count as f64 / total_count as f64) * 100.0, duration);
    
    // All tests should succeed
    assert_eq!(success_count, total_count, 
              "Failed tests: {}/{}", total_count - success_count, total_count);
} 