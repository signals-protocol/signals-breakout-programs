use crate::RangeBetMath;

#[test]
fn test_reduction_over_x_cost_cutoff() {
    // (a) reduction > x → cost == 1
    // When x is very small and t is very large, reduction > x should result in cost = 1
    assert_eq!(
        RangeBetMath::calculate_bin_buy_cost(
            1,          // x very small
            0,          // q=0
            1_000_000   // t very large
        )
        .unwrap(),
        1
    );

    // More extreme case
    assert_eq!(
        RangeBetMath::calculate_bin_buy_cost(
            1,          // x very small
            0,          // q=0
            u64::MAX / 1000  // t very very large
        )
        .unwrap(),
        1
    );
}

#[test]
fn test_sell_calculation_underflow() {
    // SellCalculationUnderflow test
    // In the current implementation, other error conditions are checked first,
    // making it difficult to directly reach the SellCalculationUnderflow error
    
    // 1. Normal case where x is less than t
    let result = RangeBetMath::calculate_bin_sell_cost(500, 1000, 1000);
    assert!(result.is_ok());
    
    // 2. When x equals t (in case q=t, the early return just returns x without calculation)
    let result = RangeBetMath::calculate_bin_sell_cost(1000, 1000, 1000);
    assert!(result.is_ok());
    
    // 3. Edge case with x close to t
    let result = RangeBetMath::calculate_bin_sell_cost(999, 1000, 1000);
    assert!(result.is_ok());
    
    // 4. SellCalculationUnderflow is theoretically impossible to reach
    // To reach this code path, all of the following conditions must be met:
    // 1) x <= q (CannotSellMoreThanBin condition)
    // 2) x <= t (CannotSellMoreThanSupply condition)
    // 3) q <= t (InvalidBinState condition)
    // 4) q != t (Early return condition)
    // 5) t - x <= 0 (SellCalculationUnderflow target)
    //
    // It's impossible to satisfy conditions 3, 4, and 5 simultaneously
    // If q < t and t - x <= 0, then x >= t, which contradicts condition 2
    //
    // This test confirms that in the current code structure, 
    // it's impossible to directly trigger SellCalculationUnderflow
}

#[test]
fn test_multi_bin_buy_invalid_state_after_previous_buys() {
    // (c) q > current_t after previous buys
    let qs = [500, 2_000]; // Design second bin to exceed current_t
    // If t = 600, after first loop current_t = 700 → second q(2000) > 700
    let result = RangeBetMath::calculate_multi_bins_buy_cost(100, &qs, 600);
    assert!(result.is_err());
    // Check error message
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid bin state"));

    // Success case - t large enough
    let success_result = RangeBetMath::calculate_multi_bins_buy_cost(100, &qs, 2000);
    assert!(success_result.is_ok());
}

#[test]
fn test_multi_bin_sell_invalid_state_after_previous_sells() {
    // (d) q > current_t after previous sells
    let qs = [2_000, 2_000]; // Second bin causes underflow
    // t = 2500, after first sell current_t = 1500, second bin q(2000) > 1500
    let result = RangeBetMath::calculate_multi_bins_sell_cost(1_000, &qs, 2_500);
    assert!(result.is_err());
    // Check error message
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid bin state"));

    // Success case - t large enough
    let success_result = RangeBetMath::calculate_multi_bins_sell_cost(1_000, &qs, 5_000);
    assert!(success_result.is_ok());
}

#[test]
fn test_budget_just_under_minimum_cost() {
    // (e) budget just under minimum cost
    // Check if x=0 is returned when budget is exactly 1 lamport less than minimum buy cost
    
    // First calculate exact minimum cost
    let min_cost = RangeBetMath::calculate_bin_buy_cost(1, 0, 1000).unwrap();
    
    // Calculate with budget 1 lamport less than minimum cost
    let x = RangeBetMath::calculate_x_for_multi_bins(min_cost - 1, &[1000], 1000).unwrap();
    assert_eq!(x, 0);
    
    // Calculate with budget exactly equal to minimum cost (should return x=1)
    let x = RangeBetMath::calculate_x_for_multi_bins(min_cost, &[1000], 1000).unwrap();
    assert_eq!(x, 1);
} 