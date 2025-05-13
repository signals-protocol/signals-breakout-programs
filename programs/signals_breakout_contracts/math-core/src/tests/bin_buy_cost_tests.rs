use crate::RangeBetMath;

#[test]
fn test_bin_buy_cost_edge_cases() {
    // 구매량이 0인 경우
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(0, 100, 1000).unwrap(), 0);
    
    // 최초 구매 (t=0)인 경우
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(100, 0, 0).unwrap(), 100);
    
    // q=t인 경우 (로그 항의 계수가 0)
    assert_eq!(RangeBetMath::calculate_bin_buy_cost(100, 1000, 1000).unwrap(), 100);
}

#[test]
fn test_bin_buy_cost_normal_cases() {
    // 일반적인 케이스
    assert!(RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap() < 100);
    assert!(RangeBetMath::calculate_bin_buy_cost(100, 0, 1000).unwrap() < 100);
    
    // q가 t에 가까워질수록 비용이 x에 가까워짐
    let cost1 = RangeBetMath::calculate_bin_buy_cost(100, 0, 1000).unwrap();
    let cost2 = RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap();
    let cost3 = RangeBetMath::calculate_bin_buy_cost(100, 900, 1000).unwrap();
    assert!(cost1 < cost2);
    assert!(cost2 < cost3);
    assert!(cost3 < 100);
}

#[test]
fn test_bin_buy_cost_invalid_state() {
    // q > t인 경우 (불가능한 상태)
    let result = RangeBetMath::calculate_bin_buy_cost(100, 1500, 1000);
    assert!(result.is_err());
}

#[test]
fn test_bin_buy_cost_extreme_values() {
    // 매우 큰 값 테스트
    let huge_x = 10_000_000_000;           // 10^10
    let huge_q = u64::MAX / 3;             // 약 6.1 * 10^18
    let huge_t = u64::MAX / 2;             // 약 9.2 * 10^18
    
    // 매우 큰 값으로 테스트해도 오류가 발생하지 않아야 함
    let cost = RangeBetMath::calculate_bin_buy_cost(huge_x, huge_q, huge_t).unwrap();
    assert!(cost > 0 && cost <= huge_x);
    
    // q가 거의 t에 가까운 경우 - 비용은 x에 가까워야 함
    let nearly_t = huge_t - 1;
    let cost_near_t = RangeBetMath::calculate_bin_buy_cost(huge_x, nearly_t, huge_t).unwrap();
    assert!(cost_near_t > cost);
    assert!(cost_near_t <= huge_x);
    
    // 매우 작은 값 테스트 - lamport 단위 (10^-9 SOL)
    let tiny_x = 1;  // 1 lamport
    let tiny_cost = RangeBetMath::calculate_bin_buy_cost(tiny_x, 10, 100).unwrap();
    assert!(tiny_cost > 0);
}

#[test]
fn test_bin_buy_cost_incremental() {
    // x 값이 증가할 때 비용도 증가해야 함을 확인
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
    // 다양한 입력 조합으로 대규모 테스트 수행
    let x_values = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000];
    let q_ratios = [0.0, 0.1, 0.5, 0.9, 0.99, 1.0]; // q/t 비율
    let t_values = [100, 1_000, 10_000, 100_000, 1_000_000, 10_000_000];
    
    for &x in &x_values {
        for &t in &t_values {
            for &ratio in &q_ratios {
                let q = (t as f64 * ratio) as u64;
                
                match RangeBetMath::calculate_bin_buy_cost(x, q, t) {
                    Ok(cost) => {
                        // 비용이 x 이하여야 함
                        assert!(cost <= x);
                        
                        // q가 t에 가까워질수록 비용은 x에 가까워짐
                        if ratio > 0.5 {
                            assert!(cost > x / 2);
                        }
                        if ratio == 1.0 {
                            assert_eq!(cost, x);
                        }
                        
                        // 비용은 항상 양수여야 함 (x > 0인 경우)
                        if x > 0 {
                            assert!(cost > 0);
                        } else {
                            assert_eq!(cost, 0);
                        }
                    },
                    Err(_) => {
                        // q > t인 경우에만 오류가 발생함
                        // 이 테스트에서는 발생하지 않아야 함 (ratio <= 1.0)
                        assert!(false, "계산 중 오류 발생: x={}, q={}, t={}", x, q, t);
                    }
                }
            }
        }
    }
}

#[test]
fn test_bin_buy_cost_boundary_cases() {
    // Lamport 정밀도 테스트 (1 lamport = 10^-9 SOL)
    let lamport_costs = [
        RangeBetMath::calculate_bin_buy_cost(1, 0, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 50, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 90, 100).unwrap(),
        RangeBetMath::calculate_bin_buy_cost(1, 99, 100).unwrap()
    ];
    
    // 최소값 1 lamport 처리 확인
    for cost in lamport_costs {
        assert!(cost >= 1);
    }
    
    // 경계값 테스트: 매우 큰 x, 매우 작은 q/t 비율
    let result = RangeBetMath::calculate_bin_buy_cost(u64::MAX / 1_000, 1, u64::MAX / 100);
    assert!(result.is_ok());
}

#[test]
fn test_bin_buy_cost_mathematical_properties() {
    // 수학적 특성 테스트: q=0인 경우 비용이 이론적으로 계산한 값과 일치하는지 확인
    // 이론: q=0인 경우, 비용 = x * (1 - ln(1 + x/t))
    
    let test_cases = [
        (100, 0, 1000),
        (200, 0, 1000),
        (1000, 0, 10000),
        (5000, 0, 10000)
    ];
    
    for (x, q, t) in test_cases {
        let actual_cost = RangeBetMath::calculate_bin_buy_cost(x, q, t).unwrap();
        
        // 이론적인 비용 계산 (이론식 적용, 부동소수점 정밀도 문제로 근사치 비교)
        let x_f64 = x as f64;
        let t_f64 = t as f64;
        let theoretical_cost = x_f64 * (1.0 - (1.0 + x_f64/t_f64).ln() * t_f64 / x_f64);
        let theoretical_cost_rounded = (theoretical_cost + 0.5) as u64;
        
        // actual_cost가 theoretical_cost_rounded와 1% 이내의 오차 범위에 있는지 확인
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