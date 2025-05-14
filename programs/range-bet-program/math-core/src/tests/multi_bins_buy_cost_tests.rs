use crate::RangeBetMath;

#[test]
fn test_multi_bins_buy_cost_edge_cases() {
    // Empty array or x=0 case
    assert_eq!(RangeBetMath::calculate_multi_bins_buy_cost(0, &[100, 200, 300], 1000).unwrap(), 0);
    assert_eq!(RangeBetMath::calculate_multi_bins_buy_cost(100, &[], 1000).unwrap(), 0);
}

#[test]
fn test_multi_bins_buy_cost_basic_cases() {
    // Normal cases
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(100, &[300, 400, 500], 1000).unwrap();
    assert!(cost > 0);
    
    // Cost across multiple bins compared to single bin purchase
    let single_cost = RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap();
    let multi_cost = RangeBetMath::calculate_multi_bins_buy_cost(100, &[500], 1000).unwrap();
    assert_eq!(single_cost, multi_cost);
    
    let multi_cost_higher = RangeBetMath::calculate_multi_bins_buy_cost(100, &[500, 500], 1000).unwrap();
    assert!(multi_cost_higher > multi_cost);
}

#[test]
fn test_multi_bins_buy_cost_sequential_effect() {
    // Sequential purchase effect test - t increases in subsequent bins
    let x = 100;
    let bins = [500, 500, 500];
    let t = 1000;
    
    // Compare results of calculate_multi_bins_buy_cost with manual calculation
    let multi_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap();
    
    // Manual sequential calculation
    let mut manual_total = 0;
    let mut current_t = t;
    
    for &q in &bins {
        let cost = RangeBetMath::calculate_bin_buy_cost(x, q, current_t).unwrap();
        manual_total += cost;
        current_t += x;  // t increases after purchase
    }
    
    assert_eq!(multi_cost, manual_total);
}

#[test]
fn test_multi_bins_buy_cost_large_bins() {
    // Test with many bins
    let x = 10;
    let t = 1000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100 identical bins
    
    // Calculation should succeed
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &many_bins, t).unwrap();
    assert!(cost > 0);
    
    // Overflow test
    let huge_x = u64::MAX / 1000;
    let few_bins = vec![100, 200, 300];
    
    // Check handling of large x values - test for overflow possibility
    let result = RangeBetMath::calculate_multi_bins_buy_cost(huge_x, &few_bins, t);
    match result {
        Ok(cost) => {
            assert!(cost > 0);
        },
        Err(_) => {
            // Overflow may occur - don't consider test failure
        }
    }
}

#[test]
fn test_multi_bins_buy_cost_varying_bins() {
    // Test with various bin sizes
    let x = 100;
    let t = 10000;  // Set t sufficiently large
    let varying_bins = [10, 100, 1000, 5000];  // All bins should be smaller than t
    
    // Should work normally even with varied bin sizes
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &varying_bins, t).unwrap();
    assert!(cost > 0);
    
    // Check error when there's a bin with q > t
    let invalid_bins = [500, 15000, 500];  // Second bin is larger than t(10000)
    
    // No problem initially, but error should occur at the second bin
    let result = RangeBetMath::calculate_multi_bins_buy_cost(x, &invalid_bins, t);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_buy_cost_extreme_cases() {
    // Test extreme cases
    
    // Test very small values (lamport units)
    let tiny_x = 1;
    let tiny_bins = [1, 2, 3];
    let tiny_cost = RangeBetMath::calculate_multi_bins_buy_cost(tiny_x, &tiny_bins, 10).unwrap();
    assert!(tiny_cost > 0);
    
    // Case where one bin is very large and others are small
    let mixed_bins = [10, 10, u64::MAX / 100000000, 10];
    match RangeBetMath::calculate_multi_bins_buy_cost(100, &mixed_bins, 1000) {
        Ok(_) => {},
        Err(_) => {} // Error may occur due to extreme values
    }
    
    // Test very large t value
    let huge_t = u64::MAX / 100;
    let normal_bins = [1000, 2000, 3000];
    let huge_t_result = RangeBetMath::calculate_multi_bins_buy_cost(100, &normal_bins, huge_t);
    assert!(huge_t_result.is_ok());
}

