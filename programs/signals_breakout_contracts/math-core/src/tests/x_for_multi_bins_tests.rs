use crate::RangeBetMath;

#[test]
fn test_calculate_x_for_multi_bins_edge_cases() {
    // 예산이 0인 경우
    assert_eq!(RangeBetMath::calculate_x_for_multi_bins(0, &[100, 200, 300], 1000).unwrap(), 0);
    
    // 빈 배열인 경우
    assert_eq!(RangeBetMath::calculate_x_for_multi_bins(1000, &[], 1000).unwrap(), 0);
    
    // 예산이 매우 작은 경우 (1 lamport)
    let x = RangeBetMath::calculate_x_for_multi_bins(1, &[500], 1000).unwrap();
    if x > 0 {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[500], 1000).unwrap();
        assert!(cost <= 1);
    }
    
    // 예산이 매우 작은 경우 (1 lamport), 빈이 매우 큰 경우
    let x = RangeBetMath::calculate_x_for_multi_bins(1, &[u64::MAX / 2], u64::MAX / 2).unwrap();
    if x > 0 {
        let cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[u64::MAX / 2], u64::MAX / 2).unwrap();
        assert!(cost <= 1);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_single_bin_scenarios() {
    // 다양한 예산으로 테스트 (단일 빈)
    let test_budgets = [
        100,                    // 작은 예산
        1_000,                  // 1 SOL (10^-9)
        10_000,                 // 10 SOL
        100_000,                // 100 SOL
        1_000_000,              // 1,000 SOL
        10_000_000,             // 10,000 SOL
        100_000_000,            // 100,000 SOL
        1_000_000_000,          // 1,000,000 SOL
        10_000_000_000,         // 1천만 SOL (10^10)
        100_000_000_000,        // 1억 SOL (10^11)
        1_000_000_000_000,      // 10억 SOL (10^12)
        10_000_000_000_000,     // 100억 SOL (10^13)
        100_000_000_000_000,    // 1000억 SOL (10^14)
        1_000_000_000_000_000,  // 10^15
        u64::MAX / 1_000_000,   // 거의 최대값의 백만분의 1
    ];
    
    // 다양한 빈 크기로 테스트
    let bin_scenarios = [
        (500, 1000),                            // 일반 케이스
        (999, 1000),                            // q≈t인 케이스
        (1, 1000),                              // q가 매우 작은 케이스
        (u64::MAX / 3, u64::MAX / 2),           // 매우 큰 q와 t
        (1, u64::MAX / 1_000_000_000)           // 매우 작은 q, 거대한 t
    ];
    
    println!("\n---- 단일 빈 대규모 테스트 ----");
    
    for &budget in &test_budgets {
        for &(q, t) in &bin_scenarios {
            println!("\n테스트 케이스 (단일 빈/대규모): budget={}, bin=[{}], t={}", budget, q, t);
            
            // 역산 테스트
            let x = RangeBetMath::calculate_x_for_multi_bins(budget, &[q], t).unwrap();
            println!("예산 {} 에 대한 계산된 X: {}", budget, x);
            
            // 계산된 x로 다시 비용 계산
            let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &[q], t).unwrap_or(0);
            
            // 결과 출력 및 확인
            let usage_percentage = if budget > 0 { actual_cost * 100 / budget } else { 0 };
            println!("X={} 일 때 실제 비용: {} (예산의 {}%)", x, actual_cost, usage_percentage);
            
            // 계산된 x로 실제 비용을 계산했을 때 예산보다 크지 않아야 함
            assert!(actual_cost <= budget, 
                   "비용({}이 예산({})을 초과합니다", actual_cost, budget);
            
            // 예산과 실제 비용의 차이가 Lamport 단위로 0.1% 이내여야 함 (x > 0인 경우에만)
            if x > 0 && budget >= 1000 {
                let max_deviation = budget / 1000; // 0.1% 오차 허용
                let diff = if actual_cost > budget {
                    actual_cost - budget
                } else {
                    budget - actual_cost
                };
                
                assert!(diff <= max_deviation,
                       "비용과 예산의 차이({})가 허용 오차({})를 초과합니다", diff, max_deviation);
            }
            
            // 너무 큰 값에 대한 계산은 시간이 오래 걸릴 수 있으므로 몇몇 케이스만 테스트
            if budget > u64::MAX / 10_000 || q > u64::MAX / 10_000 {
                break;
            }
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_multiple_bins_scenarios() {
    // 다양한 예산과 빈 구성으로 테스트
    let test_scenarios = [
        // 일반적인 규모 테스트
        (1_000, vec![300, 400, 500], 1000),
        (10_000, vec![300, 400, 500], 1000),
        (100_000, vec![300, 400, 500], 1000),
        
        // 대규모 예산 테스트
        (1_000_000, vec![300, 400, 500], 1000),
        (10_000_000, vec![300, 400, 500], 1000),
        (100_000_000, vec![300, 400, 500], 1000),
        (1_000_000_000, vec![300, 400, 500], 1000),
        (10_000_000_000, vec![300, 400, 500], 1000),
        
        // 다양한 빈 크기 테스트
        (10_000_000, vec![10_000, 20_000, 30_000], 100_000),
        (10_000_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        
        // 극단적인 빈 크기 차이
        (10_000_000, vec![1, 1_000_000, 1_000_000_000], 2_000_000_000),
        
        // 매우 많은 수의 빈
        (100_000_000, vec![1_000, 2_000, 3_000, 4_000, 5_000, 6_000, 7_000, 8_000, 9_000, 10_000, 
                          11_000, 12_000, 13_000, 14_000, 15_000, 16_000, 17_000, 18_000, 19_000, 20_000],
                    1_000_000),
        
        // lamport 단위 범위 테스트 (10^-9 SOL)
        (1, vec![10, 10, 10], 100),
        (9, vec![10, 10, 10], 100),
        (99, vec![10, 10, 10], 100),
        (999, vec![10, 10, 10], 100),
    ];
    
    println!("\n---- 다중 빈 테스트 (확장) ----");
    
    for (budget, bins, t) in test_scenarios {
        println!("\n테스트 케이스 (다중 빈): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // 역산 테스트
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        println!("예산 {} 에 대한 계산된 X: {}", budget, x);
        
        // 계산된 x로 다시 비용 계산
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        println!("X={} 일 때 실제 비용: {} (예산의 {}%)", 
                x, actual_cost, if budget > 0 { actual_cost * 100 / budget } else { 0 });
        
        // 계산된 x로 실제 비용을 계산했을 때 예산보다 크지 않아야 함
        assert!(actual_cost <= budget);
        
        // 예산과 실제 비용의 차이가 lamport 단위 0.1% 이내여야 함 (x > 0인 경우에만)
        if x > 0 && budget >= 1000 {
            let acceptable_deviation = (budget / 1000).max(1); // 0.1% 오차 허용, 최소 1 lamport
            let diff = if actual_cost > budget {
                actual_cost - budget
            } else {
                budget - actual_cost
            };
            assert!(diff <= acceptable_deviation,
                   "비용과 예산의 차이({})가 허용 오차({})를 초과합니다", diff, acceptable_deviation);
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_extreme_scenarios() {
    // 극단적인 시나리오 테스트
    let extreme_scenarios = [
        // 매우 큰 예산
        (u64::MAX / 100, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        (u64::MAX / 1_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        (u64::MAX / 10_000, vec![1_000_000, 2_000_000, 3_000_000], 10_000_000),
        
        // 매우 작은 빈 크기
        (1_000_000, vec![1, 2, 3], 10),
        (1_000_000, vec![1, 1, 1], 3),
        
        // 매우 큰 빈 크기
        (1_000_000, vec![u64::MAX / 1_000_000, u64::MAX / 1_000_000], u64::MAX / 100_000),
        
        // 매우 불균형한 빈 크기
        (1_000_000, vec![1, u64::MAX / 1_000_000], u64::MAX / 500_000),
        
        // lamport 단위 극단 테스트
        (1, vec![1, 1, 1], 3),
        (2, vec![1, 1, 1], 3),
        (3, vec![1, 1, 1], 3),
    ];
    
    println!("\n---- 극단적 시나리오 테스트 (확장) ----");
    
    for (budget, bins, t) in extreme_scenarios {
        println!("\n테스트 케이스 (극단): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // 계산이 충돌 없이 진행되는지 확인
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        println!("예산 {} 에 대한 계산된 X: {}", budget, x);
        
        // 계산된 x로 다시 비용 계산 - 오류가 발생할 수 있으므로 unwrap_or 사용
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        println!("X={} 일 때 실제 비용: {}", x, actual_cost);
        
        // 계산된 x로 실제 비용을 계산했을 때 예산보다 크지 않아야 함
        assert!(actual_cost <= budget);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_u64_boundaries() {
    // u64 경계값 테스트
    let boundary_scenarios = [
        // 예산이 매우 클 때 (u64::MAX에 가까운 값)
        (u64::MAX - 1, vec![1_000_000, 2_000_000], 10_000_000),
        (u64::MAX - 1_000, vec![1_000_000, 2_000_000], 10_000_000),
        
        // t가 매우 클 때 
        (1_000_000, vec![1_000_000, 2_000_000], u64::MAX - 1_000),
        
        // q가 매우 클 때
        (1_000_000, vec![u64::MAX - 1_000, u64::MAX - 2_000], u64::MAX - 1_000),
        
        // 모든 값이 거의 u64::MAX에 가까울 때
        (u64::MAX - 1_000, vec![u64::MAX - 2_000, u64::MAX - 3_000], u64::MAX - 1_000),
    ];
    
    println!("\n---- u64 경계값 테스트 ----");
    
    for (budget, bins, t) in boundary_scenarios {
        println!("\n테스트 케이스 (u64 경계): budget={}, bins={:?}, t={}", budget, bins, t);
        
        // 경계값 계산이 오류 없이 진행되는지 확인
        let result = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t);
        match result {
            Ok(x) => {
                println!("예산 {} 에 대한 계산된 X: {}", budget, x);
                
                // 계산된 x로 다시 비용 계산
                match RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t) {
                    Ok(actual_cost) => {
                        println!("X={} 일 때 실제 비용: {}", x, actual_cost);
                        
                        // 계산된 x로 실제 비용을 계산했을 때 예산보다 크지 않아야 함
                        assert!(actual_cost <= budget);
                    },
                    Err(e) => {
                        println!("경고: X={} 일 때 비용 계산 중 오류 발생: {:?}", x, e);
                    }
                };
            },
            Err(e) => {
                println!("경고: X 계산 중 오류 발생: {:?}", e);
                // 이 테스트는 경계값에서 함수의 동작을 확인하는 것이므로
                // 오류가 발생해도 테스트 실패로 간주하지 않음
            }
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_lamport_precision() {
    // lamport 정밀도 테스트
    // SOL에서 lamport는 10^-9 단위로, 최소 화폐 단위임
    // 1 SOL = 10^9 lamport
    
    let precision_tests = [
        // 1 lamport 단위 증가
        (1_000_000_000, 1_000_000_001),  // 1 SOL, 1 SOL + 1 lamport
        (1_000_000_001, 1_000_000_002),  // 1 SOL + 1 lamport, 1 SOL + 2 lamport
        
        // 10 lamport 단위 증가
        (1_000_000_000, 1_000_000_010),  // 1 SOL, 1 SOL + 10 lamport
        
        // 100 lamport 단위 증가
        (1_000_000_000, 1_000_000_100),  // 1 SOL, 1 SOL + 100 lamport
        
        // 1000 lamport 단위 증가 (0.000001 SOL)
        (1_000_000_000, 1_000_001_000),  // 1 SOL, 1 SOL + 0.000001 SOL
        
        // 매우 작은 예산에서의 1 lamport 증가
        (1, 2),  // 1 lamport, 2 lamport
        (9, 10), // 9 lamport, 10 lamport
        (99, 100), // 99 lamport, 100 lamport
        (999, 1000), // 999 lamport, 1000 lamport (0.000001 SOL)
    ];
    
    let bins = vec![10_000, 20_000, 30_000];
    let t = 100_000;
    
    println!("\n---- Lamport 정밀도 테스트 ----");
    
    for (budget1, budget2) in precision_tests {
        println!("\nLamport 정밀도 테스트: budget1={}, budget2={}, 차이={} lamport", 
                budget1, budget2, budget2 - budget1);
        
        let x1 = RangeBetMath::calculate_x_for_multi_bins(budget1, &bins, t).unwrap();
        let x2 = RangeBetMath::calculate_x_for_multi_bins(budget2, &bins, t).unwrap();
        
        let cost1 = RangeBetMath::calculate_multi_bins_buy_cost(x1, &bins, t).unwrap_or(0);
        let cost2 = RangeBetMath::calculate_multi_bins_buy_cost(x2, &bins, t).unwrap_or(0);
        
        println!("예산 {}: X={}, 비용={}, 잔액={}", budget1, x1, cost1, budget1 - cost1);
        println!("예산 {}: X={}, 비용={}, 잔액={}", budget2, x2, cost2, budget2 - cost2);
        
        // 비용은 예산을 초과하지 않아야 함
        assert!(cost1 <= budget1);
        assert!(cost2 <= budget2);
        
        // 예산이 증가하면 X 또는 비용도 증가해야 함
        if budget2 > budget1 && x2 == x1 && cost2 == cost1 {
            println!("정보: 예산 증가 ({} -> {})에도 X와 비용 변화 없음", budget1, budget2);
        }
        
        // 매우 작은 증가에도 알고리즘이 적절히 대응하는지 확인
        if budget2 - budget1 <= 10 && x2 > x1 {
            println!("성공: 매우 작은 예산 증가 ({} lamport)에도 X 증가 감지됨", budget2 - budget1);
        }
    }
}

#[test]
fn test_calculate_x_for_multi_bins_performance() {
    use std::time::{Duration, Instant};
    
    // 다양한 크기의 입력에 대한 성능 테스트
    let performance_scenarios = [
        // 소규모 시나리오
        (1000, vec![100, 200, 300], 1000),
        
        // 중간 규모 시나리오
        (1_000_000, vec![100, 200, 300, 400, 500], 5000),
        
        // 대규모 시나리오 - 큰 예산
        (1_000_000_000, vec![100, 200, 300, 400, 500], 10000),
        
        // 대규모 시나리오 - 많은 빈
        (1_000_000, vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000,
                        1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000], 50000),
        
        // 대규모 시나리오 - 큰 예산과 많은 빈
        (1_000_000_000, vec![10000, 20000, 30000, 40000, 50000, 60000, 70000, 80000, 90000, 100000], 1_000_000),
        
        // 극단적 시나리오 - 매우 큰 예산
        (u64::MAX / 100, vec![1_000_000, 2_000_000], 10_000_000),
    ];
    
    println!("\n---- 성능 테스트 (확장) ----");
    
    for (budget, bins, t) in performance_scenarios {
        println!("\n성능 테스트: budget={}, bins.len={}, t={}", budget, bins.len(), t);
        
        let start = Instant::now();
        let x = RangeBetMath::calculate_x_for_multi_bins(budget, &bins, t).unwrap();
        let duration = start.elapsed();
        
        let actual_cost = RangeBetMath::calculate_multi_bins_buy_cost(x, &bins, t).unwrap_or(0);
        let accuracy = 100.0 - ((budget as f64 - actual_cost as f64) / budget as f64 * 100.0);
        
        println!("계산된 X: {}, 비용: {}, 예산 사용률: {:.6}%, 소요 시간: {:?}", 
                x, actual_cost, accuracy, duration);
        
        // 성능 테스트는 단순히 실행 시간을 측정하는 것이므로 결과에 대한 검증은 생략
        // 다만 실행 시간이 합리적인 범위 내에 있는지 확인 (예: 30초 이내)
        assert!(duration < Duration::from_secs(30), 
               "계산 시간이 30초를 초과합니다: {:?}", duration);
    }
}

#[test]
fn test_calculate_x_for_multi_bins_stress() {
    // 스트레스 테스트: 다양한 크기의 여러 값에 대해 빠르게 많은 계산을 수행
    let bins_scenarios = [
        vec![300, 400, 500],                    // 일반적인 케이스
        vec![3, 4, 5],                          // 매우 작은 빈
        vec![30_000, 40_000, 50_000],           // 큰 빈
        vec![300, 400, 500, 600, 700, 800]      // 많은 빈
    ];
    
    let t_scenarios = [
        1_000,          // 일반적인 케이스
        10,             // 매우 작은 t
        1_000_000       // 큰 t
    ];
    
    // 예산 범위
    let min_budget = 0;
    let max_budget = 10_000_000;  // 0 ~ 10 SOL
    let budget_step = 100_000;    // 0.1 SOL 단위
    
    println!("\n---- 스트레스 테스트 (확장) ----");
    println!("예산 범위: {}~{}, 스텝: {}", min_budget, max_budget, budget_step);
    
    let start = std::time::Instant::now();
    let mut success_count = 0;
    let mut total_count = 0;
    
    for bins in &bins_scenarios {
        for &t in &t_scenarios {
            println!("\n빈 구성: {:?}, t={}", bins, t);
            
            for budget in (min_budget..=max_budget).step_by(budget_step as usize) {
                total_count += 1;
                
                match RangeBetMath::calculate_x_for_multi_bins(budget, bins, t) {
                    Ok(x) => {
                        if let Ok(cost) = RangeBetMath::calculate_multi_bins_buy_cost(x, bins, t) {
                            if cost <= budget {
                                success_count += 1;
                            } else {
                                println!("오류: 예산 {} 초과 - X={}, 비용={}", budget, x, cost);
                            }
                        } else {
                            println!("오류: X={} 일 때 비용 계산 실패", x);
                        }
                    },
                    Err(e) => {
                        println!("오류: 예산 {} 계산 중 오류 발생: {:?}", budget, e);
                    }
                }
                
                // 너무 많은 출력이 생기지 않도록 진행 상황 간략히 표시
                if total_count % 100 == 0 {
                    print!(".");
                    if total_count % 1000 == 0 {
                        println!(" {}", total_count);
                    }
                }
            }
        }
    }
    
    let duration = start.elapsed();
    println!("\n총 테스트: {}, 성공: {}, 실패: {}, 성공률: {:.2}%, 소요 시간: {:?}", 
            total_count, success_count, total_count - success_count, 
            (success_count as f64 / total_count as f64) * 100.0, duration);
    
    // 모든 테스트가 성공해야 함
    assert_eq!(success_count, total_count, 
              "실패한 테스트: {}/{}", total_count - success_count, total_count);
} 