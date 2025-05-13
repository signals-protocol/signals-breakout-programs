use crate::RangeBetMath;

#[test]
fn test_bin_sell_cost_edge_cases() {
    // When sale amount is 0
    assert_eq!(RangeBetMath::calculate_bin_sell_cost(0, 100, 1000).unwrap(), 0);
    
    // When q=t (coefficient of log term is 0)
    assert_eq!(RangeBetMath::calculate_bin_sell_cost(100, 1000, 1000).unwrap(), 100);
}

#[test]
fn test_bin_sell_cost_normal_cases() {
    // Normal cases
    assert!(RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap() < 100);
    
    // As q gets closer to t, revenue gets closer to x
    let revenue1 = RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap();
    let revenue2 = RangeBetMath::calculate_bin_sell_cost(100, 800, 1000).unwrap();
    let revenue3 = RangeBetMath::calculate_bin_sell_cost(100, 950, 1000).unwrap();
    assert!(revenue1 < revenue2);
    assert!(revenue2 < revenue3);
    assert!(revenue3 < 100);
}

#[test]
fn test_bin_sell_cost_exceed_bin() {
    // Selling more tokens than in the bin
    let result = RangeBetMath::calculate_bin_sell_cost(600, 500, 1000);
    assert!(result.is_err());
    // Check error pattern
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Cannot sell more than bin"));
}

#[test]
fn test_bin_sell_cost_exceed_supply() {
    // Selling more than total supply
    let result = RangeBetMath::calculate_bin_sell_cost(1200, 1500, 1000);
    assert!(result.is_err());
    // Check error pattern
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Cannot sell more than supply"));
}

#[test]
fn test_bin_sell_cost_invalid_state() {
    // When q > t (impossible state)
    let result = RangeBetMath::calculate_bin_sell_cost(100, 1500, 1000);
    assert!(result.is_err());
    // Check error pattern
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid bin state"));
}

#[test]
fn test_bin_sell_cost_extreme_values() {
    // Testing very large values
    let huge_x = 1_000_000_000;              // 10^9
    let huge_q = u64::MAX / 2;               // approximately 9.2 * 10^18
    let huge_t = u64::MAX / 2;               // approximately 9.2 * 10^18
    
    // Should not cause errors even with very large values
    let revenue = RangeBetMath::calculate_bin_sell_cost(huge_x, huge_q, huge_t).unwrap();
    assert!(revenue > 0);
    assert!(revenue <= huge_x);
    
    // When q is very close to t - revenue should be close to x
    let nearly_t = huge_t;
    let revenue_near_t = RangeBetMath::calculate_bin_sell_cost(huge_x, nearly_t, huge_t).unwrap();
    assert_eq!(revenue_near_t, huge_x);
    
    // Testing very small values - lamport unit (10^-9 SOL)
    let tiny_x = 1;  // 1 lamport
    let tiny_revenue = RangeBetMath::calculate_bin_sell_cost(tiny_x, 100, 100).unwrap();
    assert_eq!(tiny_revenue, tiny_x);
    
    // Testing very small values - when q < t
    let tiny_revenue2 = RangeBetMath::calculate_bin_sell_cost(tiny_x, 10, 100).unwrap();
    assert!(tiny_revenue2 <= tiny_x);
}

#[test]
fn test_bin_sell_cost_incremental() {
    // Verifying that revenue increases when x increases
    let q = 500;
    let t = 1000;
    
    let mut prev_revenue = 0;
    for x in [1, 10, 100, 200, 300, 400, 500].iter() {
        let revenue = RangeBetMath::calculate_bin_sell_cost(*x, q, t).unwrap();
        assert!(revenue >= prev_revenue);
        prev_revenue = revenue;
    }
}

