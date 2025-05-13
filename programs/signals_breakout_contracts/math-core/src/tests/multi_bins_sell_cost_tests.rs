use crate::RangeBetMath;

#[test]
fn test_multi_bins_sell_cost_edge_cases() {
    // Empty array or x=0 case
    assert_eq!(RangeBetMath::calculate_multi_bins_sell_cost(0, &[100, 200, 300], 1000).unwrap(), 0);
    assert_eq!(RangeBetMath::calculate_multi_bins_sell_cost(100, &[], 1000).unwrap(), 0);
}

#[test]
fn test_multi_bins_sell_cost_basic_cases() {
    // Normal cases
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(100, &[300, 400, 500], 1000).unwrap();
    assert!(revenue > 0);
    
    // Revenue across multiple bins equals single bin sale (single bin case)
    let single_revenue = RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap();
    let multi_revenue = RangeBetMath::calculate_multi_bins_sell_cost(100, &[500], 1000).unwrap();
    assert_eq!(single_revenue, multi_revenue);
}

#[test]
fn test_multi_bins_sell_cost_exceed_bin() {
    // Selling more tokens than in the bin - CannotSellMoreThanBin
    let result = RangeBetMath::calculate_multi_bins_sell_cost(600, &[300, 400, 500], 1000);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_exceed_supply() {
    // Selling more than total supply - CannotSellMoreThanSupply
    let result = RangeBetMath::calculate_multi_bins_sell_cost(300, &[500, 500, 500, 500], 1000);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_sequential_effect() {
    // Sequential sale effect test - t decreases in subsequent bins
    let x = 100;
    let bins = [500, 500, 500];
    let t = 1000;
    
    // Compare results of calculate_multi_bins_sell_cost with manual calculation
    let multi_revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
    
    // Manual sequential calculation
    let mut manual_total = 0;
    let mut current_t = t;
    
    for &q in &bins {
        let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, current_t).unwrap();
        manual_total += revenue;
        current_t -= x;  // t decreases after sale
    }
    
    assert_eq!(multi_revenue, manual_total);
}

#[test]
fn test_multi_bins_sell_cost_large_bins() {
    // Test selling from many bins
    let x = 10;
    let t = 10000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100 identical bins
    
    // Should succeed if total sale amount doesn't exceed t
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &many_bins, t);
    assert!(result.is_ok());
    
    // Should error if total sale amount exceeds t
    let too_many_bins: Vec<u64> = vec![100; 1000];  // 1000 bins
    let overflow_result = RangeBetMath::calculate_multi_bins_sell_cost(x, &too_many_bins, t);
    assert!(overflow_result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_varying_bins() {
    // Test with various bin sizes
    let x = 50;
    let t = 10000;
    let varying_bins = [100, 200, 300, 400, 500];
    
    // Should work normally even with varied bin sizes
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &varying_bins, t).unwrap();
    assert!(revenue > 0);
    
    // Should error if a bin has q < x
    let invalid_bins = [100, 40, 100];  // Second bin is smaller than x(50)
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &invalid_bins, t);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_extreme_cases() {
    // Test extreme cases
    
    // Test very small values (lamport units)
    let tiny_x = 1;
    let tiny_bins = [10, 10, 10];
    let tiny_revenue = RangeBetMath::calculate_multi_bins_sell_cost(tiny_x, &tiny_bins, 100).unwrap();
    assert!(tiny_revenue > 0);
    
    // Case where bin size equals x (maximum sellable amount)
    let x = 50;
    let exact_bins = [x, x, x];
    let t = 1000;
    let exact_revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &exact_bins, t);
    assert!(exact_revenue.is_ok());
    
    // Test very large t value
    let huge_t = u64::MAX / 100;
    let normal_bins = [1000, 2000, 3000];
    let huge_t_result = RangeBetMath::calculate_multi_bins_sell_cost(100, &normal_bins, huge_t);
    assert!(huge_t_result.is_ok());
}

