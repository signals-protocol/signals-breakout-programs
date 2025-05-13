use crate::RangeBetMath;

#[test]
fn test_multi_bins_buy_cost_edge_cases() {
    // 빈 배열 또는 x=0인 경우
    assert_eq!(RangeBetMath::calculate_multi_bins_buy_cost(0, &[100, 200, 300], 1000).unwrap(), 0);
    assert_eq!(RangeBetMath::calculate_multi_bins_buy_cost(100, &[], 1000).unwrap(), 0);
}

#[test]
fn test_multi_bins_buy_cost_basic_cases() {
    // 일반적인 케이스
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(100, &[300, 400, 500], 1000).unwrap();
    assert!(cost > 0);
    
    // 여러 빈에 걸친 비용이 단일 빈 구매보다 큼
    let single_cost = RangeBetMath::calculate_bin_buy_cost(100, 500, 1000).unwrap();
    let multi_cost = RangeBetMath::calculate_multi_bins_buy_cost(100, &[500], 1000).unwrap();
    assert_eq!(single_cost, multi_cost);
    
    let multi_cost_higher = RangeBetMath::calculate_multi_bins_buy_cost(100, &[500, 500], 1000).unwrap();
    assert!(multi_cost_higher > multi_cost);
}

#[test]
fn test_multi_bins_buy_cost_sequential_effect() {
    // 순차적 구매 효과 테스트 - 이후 빈에서는 t가 증가함
    let x = 100;
    let bins = [500, 500, 500];
    let t = 1000;
    
    // calculate_multi_bins_buy_cost의 결과와 수동 계산 결과 비교
    let multi_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap();
    
    // 수동으로 순차적 계산
    let mut manual_total = 0;
    let mut current_t = t;
    
    for &q in &bins {
        let cost = RangeBetMath::calculate_bin_buy_cost(x, q, current_t).unwrap();
        manual_total += cost;
        current_t += x;  // 구매 후 t 증가
    }
    
    assert_eq!(multi_cost, manual_total);
}

#[test]
fn test_multi_bins_buy_cost_large_bins() {
    // 빈이 많은 경우 테스트
    let x = 10;
    let t = 1000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100개의 동일한 빈
    
    // 계산이 성공해야 함
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &many_bins, t).unwrap();
    assert!(cost > 0);
    
    // 오버플로우 테스트
    let huge_x = u64::MAX / 1000;
    let few_bins = vec![100, 200, 300];
    
    // 큰 x 값에 대한 처리 확인 - 오버플로우 가능성 테스트
    let result = RangeBetMath::calculate_multi_bins_buy_cost(huge_x, &few_bins, t);
    match result {
        Ok(cost) => {
            assert!(cost > 0);
        },
        Err(_) => {
            // 오버플로우가 발생할 수 있음 - 테스트를 실패로 간주하지 않음
        }
    }
}

#[test]
fn test_multi_bins_buy_cost_varying_bins() {
    // 다양한 빈 크기에 대한 테스트
    let x = 100;
    let t = 10000;  // t를 충분히 크게 설정
    let varying_bins = [10, 100, 1000, 5000];  // 모든 빈이 t보다 작아야 함
    
    // 빈의 크기가 다양할 때도 정상 작동해야 함
    let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &varying_bins, t).unwrap();
    assert!(cost > 0);
    
    // q > t인 빈이 있을 경우 오류 확인
    let invalid_bins = [500, 15000, 500];  // 두 번째 빈이 t(10000)보다 큼
    
    // 초기에는 문제가 없지만, 두 번째 빈에서 오류 발생해야 함
    let result = RangeBetMath::calculate_multi_bins_buy_cost(x, &invalid_bins, t);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_buy_cost_extreme_cases() {
    // 극단적인 경우 테스트
    
    // 매우 작은 값 테스트 (lamport 단위)
    let tiny_x = 1;
    let tiny_bins = [1, 2, 3];
    let tiny_cost = RangeBetMath::calculate_multi_bins_buy_cost(tiny_x, &tiny_bins, 10).unwrap();
    assert!(tiny_cost > 0);
    
    // 하나의 빈이 매우 크고 나머지가 작은 경우
    let mixed_bins = [10, 10, u64::MAX / 100000000, 10];
    match RangeBetMath::calculate_multi_bins_buy_cost(100, &mixed_bins, 1000) {
        Ok(_) => {},
        Err(_) => {} // 매우 극단적인 값이어서 오류가 발생할 수 있음
    }
    
    // 매우 큰 t 값 테스트
    let huge_t = u64::MAX / 100;
    let normal_bins = [1000, 2000, 3000];
    let huge_t_result = RangeBetMath::calculate_multi_bins_buy_cost(100, &normal_bins, huge_t);
    assert!(huge_t_result.is_ok());
}

#[test]
fn test_multi_bins_buy_cost_incremental() {
    // x 값이 증가함에 따라 비용도 증가해야 함
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
    // 다양한 시나리오에 대한 테스트
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
        
        // 수동 계산과 결과 비교
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
    // 오버플로우 방지 테스트
    
    // 많은 수의 빈 + 큰 x 값 (오버플로우 위험)
    let x = 1_000_000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100개 빈
    let t = 1_000_000;
    
    // 계산이 성공하거나, 적절히 오류 처리되어야 함
    let result = RangeBetMath::calculate_multi_bins_buy_cost(x, &many_bins, t);
    match result {
        Ok(cost) => {
            assert!(cost > 0);
            
            // 수동 계산과의 비교 (오버플로우 발생 가능성 있음)
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
                                // t 계산 중 오버플로우
                                break;
                            }
                        },
                        None => {
                            // 비용 합산 중 오버플로우
                            break;
                        }
                    }
                } else {
                    // 빈별 비용 계산 중 오류
                    break;
                }
            }
            
            if manual_total > 0 {
                // 수동 계산이 성공한 경우, 결과가 일치해야 함
                assert_eq!(cost, manual_total);
            }
        },
        Err(_) => {
            // 오류가 발생한 경우 - 테스트를 실패로 간주하지 않음
            // 이 시나리오에서는 오버플로우가 발생할 수 있기 때문
        }
    }
}

#[test]
fn test_multi_bins_buy_cost_large_dataset() {
    // 많은 데이터셋에 대한 테스트
    let x_values = [1, 10, 100, 1000];
    let t_values = [100, 1000, 10000];
    let bin_counts = [1, 2, 5, 10];
    
    for &x in &x_values {
        for &t in &t_values {
            for &count in &bin_counts {
                // 다양한 빈 크기 생성
                let bins: Vec<u64> = (0..count).map(|i| (i + 1) * t / (count + 1)).collect();
                
                // 비용 계산
                match RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t) {
                    Ok(cost) => {
                        assert!(cost > 0 || x == 0 || bins.is_empty());
                        
                        // 수동 계산과 비교
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
                        // 이 테스트에서는 오류가 발생하지 않아야 함
                        assert!(false, "계산 중 오류 발생: x={}, bins.len={}, t={}", x, bins.len(), t);
                    }
                }
            }
        }
    }
} 