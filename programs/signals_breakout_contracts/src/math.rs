use anchor_lang::prelude::*;
use crate::errors::RangeBetError;

/// Range-Bet Math 라이브러리
pub struct RangeBetMath;

impl RangeBetMath {
    /// 토큰 구매 비용 계산 함수
    /// Formula: ∫(q+t)/(T+t) dt = x + (q-T)*ln((T+x)/T)
    /// @param x 구매할 토큰 수량
    /// @param q 현재 Bin의 토큰 수량
    /// @param t 시장 전체 토큰 수량
    /// @return 담보 토큰 비용
    pub fn calculate_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // q가 t보다 크면 에러 발생 (불가능한 상태)
        require!(q <= t, RangeBetError::InvalidBinState);

        if x == 0 {
            return Ok(0);
        }
        if t == 0 {
            return Ok(x); // 첫 구매
        }
        
        // q = t인 경우 단순히 x 반환 (로그 항의 계수가 0)
        if q == t {
            return Ok(x);
        }
        
        // 더 안정적인 계산을 위해 정밀한 f64로 직접 계산
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        let x_f64 = x as f64;
        
        // 비율 계산: (t+x)/t = 1 + x/t
        let ratio = (t_f64 + x_f64) / t_f64;
        // 자연로그 계산
        let ln_ratio = ratio.ln();
        
        // q < t 경우: x - (t-q)*ln((t+x)/t)
        let reduction = (t_f64 - q_f64) * ln_ratio;
        
        // 언더플로우 방지를 위한 검사
        let cost_f64 = if reduction > x_f64 {
            // 극단적인 경우, 최소 단위인 1 반환
            1.0
        } else {
            x_f64 - reduction
        };
        
        // 결과가 0보다 작으면 1 반환 (최소 단위)
        if cost_f64 <= 0.0 {
            Ok(1)
        } else {
            // 반올림하여 u64로 변환
            let cost = (cost_f64 + 0.5) as u64;
            // 0이 될 경우 최소값 1 반환
            Ok(if cost == 0 { 1 } else { cost })
        }
    }
    
    /// 특정 비용으로 구매 가능한 토큰 수량 계산 (이진 탐색)
    /// @param cost 사용 가능한 담보 비용
    /// @param q 현재 Bin의 토큰 수량
    /// @param t 시장 전체 토큰 수량
    /// @return 구매 가능한 토큰 수량
    pub fn calculate_x_for_cost(cost: u64, q: u64, t: u64) -> Result<u64> {
        if cost == 0 {
            return Ok(0);
        }
        if t == 0 {
            return Ok(cost); // 첫 구매
        }
        
        // 허용 오차 계산
        let epsilon_abs: u64 = 10_000;                    // 1e-5 USDC (약 $0.00001)
        let epsilon_rel: u64 = cost / 2_000;              // 0.05% 상대 오차
        let epsilon: u64 = epsilon_abs.max(epsilon_rel).max(1); // 최소 1 lamport 보장
        
        let mut right: u64 = u64::MAX;
        let mut left: u64 = 0;
        
        // 이진 탐색 (최대 32회 반복)
        for _ in 0..32 {
            // 탐색 범위가 허용 오차 내에 들어오면 종료
            if right - left <= epsilon {
                break;
            }
            
            let mid = left + (right - left) / 2;
            
            // 중간값의 비용 계산
            let calculated_cost = match Self::calculate_cost(mid, q, t) {
                Ok(c) => c,
                Err(_) => {
                    // 오버플로우가 발생하면 범위를 줄임
                    right = mid;
                    continue;
                }
            };
            
            // 계산된 비용이 목표 비용과 허용 오차 내에 있으면 바로 반환
            if (calculated_cost as i128 - cost as i128).abs() as u64 <= epsilon {
                return Ok(mid);
            }
            
            // 탐색 범위 조정
            if calculated_cost < cost {
                left = mid;
            } else {
                right = mid;
            }
        }
        
        // 최종 양쪽 값의 비용 계산하여 더 가까운 값 선택
        let left_cost = match Self::calculate_cost(left, q, t) {
            Ok(c) => c,
            Err(_) => return Ok(right), // 왼쪽이 오버플로우면 오른쪽 반환
        };
        
        let right_cost = match Self::calculate_cost(right, q, t) {
            Ok(c) => c,
            Err(_) => return Ok(left), // 오른쪽이 오버플로우면 왼쪽 반환
        };
        
        // 목표 비용에 더 가까운 값 선택
        let left_diff = (cost as i128 - left_cost as i128).abs();
        let right_diff = (right_cost as i128 - cost as i128).abs();
        
        if left_diff < right_diff {
            Ok(left)
        } else {
            Ok(right)
        }
    }
    
    /// 토큰 판매 시 수익 계산 함수
    /// Formula: ∫(q-t)/(T-t) dt = x + (q-T)*ln(T/(T-x))
    /// @param x 판매할 토큰 수량
    /// @param q 현재 Bin의 토큰 수량
    /// @param t 시장 전체 토큰 수량
    /// @return 판매 수익
    pub fn calculate_sell_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // 입력 유효성 검사
        if x == 0 {
            return Ok(0);
        }
        
        require!(x <= q, RangeBetError::CannotSellMoreThanBin);
        require!(x <= t, RangeBetError::CannotSellMoreThanSupply);
        require!(q <= t, RangeBetError::InvalidBinState);

        // q = t인 경우 단순히 x 반환 (로그 항의 계수가 0)
        if q == t {
            return Ok(x);
        }
        
        // 더 안정적인 계산을 위해, 정밀한 f64로 직접 계산
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        let x_f64 = x as f64;
        
        // t-x가 0인지 확인
        let t_minus_x_f64 = t_f64 - x_f64;
        if t_minus_x_f64 <= 0.0 {
            return Err(error!(RangeBetError::MathUnderflow));
        }
        
        // 비율 계산 및 자연로그
        let ratio = t_f64 / t_minus_x_f64;
        let ln_ratio = ratio.ln();
        
        // q < t 경우: x - (t-q)*ln(t/(t-x))
        let reduction = (t_f64 - q_f64) * ln_ratio;
        
        // 언더플로우 방지를 위한 검사
        let revenue_f64 = if reduction > x_f64 {
            // 극단적인 경우, 최소 단위인 1 반환
            1.0
        } else {
            x_f64 - reduction
        };
        
        // 결과가 0보다 작으면 1 반환 (최소 단위)
        if revenue_f64 <= 0.0 {
            Ok(1)
        } else {
            // 반올림하여 u64로 변환
            let revenue = (revenue_f64 + 0.5) as u64;
            // 0이 될 경우 최소값 1 반환
            Ok(if revenue == 0 { 1 } else { revenue })
        }
    }
}