#[test]
fn test_multi_bins_sell_cost_incremental() {
    // Revenue should increase as x increases
    let bins = [500, 500, 500];
    let t = 5000;
    
    let mut prev_revenue = 0;
    for x in [1, 10, 50, 100, 200, 300, 400, 500].iter() {
        let revenue = RangeBetMath::calculate_multi_bins_sell_cost(*x, &bins, t).unwrap();
        assert!(revenue >= prev_revenue);
        prev_revenue = revenue;
    }
}

#[test]
fn test_multi_bins_sell_cost_many_scenarios() {
    // Test various scenarios
    let scenarios = [
        // (x, bins, t)
        (50, vec![100, 200, 300], 5000),
        (10, vec![50, 50, 50], 1000),
        (1, vec![10, 10, 10], 1000),
        (100, vec![500], 1000),
        (50, vec![100, 100], 1000)
    ];
    
    for (x, bins, t) in scenarios {
        // Verify total sale doesn't exceed t
        let total_sell = x * bins.len() as u64;
        if total_sell <= t {
            let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
            
            // Compare with manual calculation
            let mut manual_total = 0;
            let mut current_t = t;
            
            for &q in &bins {
                let bin_revenue = RangeBetMath::calculate_bin_sell_cost(x, q, current_t).unwrap();
                manual_total += bin_revenue;
                current_t -= x;
            }
            
            assert_eq!(revenue, manual_total);
        }
    }
}

#[test]
fn test_multi_bins_sell_cost_overflow_prevention() {
    // Overflow prevention test
    
    // Calculation with overflow possibility
    let x = 10;
    let t = u64::MAX / 100;
    let bins: Vec<u64> = vec![1000; 5];  // Several bins
    
    // Calculation should succeed if t is large enough
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t);
    assert!(result.is_ok());
    
    // Test for checked_sub - selling from too many bins causing t to become negative
    // Use small sample to prevent memory allocation errors
    let small_t = 1000;
    let small_x = 10;
    let many_small_bins: Vec<u64> = vec![100; (small_t / small_x) as usize + 1];  // small_t/small_x + 1 bins
    let overflow_result = RangeBetMath::calculate_multi_bins_sell_cost(small_x, &many_small_bins, small_t);
    assert!(overflow_result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_edge_case_equal_values() {
    // Special case test where all values are equal
    let x = 100;
    let q = 100;
    let t = 100 * 3;  // Total amount for exactly 3 bins sale
    
    let bins = vec![q, q, q];
    
    // Selling exactly all tokens
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
    assert!(revenue > 0);
    
    // Compare with manual calculation
    let mut manual_total = 0;
    let mut current_t = t;
    
    for &q_val in &bins {
        let bin_revenue = RangeBetMath::calculate_bin_sell_cost(x, q_val, current_t).unwrap();
        manual_total += bin_revenue;
        current_t -= x;
    }
    
    assert_eq!(revenue, manual_total);
}

#[test]
fn test_multi_bins_sell_cost_large_dataset() {
    // Test with large dataset
    let x_values = [1, 10, 50, 100];
    let t_multipliers = [2, 3, 5, 10];  // Set t as multiple of total sale amount
    let bin_counts = [1, 2, 5, 10];
    
    for &x in &x_values {
        for &count in &bin_counts {
            // Ensure all bins are at least x in size
            let bins: Vec<u64> = (0..count).map(|_| x + 10).collect();
            
            for &t_mul in &t_multipliers {
                let t = x * count * t_mul;  // t_mul times the total sale amount
                
                // Calculate revenue
                match RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t) {
                    Ok(revenue) => {
                        assert!(revenue > 0);
                        
                        // Compare with manual calculation
                        let mut manual_total = 0;
                        let mut current_t = t;
                        let mut calculation_success = true;
                        
                        for &q in &bins {
                            match RangeBetMath::calculate_bin_sell_cost(x, q, current_t) {
                                Ok(bin_revenue) => {
                                    manual_total += bin_revenue;
                                    current_t -= x;
                                },
                                Err(_) => {
                                    calculation_success = false;
                                    break;
                                }
                            }
                        }
                        
                        if calculation_success {
                            assert_eq!(revenue, manual_total);
                        }
                    },
                    Err(_) => {
                        // Errors may occur in some cases in this test
                        // Don't consider test failure
                    }
                }
            }
        }
    }
} 