#[test]
fn test_multi_bins_buy_cost_incremental() {
    // Cost should increase as x increases
    let bins = [300, 400, 500];
    let t = 1000;
    
    let mut prev_cost = 0;
    for x in [1, 10, 100, 1000].iter() {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(*x, &bins, t).unwrap();
        assert!(cost > prev_cost);
        prev_cost = cost;
    }
}

#[test]
fn test_multi_bins_buy_cost_many_scenarios() {
    // Test various scenarios
    let scenarios = [
        // (x, bins, t)
        (100, vec![100, 200, 300], 1000),
        (10, vec![500, 500, 500], 1000),
        (1, vec![10, 10, 10], 100),
        (1000, vec![500], 1000),
        (500, vec![1000, 1000], 2000)
    ];
    
    for (x, bins, t) in scenarios {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap();
        
        // Compare with manual calculation
        let mut manual_total = 0;
        let mut current_t = t;
        
        for &q in &bins {
            let bin_cost = RangeBetMath::calculate_bin_buy_cost(x, q, current_t).unwrap();
            manual_total += bin_cost;
            current_t += x;
        }
        
        assert_eq!(cost, manual_total);
    }
}

#[test]
fn test_multi_bins_buy_cost_overflow_prevention() {
    // Overflow prevention test
    
    // Many bins + large x value (overflow risk)
    let x = 1_000_000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100 bins
    let t = 1_000_000;
    
    // Calculation should succeed or be properly error-handled
    let result = RangeBetMath::calculate_multi_bins_buy_cost(x, &many_bins, t);
    match result {
        Ok(cost) => {
            assert!(cost > 0);
            
            // Compare with manual calculation (possible overflow)
            let mut manual_total: u64 = 0;
            let mut current_t = t;
            
            for &q in &many_bins {
                if let Ok(bin_cost) = RangeBetMath::calculate_bin_buy_cost(x, q, current_t) {
                    match manual_total.checked_add(bin_cost) {
                        Some(new_total) => {
                            manual_total = new_total;
                            if let Some(new_t) = current_t.checked_add(x) {
                                current_t = new_t;
                            } else {
                                // Overflow during t calculation
                                break;
                            }
                        },
                        None => {
                            // Overflow during cost summation
                            break;
                        }
                    }
                } else {
                    // Error during bin cost calculation
                    break;
                }
            }
            
            if manual_total > 0 {
                // If manual calculation succeeded, results should match
                assert_eq!(cost, manual_total);
            }
        },
        Err(_) => {
            // If error occurred - don't consider test failure
            // Overflow can happen in this scenario
        }
    }
}

#[test]
fn test_multi_bins_buy_cost_large_dataset() {
    // Test with large dataset
    let x_values = [1, 10, 100, 1000];
    let t_values = [100, 1000, 10000];
    let bin_counts = [1, 2, 5, 10];
    
    for &x in &x_values {
        for &t in &t_values {
            for &count in &bin_counts {
                // Generate various bin sizes
                let bins: Vec<u64> = (0..count).map(|i| (i + 1) * t / (count + 1)).collect();
                
                // Calculate cost
                match RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t) {
                    Ok(cost) => {
                        assert!(cost > 0 || x == 0 || bins.is_empty());
                        
                        // Compare with manual calculation
                        let mut manual_total = 0;
                        let mut current_t = t;
                        
                        for &q in &bins {
                            let bin_cost = RangeBetMath::calculate_bin_buy_cost(x, q, current_t).unwrap();
                            manual_total += bin_cost;
                            current_t += x;
                        }
                        
                        assert_eq!(cost, manual_total);
                    },
                    Err(_) => {
                        // No errors should occur in this test
                        assert!(false, "Calculation error: x={}, bins.len={}, t={}", x, bins.len(), t);
                    }
                }
            }
        }
    }
} 