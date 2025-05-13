use crate::RangeBetMath;

#[test]
fn test_bin_buy_cost_edge_cases() {
    // When purchase amount is 0
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(0, 100, 1000).unwrap(), 0);
    
    // First purchase (t=0) case
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(100, 0, 0).unwrap(), 100);
    
    // When q=t (coefficient of log term is 0)
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(100, 1000, 1000).unwrap(), 100);
}

#[test]
fn test_bin_buy_cost_normal_cases() {
    // Normal cases
    assert!(RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap() < 100);
    assert!(RangeBetMath::calculate_bin_buy_cost(100, 0, 1000).unwrap() < 100);
    
    // As q gets closer to t, cost gets closer to x
    let cost1 = RangeBetMath::calculate_bin_buy_cost(100, 0, 1000).unwrap();
    let cost2 = RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap();
    let cost3 = RangeBetMath::calculate_bin_buy_cost(100, 900, 1000).unwrap();
    assert!(cost1 < cost2);
    assert!(cost2 < cost3);
    assert!(cost3 < 100);
}

#[test]
fn test_bin_buy_cost_invalid_state() {
    // When q > t (impossible state)
    let result = RangeBetMath::calculate_bin_buy_cost(100, 1500, 1000);
    assert!(result.is_err());
}

#[test]
fn test_bin_buy_cost_extreme_values() {
    // Testing very large values
    let huge_x = 10_000_000_000;           // 10^10
    let huge_q = u64::MAX / 3;             // approximately 6.1 * 10^18
    let huge_t = u64::MAX / 2;             // approximately 9.2 * 10^18
    
    // Should not cause errors even with very large values
    let cost = RangeBetMath::calculate_bin_buy_cost(huge_x, huge_q, huge_t).unwrap();
    assert!(cost > 0 && cost <= huge_x);
    
    // When q is very close to t - cost should be close to x
    let nearly_t = huge_t - 1;
    let cost_near_t = RangeBetMath::calculate_bin_buy_cost(huge_x, nearly_t, huge_t).unwrap();
    assert!(cost_near_t > cost);
    assert!(cost_near_t <= huge_x);
    
    // Testing very small values - lamport unit (10^-9 SOL)
    let tiny_x = 1;  // 1 lamport
    let tiny_cost = RangeBetMath::calculate_bin_buy_cost(tiny_x, 10, 100).unwrap();
    assert!(tiny_cost > 0);
}

#[test]
fn test_bin_buy_cost_incremental() {
    // Verifying that cost increases when x increases
    let q = 500;
    let t = 1000;
    
    let mut prev_cost = 0;
    for x in [1, 10, 100, 1000, 10000].iter() {
        let cost = RangeBetMath::calculate_bin_buy_cost(*x, q, t).unwrap();
        assert!(cost > prev_cost);
        prev_cost = cost;
    }
}

#[test]
fn test_bin_buy_cost_large_dataset() {
    // Performing large-scale tests with various input combinations
    let x_values = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000];
    let q_ratios = [0.0, 0.1, 0.5, 0.9, 0.99, 1.0]; // q/t ratio
    let t_values = [100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];
    
    for &x in &x_values {
        for &t in &t_values {
            for &ratio in &q_ratios {
                let q = (t as f64 * ratio) as u64;
                
                match RangeBetMath::calculate_bin_buy_cost(x, q, t) {
                    Ok(cost) => {
                        // Cost should be less than or equal to x
                        assert!(cost <= x);
                        
                        // As q gets closer to t, cost gets closer to x
                        if ratio > 0.5 {
                            assert!(cost > x / 2);
                        }
                        if ratio == 1.0 {
                            assert_eq!(cost, x);
                        }
                        
                        // Cost should always be positive (when x > 0)
                        if x > 0 {
                            assert!(cost > 0);
                        } else {
                            assert_eq!(cost, 0);
                        }
                    },
                    Err(_) => {
                        // Error should only occur when q > t
                        // Should not happen in this test (ratio <= 1.0)
                        assert!(false, "Calculation error: x={}, q={}, t={}", x, q, t);
                    }
                }
            }
        }
    }
}

#[test]
fn test_bin_buy_cost_boundary_cases() {
    // Lamport precision test (1 lamport = 10^-9 SOL)
    let lamport_costs = [
        RangeBetMath::calculate_bin_buy_cost(1, 0, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 50, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 90, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 99, 100).unwrap()
    ];
    
    // Verify minimum value of 1 lamport
    for cost in lamport_costs {
        assert!(cost >= 1);
    }
    
    // Boundary value test: very large x, very small q/t ratio
    let result = RangeBetMath::calculate_bin_buy_cost(u64::MAX / 1_000, 1, u64::MAX / 100);
    assert!(result.is_ok());
}

#[test]
fn test_bin_buy_cost_mathematical_properties() {
    // Mathematical property test: verify cost matches theoretical value when q=0
    // Theory: when q=0, cost = x * (1 - ln(1 + x/t))
    
    let test_cases = [
        (100, 0, 1000),
        (200, 0, 1000),
        (1000, 0, 10000),
        (5000, 0, 10000)
    ];
    
    for (x, q, t) in test_cases {
        let actual_cost = RangeBetMath::calculate_bin_buy_cost(x, q, t).unwrap();
        
        // Theoretical cost calculation (applying formula, approximate comparison due to floating point precision)
        let x_f64 = x as f64;
        let t_f64 = t as f64;
        let theoretical_cost = x_f64 * (1.0 - (1.0 + x_f64/t_f64).ln() * t_f64 / x_f64);
        let theoretical_cost_rounded = (theoretical_cost + 0.5) as u64;
        
        // Check if actual_cost is within 1% error margin of theoretical_cost_rounded
        let margin = (theoretical_cost_rounded / 100).max(1);
        let diff = if actual_cost > theoretical_cost_rounded {
            actual_cost - theoretical_cost_rounded
        } else {
            theoretical_cost_rounded - actual_cost
        };
        
        assert!(diff <= margin, 
            "x={}, q={}, t={}: actual={}, theoretical={}, diff={}, margin={}", 
            x, q, t, actual_cost, theoretical_cost_rounded, diff, margin
        );
    }
} 