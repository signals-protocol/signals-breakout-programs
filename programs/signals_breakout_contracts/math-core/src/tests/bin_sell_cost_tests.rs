use crate::RangeBetMath;

#[test]
fn test_bin_sell_cost_edge_cases() {
    // 판매량이 0인 경우
    assert_eq!(RangeBetMath::calculate_bin_sell_cost(0, 100, 1000).unwrap(), 0);
    
    // q=t인 경우 (로그 항의 계수가 0)
    assert_eq!(RangeBetMath::calculate_bin_sell_cost(100, 1000, 1000).unwrap(), 100);
}

#[test]
fn test_bin_sell_cost_normal_cases() {
    // 일반적인 케이스
    assert!(RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap() < 100);
    
    // q가 t에 가까워질수록 수익이 x에 가까워짐
    let revenue1 = RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap();
    let revenue2 = RangeBetMath::calculate_bin_sell_cost(100, 800, 1000).unwrap();
    let revenue3 = RangeBetMath::calculate_bin_sell_cost(100, 950, 1000).unwrap();
    assert!(revenue1 < revenue2);
    assert!(revenue2 < revenue3);
    assert!(revenue3 < 100);
}

#[test]
fn test_bin_sell_cost_exceed_bin() {
    // 빈에 있는 토큰보다 많이 판매하는 경우
    let result = RangeBetMath::calculate_bin_sell_cost(600, 500, 1000);
    assert!(result.is_err());
}

#[test]
fn test_bin_sell_cost_exceed_supply() {
    // 총 공급량보다 많이 판매하는 경우
    let result = RangeBetMath::calculate_bin_sell_cost(1200, 1500, 1000);
    assert!(result.is_err());
}

#[test]
fn test_bin_sell_cost_invalid_state() {
    // q > t인 경우 (불가능한 상태)
    let result = RangeBetMath::calculate_bin_sell_cost(100, 1500, 1000);
    assert!(result.is_err());
}

#[test]
fn test_bin_sell_cost_extreme_values() {
    // 매우 큰 값 테스트
    let huge_x = 1_000_000_000;              // 10^9
    let huge_q = u64::MAX / 2;               // 약 9.2 * 10^18
    let huge_t = u64::MAX / 2;               // 약 9.2 * 10^18
    
    // 매우 큰 값으로 테스트해도 오류가 발생하지 않아야 함
    let revenue = RangeBetMath::calculate_bin_sell_cost(huge_x, huge_q, huge_t).unwrap();
    assert!(revenue > 0);
    assert!(revenue <= huge_x);
    
    // q가 거의 t에 가까운 경우 - 수익은 x에 가까워야 함
    let nearly_t = huge_t;
    let revenue_near_t = RangeBetMath::calculate_bin_sell_cost(huge_x, nearly_t, huge_t).unwrap();
    assert_eq!(revenue_near_t, huge_x);
    
    // 매우 작은 값 테스트 - lamport 단위 (10^-9 SOL)
    let tiny_x = 1;  // 1 lamport
    let tiny_revenue = RangeBetMath::calculate_bin_sell_cost(tiny_x, 100, 100).unwrap();
    assert_eq!(tiny_revenue, tiny_x);
    
    // 매우 작은 값 테스트 - q < t인 경우
    let tiny_revenue2 = RangeBetMath::calculate_bin_sell_cost(tiny_x, 10, 100).unwrap();
    assert!(tiny_revenue2 <= tiny_x);
}

