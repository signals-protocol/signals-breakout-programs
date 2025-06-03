use crate::RangeBetMath;

#[test]
fn test_budget_never_exceeded() {
    // 이 테스트는 calculate_x_for_multi_bins가 절대로 budget을 초과하는 x를 반환하지 않는지 확인합니다
    let test_cases = [
        (1000, vec![100, 200, 300], 1000),
        (5000, vec![500, 600, 700], 2000),
        (10000, vec![1000], 2000),
        (100000, vec![10000, 20000, 30000], 100000),
        (1000000, vec![100000, 200000], 500000),
        // 더 극단적인 케이스들
        (10_000_000_000u64, vec![50], 100),
        (1_000_000_000u64, vec![10, 20, 30], 100),
    ];
    
    println!("=== Budget Validation Test ===");
    
    for (budget, bins, t) in test_cases {
        println!("\nTesting: budget={}, bins={:?}, t={}", budget, bins, t);
        
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap();
        
        println!("  Result: x={}, actual_cost={}", x, actual_cost);
        
        // 가장 중요한 검증: actual_cost는 절대로 budget을 초과하면 안됨
        assert!(actual_cost <= budget, 
               "CRITICAL: actual_cost ({}) exceeds budget ({}) for x={}, bins={:?}, t={}", 
               actual_cost, budget, x, bins, t);
        
        // 효율성 검증: budget과 actual_cost의 차이가 너무 크지 않아야 함
        let unused_budget = budget - actual_cost;
        let efficiency = (actual_cost as f64 / budget as f64) * 100.0;
        println!("  Efficiency: {:.2}%, unused: {}", efficiency, unused_budget);
        
        // x+1로 계산했을 때 budget을 초과하는지 확인 (최적성 검증)
        if x > 0 && x < u64::MAX {
            match RangeBetMath::calculate_multi_bins_buy_cost(x + 1, &bins, t) {
                Ok(next_cost) => {
                    if next_cost <= budget {
                        let diff = budget - actual_cost;
                        println!("  WARNING: x+1 ({}) costs {} <= budget {}, current x costs {}, unused budget: {}", 
                               x + 1, next_cost, budget, actual_cost, diff);
                        // 1 lamport 차이는 허용 (반올림 오차)
                        assert!(diff <= 1, 
                               "Non-optimal: significant unused budget detected");
                    } else {
                        println!("  ✓ Optimal: x+1 would exceed budget ({} > {})", next_cost, budget);
                    }
                },
                Err(_) => {
                    println!("  ✓ Optimal: x+1 causes overflow/error");
                }
            }
        }
        
        println!("  ✓ PASSED: Budget constraint satisfied");
    }
    
    println!("\n=== All budget validation tests passed! ===");
}

#[test]
fn test_edge_case_budget_precision() {
    // 부동소수점 반올림으로 인한 미세한 오차 테스트
    println!("=== Edge Case Budget Precision Test ===");
    
    let precision_cases = [
        (999, vec![100], 1000),
        (1001, vec![100], 1000),
        (9999, vec![1000], 10000),
        (10001, vec![1000], 10000),
    ];
    
    for (budget, bins, t) in precision_cases {
        println!("\nPrecision test: budget={}, bins={:?}, t={}", budget, bins, t);
        
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap();
        
        println!("  x={}, actual_cost={}", x, actual_cost);
        
        // 절대로 budget을 초과하면 안됨
        assert!(actual_cost <= budget, 
               "Precision error: actual_cost ({}) > budget ({})", actual_cost, budget);
        
        println!("  ✓ PASSED");
    }
    
    println!("\n=== All precision tests passed! ===");
} 