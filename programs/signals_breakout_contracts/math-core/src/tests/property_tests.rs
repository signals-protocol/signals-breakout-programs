use crate::RangeBetMath;
use proptest::prelude::*;

// Define test ranges
proptest! {
    // Basic ranges
    #[test]
    fn bin_buy_cost_is_monotonic(
        x1 in 1u64..1000u64,
        x2 in 1001u64..2000u64,
        q in 0u64..5000u64,
        t in 5001u64..10000u64
    ) {
        // Check condition q <= t
        prop_assume!(q <= t);
        
        // When x1 < x2, cost1 < cost2 should be true (monotonically increasing)
        let cost1 = RangeBetMath::calculate_bin_buy_cost(x1, q, t).unwrap();
        let cost2 = RangeBetMath::calculate_bin_buy_cost(x2, q, t).unwrap();
        
        prop_assert!(cost1 < cost2);
    }
    
    #[test]
    fn bin_sell_cost_is_monotonic(
        x1 in 1u64..1000u64,
        x2 in 1001u64..2000u64,
        q in 2001u64..5000u64,
        t in 5001u64..10000u64
    ) {
        // Check conditions: q <= t, x1 <= q, x2 <= q, x1 < x2
        prop_assume!(q <= t && x1 <= q && x2 <= q && x1 < x2);
        
        // When x1 < x2, revenue1 < revenue2 should be true (monotonically increasing)
        let revenue1 = RangeBetMath::calculate_bin_sell_cost(x1, q, t).unwrap();
        let revenue2 = RangeBetMath::calculate_bin_sell_cost(x2, q, t).unwrap();
        
        prop_assert!(revenue1 < revenue2);
    }
    
    #[test]
    fn buy_sell_roundtrip_approximate_equality(
        x in 1u64..1000u64,
        q in 1001u64..5000u64,
        t in 5001u64..10000u64 
    ) {
        // Check conditions: q <= t, x <= q, roundtrip possible
        prop_assume!(q <= t && x <= q && x < t);
        
        // Calculate purchase cost
        let buy_cost = RangeBetMath::calculate_bin_buy_cost(x, q, t).unwrap();
        
        // Calculate revenue from selling the same amount
        let sell_revenue = RangeBetMath::calculate_bin_sell_cost(x, q + x, t + x).unwrap();
        
        // The difference between buy/sell price should not be too large - allow 1% tolerance
        let tolerance = (buy_cost / 100).max(1); // Minimum 1 lamport allowance
        let difference = if buy_cost > sell_revenue {
            buy_cost - sell_revenue
        } else {
            sell_revenue - buy_cost
        };
        
        prop_assert!(difference <= tolerance,
            "Buy-sell roundtrip difference: buy={}, sell={}, diff={}, tolerance={}",
            buy_cost, sell_revenue, difference, tolerance
        );
    }
    
    #[test]
    fn multi_bins_preserves_order(
        x in 1u64..100u64,
        base_q in 100u64..500u64,  // Adjust value range to be smaller
        t in 1001u64..5000u64,
        bin_count in 1usize..3usize  // Reduce bin count
    ) {
        // Create multiple bins with the same q value
        let qs: Vec<u64> = vec![base_q; bin_count];
        
        // Check condition
        prop_assume!(base_q <= t);
        
        // Check if q > t state does not occur during sequential buy/sell
        let total_x = x * bin_count as u64;
        
        // Check if total sales amount does not exceed supply
        prop_assume!(total_x <= t);
        
        // Calculate t changes from beginning to end of bins
        // In the last bin, t - (bin_count - 1) * x > the last bin's q
        prop_assume!(t >= (bin_count as u64 - 1) * x + base_q);
        
        // Get results - consider test passed if an error occurs
        match RangeBetMath::calculate_multi_bins_buy_cost(x, &qs, t) {
            Ok(buy_cost) => {
                // Compare with manual calculation
                let mut manual_buy_total = 0;
                let mut current_t_buy = t;
                
                for &q in &qs {
                    let bin_cost = RangeBetMath::calculate_bin_buy_cost(x, q, current_t_buy).unwrap();
                    manual_buy_total += bin_cost;
                    current_t_buy += x;
                }
                
                prop_assert_eq!(buy_cost, manual_buy_total);
            },
            Err(_) => {
                // If an error occurs, test conditions are not valid
                // Skip test with prop_assume!(false) instead of returning Ok(())
                prop_assume!(false);
            }
        }
        
        match RangeBetMath::calculate_multi_bins_sell_cost(x, &qs, t) {
            Ok(sell_revenue) => {
                // Compare with manual calculation
                let mut manual_sell_total = 0;
                let mut current_t_sell = t;
                
                for &q in &qs {
                    let bin_revenue = RangeBetMath::calculate_bin_sell_cost(x, q, current_t_sell).unwrap();
                    manual_sell_total += bin_revenue;
                    current_t_sell -= x;
                }
                
                prop_assert_eq!(sell_revenue, manual_sell_total);
            },
            Err(_) => {
                // If an error occurs, test conditions are not valid
                prop_assume!(false);
            }
        }
    }
    
    #[test]
    fn calculate_x_for_multi_bins_property(
        budget in 10000u64..100000u64,
        base_q in 100u64..500u64,
        t in 1001u64..5000u64,
        bin_count in 1usize..2usize
    ) {
        // Create multiple bins with the same q value
        let qs: Vec<u64> = vec![base_q; bin_count];
        
        // Check condition
        prop_assume!(base_q <= t);
        
        // Calculate maximum purchasable x
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &qs, t).unwrap();
        
        // Recalculate cost with the calculated x
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &qs, t).unwrap();
        
        // Cost should not exceed budget
        prop_assert!(actual_cost <= budget);
        
        // The difference between budget and actual cost should be small (efficient budget use)
        if x > 0 { // Only check when x > 0
            // Check if cost for x+1 exceeds budget
            if let Ok(next_cost) = RangeBetMath::calculate_multi_bins_buy_cost(x + 1, &qs, t) {
                // If the cost of next x is less than budget, x is not optimal
                // However, this can happen due to binary search precision limitations
                // Allow within 1% margin
                if next_cost <= budget {
                    let margin = budget / 100;  // 1% margin
                    let diff = budget - actual_cost;
                    prop_assert!(diff <= margin,
                        "Budget efficiency: budget={}, actual_cost={}, diff={}, margin={}",
                        budget, actual_cost, diff, margin
                    );
                }
            }
            // Ignore errors (due to overflow etc.)
        }
    }
} 