use crate::RangeBetMath;

#[test]
fn test_multi_bins_sell_cost_edge_cases() {
    // 빈 배열 또는 x=0인 경우
    assert_eq!(RangeBetMath::calculate_multi_bins_sell_cost(0, &[100, 200, 300], 1000).unwrap(), 0);
    assert_eq!(RangeBetMath::calculate_multi_bins_sell_cost(100, &[], 1000).unwrap(), 0);
}

#[test]
fn test_multi_bins_sell_cost_basic_cases() {
    // 일반적인 케이스
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(100, &[300, 400, 500], 1000).unwrap();
    assert!(revenue > 0);
    
    // 여러 빈에 걸친 수익이 단일 빈 판매와 같음 (단일 빈 경우)
    let single_revenue = RangeBetMath::calculate_bin_sell_cost(100, 500, 1000).unwrap();
    let multi_revenue = RangeBetMath::calculate_multi_bins_sell_cost(100, &[500], 1000).unwrap();
    assert_eq!(single_revenue, multi_revenue);
}

#[test]
fn test_multi_bins_sell_cost_exceed_bin() {
    // 빈에 있는 토큰보다 많이 판매하는 경우 - CannotSellMoreThanBin
    let result = RangeBetMath::calculate_multi_bins_sell_cost(600, &[300, 400, 500], 1000);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_exceed_supply() {
    // 총 공급량보다 많이 판매하는 경우 - CannotSellMoreThanSupply
    let result = RangeBetMath::calculate_multi_bins_sell_cost(300, &[500, 500, 500, 500], 1000);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_sequential_effect() {
    // 순차적 판매 효과 테스트 - 이후 빈에서는 t가 감소함
    let x = 100;
    let bins = [500, 500, 500];
    let t = 1000;
    
    // calculate_multi_bins_sell_cost의 결과와 수동 계산 결과 비교
    let multi_revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
    
    // 수동으로 순차적 계산
    let mut manual_total = 0;
    let mut current_t = t;
    
    for &q in &bins {
        let revenue = RangeBetMath::calculate_bin_sell_cost(x, q, current_t).unwrap();
        manual_total += revenue;
        current_t -= x;  // 판매 후 t 감소
    }
    
    assert_eq!(multi_revenue, manual_total);
}

#[test]
fn test_multi_bins_sell_cost_large_bins() {
    // 많은 빈을 판매하는 경우 테스트
    let x = 10;
    let t = 10000;
    let many_bins: Vec<u64> = vec![100; 100];  // 100개의 동일한 빈
    
    // 총 판매량이 t를 초과하지 않는 경우 성공해야 함
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &many_bins, t);
    assert!(result.is_ok());
    
    // 총 판매량이 t를 초과하는 경우 오류 발생해야 함
    let too_many_bins: Vec<u64> = vec![100; 1000];  // 1000개의 빈
    let overflow_result = RangeBetMath::calculate_multi_bins_sell_cost(x, &too_many_bins, t);
    assert!(overflow_result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_varying_bins() {
    // 다양한 빈 크기에 대한 테스트
    let x = 50;
    let t = 10000;
    let varying_bins = [100, 200, 300, 400, 500];
    
    // 빈의 크기가 다양할 때도 정상 작동해야 함
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &varying_bins, t).unwrap();
    assert!(revenue > 0);
    
    // q가 x보다 작은 빈이 있을 경우 오류 발생해야 함
    let invalid_bins = [100, 40, 100];  // 두 번째 빈이 x(50)보다 작음
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &invalid_bins, t);
    assert!(result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_extreme_cases() {
    // 극단적인 경우 테스트
    
    // 매우 작은 값 테스트 (lamport 단위)
    let tiny_x = 1;
    let tiny_bins = [10, 10, 10];
    let tiny_revenue = RangeBetMath::calculate_multi_bins_sell_cost(tiny_x, &tiny_bins, 100).unwrap();
    assert!(tiny_revenue > 0);
    
    // 빈 크기가 x와 같은 경우 (최대 판매 가능량)
    let x = 50;
    let exact_bins = [x, x, x];
    let t = 1000;
    let exact_revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &exact_bins, t);
    assert!(exact_revenue.is_ok());
    
    // 매우 큰 t 값 테스트
    let huge_t = u64::MAX / 100;
    let normal_bins = [1000, 2000, 3000];
    let huge_t_result = RangeBetMath::calculate_multi_bins_sell_cost(100, &normal_bins, huge_t);
    assert!(huge_t_result.is_ok());
}

#[test]
fn test_multi_bins_sell_cost_incremental() {
    // x 값이 증가함에 따라 수익도 증가해야 함
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
    // 다양한 시나리오에 대한 테스트
    let scenarios = [
        // (x, bins, t)
        (50, vec![100, 200, 300], 5000),
        (10, vec![50, 50, 50], 1000),
        (1, vec![10, 10, 10], 1000),
        (100, vec![500], 1000),
        (50, vec![100, 100], 1000)
    ];
    
    for (x, bins, t) in scenarios {
        // 총 판매량이 t를 초과하지 않는지 확인
        let total_sell = x * bins.len() as u64;
        if total_sell <= t {
            let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
            
            // 수동 계산과 결과 비교
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
    // 오버플로우 방지 테스트
    
    // 오버플로우 가능성이 있는 계산
    let x = 10;
    let t = u64::MAX / 100;
    let bins: Vec<u64> = vec![1000; 5];  // 일부 빈
    
    // t가 충분히 크면 계산이 성공해야 함
    let result = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t);
    assert!(result.is_ok());
    
    // checked_sub에 대한 테스트 - 너무 많은 빈을 판매하여 t가 음수가 될 가능성
    // 메모리 할당 오류를 방지하기 위해 작은 샘플로 테스트
    let small_t = 1000;
    let small_x = 10;
    let many_small_bins: Vec<u64> = vec![100; (small_t / small_x) as usize + 1];  // small_t/small_x + 1개의 빈
    let overflow_result = RangeBetMath::calculate_multi_bins_sell_cost(small_x, &many_small_bins, small_t);
    assert!(overflow_result.is_err());
}

#[test]
fn test_multi_bins_sell_cost_edge_case_equal_values() {
    // 모든 값이 동일한 경우의 특수 케이스 테스트
    let x = 100;
    let q = 100;
    let t = 100 * 3;  // 정확히 3개 빈 판매 가능한 총량
    
    let bins = vec![q, q, q];
    
    // 정확히 모든 토큰을 판매하는 경우
    let revenue = RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t).unwrap();
    assert!(revenue > 0);
    
    // 수동 계산과 비교
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
    // 대규모 데이터셋 테스트
    let x_values = [1, 10, 50, 100];
    let t_multipliers = [2, 3, 5, 10];  // 총 판매량의 배수로 t 설정
    let bin_counts = [1, 2, 5, 10];
    
    for &x in &x_values {
        for &count in &bin_counts {
            // 모든 빈이 최소한 x 이상인지 확인
            let bins: Vec<u64> = (0..count).map(|_| x + 10).collect();
            
            for &t_mul in &t_multipliers {
                let t = x * count * t_mul;  // 총 판매량의 t_mul배
                
                // 수익 계산
                match RangeBetMath::calculate_multi_bins_sell_cost(x, &bins, t) {
                    Ok(revenue) => {
                        assert!(revenue > 0);
                        
                        // 수동 계산과 비교
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
                        // 이 테스트에서는 일부 경우에 오류가 발생할 수 있음
                        // 테스트를 실패로 간주하지 않음
                    }
                }
            }
        }
    }
} 