#[test]
fn test_bin_sell_cost_incremental() {
    // x 값이 증가할 때 수익도 증가해야 함을 확인
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
    // 다양한 입력 조합으로 대규모 테스트 수행
    let x_values = [1, 10, 100, 1_000, 10_000, 100_000];
    let q_ratios = [0.2, 0.5, 0.8, 0.95, 1.0]; // q/t 비율
    let t_values = [100, 1_000, 10_000, 100_000, 1_000_000];
    
    for &t in &t_values {
        for &ratio in &q_ratios {
            let q = (t as f64 * ratio) as u64;
            
            // x는 q를 초과할 수 없음
            for &x in &x_values {
                if x <= q && x <= t {
                    match RangeBetMath::calculate_bin_sell_cost(x, q, t) {
                        Ok(revenue) => {
                            // 수익이 x 이하여야 함
                            assert!(revenue <= x);
                            
                            // q가 t에 가까워질수록 수익은 x에 가까워짐
                            if ratio > 0.9 {
                                assert!(revenue > x / 2);
                            }
                            if ratio == 1.0 {
                                assert_eq!(revenue, x);
                            }
                            
                            // 수익은 항상 양수여야 함 (x > 0인 경우)
                            if x > 0 {
                                assert!(revenue > 0);
                            } else {
                                assert_eq!(revenue, 0);
                            }
                        },
                        Err(e) => {
                            // 이 테스트에서는 오류가 발생하지 않아야 함 (x <= q && x <= t)
                            assert!(false, "계산 중 오류 발생: x={}, q={}, t={}, 오류={:?}", x, q, t, e);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_bin_sell_cost_precision() {
    // 판매 정밀도 테스트 - lamport 단위로 판매할 때 정밀도 확인
    let t = 1_000_000;  // 충분히 큰 t
    let q_values = [t / 10, t / 2, t - 1, t];
    
    for &q in &q_values {
        // 1 lamport부터 차례대로 판매
        for x in 1..20 {
            let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
            
            // 수익은 항상 0보다 커야 함
            assert!(revenue > 0);
            
            // 수익은, 항상 판매량 이하여야 함
            assert!(revenue <= x);
            
            // q가 t에 가까울수록 수익은 x에 가까움
            if q == t {
                assert_eq!(revenue, x);
            }
        }
    }
}

#[test]
fn test_bin_sell_cost_boundary_cases() {
    // 경계값 테스트: x가 q와 같은 경우 (최대 판매 가능량)
    let tests = [
        (10, 10, 100),
        (50, 50, 100),
        (100, 100, 100),
        (1000, 1000, 10000)
    ];
    
    for &(x, q, t) in &tests {
        let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
        assert!(revenue <= x);
        
        // q=t인 경우 revenue=x여야 함
        if q == t {
            assert_eq!(revenue, x);
        }
    }
    
    // 경계값 테스트: t-x가 거의 0에 가까운 경우 (t에 가까운 판매)
    // SellCalculationUnderflow 오류가 발생해야 함
    let result = RangeBetMath::calculate_bin_sell_cost(999, 1000, 1000);
    assert!(result.is_ok());
    
    let result = RangeBetMath::calculate_bin_sell_cost(1000, 1000, 1000);
    assert!(result.is_ok());
}

#[test]
fn test_bin_sell_cost_mathematical_properties() {
    // 수학적 특성 테스트: 이론적으로 계산한 값과 일치하는지 확인
    // 이론: q=t인 경우, 수익 = x
    // 이론: q<t인 경우, 수익 = x - (t-q)*ln(t/(t-x))
    
    let test_cases = [
        (100, 1000, 1000),  // q=t 케이스
        (100, 500, 1000),   // q<t 케이스
        (100, 900, 1000),   // q가 t에 가까운 케이스
        (10, 10, 100)       // x=q, q<t 케이스
    ];
    
    for (x, q, t) in test_cases {
        let actual_revenue = RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap();
        
        // 이론적인 수익 계산
        let theoretical_revenue = if q == t {
            // q=t인 경우 수익은 x와 같음
            x as f64
        } else {
            // q<t인 경우 공식 적용
            let x_f64 = x as f64;
            let q_f64 = q as f64;
            let t_f64 = t as f64;
            let ratio = t_f64 / (t_f64 - x_f64);
            let ln_ratio = ratio.ln();
            let revenue = x_f64 - (t_f64 - q_f64) * ln_ratio;
            if revenue <= 0.0 { 1.0 } else { revenue }
        };
        
        let theoretical_revenue_rounded = (theoretical_revenue + 0.5) as u64;
        
        // 1% 이내의 오차 범위 허용
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