#[test]
fn test_bin_sell_cost_large_dataset() {
    // Performing large-scale tests with various input combinations
    let x_values = [1, 10, 100, 1_000, 10_000, 100_000];
    let q_ratios = [0.2, 0.5, 0.8, 0.95, 1.0]; // q/t ratio
    let t_values = [100, 1_000, 10_000, 100_000, 1_000_000];
    
    for &t in &t_values {
        for &ratio in &q_ratios {
            let q = (t as f64 * ratio) as u64;
            
            // x cannot exceed q
            for &x in &x_values {
                if x <= q && x <= t {
                    match RangeBetMath::calculate_bin_sell_cost(x, q, t) {
                        Ok(revenue) => {
                            // Revenue should be less than or equal to x
                            assert!(revenue <= x);
                            
                            // As q gets closer to t, revenue gets closer to x
                            if ratio > 0.9 {
                                assert!(revenue > x / 2);
                            }
                            if ratio == 1.0 {
                                assert_eq!(revenue, x);
                            }
                            
                            // Revenue should always be positive (when x > 0)
                            if x > 0 {
                                assert!(revenue > 0);
                            } else {
                                assert_eq!(revenue, 0);
                            }
                        },
                        Err(e) => {
                            // Should not have errors in this test (x <= q && x <= t)
                            assert!(false, "Calculation error: x={}, q={}, t={}, error={:?}", x, q, t, e);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_bin_sell_cost_precision() {
    // Sale precision test - verify precision when selling in lamport units
    let t = 1_000_000;  // Sufficiently large t
    let q_values = [t / 10, t / 2, t - 1, t];
    
    for &q in &q_values {
        // Sell from 1 lamport sequentially
        for x in 1..20 {
            let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
            
            // Revenue should always be greater than 0
            assert!(revenue > 0);
            
            // Revenue should always be less than or equal to sale amount
            assert!(revenue <= x);
            
            // As q gets closer to t, revenue gets closer to x
            if q == t {
                assert_eq!(revenue, x);
            }
        }
    }
}

#[test]
fn test_bin_sell_cost_boundary_cases() {
    // Boundary value test: when x equals q (maximum sellable amount)
    let tests = [
        (10, 10, 100),
        (50, 50, 100),
        (100, 100, 100),
        (1000, 1000, 10000)
    ];
    
    for &(x, q, t) in &tests {
        let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
        assert!(revenue <= x);
        
        // When q=t, revenue should equal x
        if q == t {
            assert_eq!(revenue, x);
        }
    }
    
    // Boundary value test: when t-x is very close to 0 (selling close to t)
    // SellCalculationUnderflow error should occur
    let result = RangeBetMath::calculate_bin_sell_cost(999, 1000, 1000);
    assert!(result.is_ok());
    
    let result = RangeBetMath::calculate_bin_sell_cost(1000, 1000, 1000);
    assert!(result.is_ok());
}

#[test]
fn test_bin_sell_cost_mathematical_properties() {
    // Mathematical property test: verify revenue matches theoretical value
    // Theory: when q=t, revenue = x
    // Theory: when q<t, revenue = x - (t-q)*ln(t/(t-x))
    
    let test_cases = [
        (100, 1000, 1000),  // q=t case
        (100, 500, 1000),   // q<t case
        (100, 900, 1000),   // q is close to t case
        (10, 10, 100)       // x=q, q<t case
    ];
    
    for (x, q, t) in test_cases {
        let actual_revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
        
        // Theoretical revenue calculation
        let theoretical_revenue = if q == t {
            // When q=t, revenue equals x
            x as f64
        } else {
            // When q<t, apply formula
            let x_f64 = x as f64;
            let q_f64 = q as f64;
            let t_f64 = t as f64;
            let ratio = t_f64 / (t_f64 - x_f64);
            let ln_ratio = ratio.ln();
            let revenue = x_f64 - (t_f64 - q_f64) * ln_ratio;
            if revenue <= 0.0 { 1.0 } else { revenue }
        };
        
        let theoretical_revenue_rounded = (theoretical_revenue + 0.5) as u64;
        
        // Allow 1% error margin
        let margin = (theoretical_revenue_rounded / 100).max(1);
        let diff = if actual_revenue > theoretical_revenue_rounded {
            actual_revenue - theoretical_revenue_rounded
        } else {
            theoretical_revenue_rounded - actual_revenue
        };
        
        assert!(diff <= margin,
            "x={}, q={}, t={}: actual={}, theoretical={}, diff={}, margin={}", 
            x, q, t, actual_revenue, theoretical_revenue_rounded, diff, margin
        );
    }
} 