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
        if x == 0 {
            return Ok(0);
        }
        if t == 0 {
            return Ok(x); // 첫 구매
        }

        // 고정소수점 대신 double 정밀도 계산으로 구현
        // TODO: brine-fp나 정밀한 고정소수점 라이브러리로 교체 필요
        let x_f64 = x as f64;
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        
        // 계산: x + (q-T)*ln((T+x)/T)
        let mut cost = x_f64;
        
        if q != t {
            // (T+x)/T 계산
            let ratio = (t_f64 + x_f64) / t_f64;
            // ln((T+x)/T) 계산
            let log_term = ratio.ln();
            
            if q > t {
                // q > T인 경우, cost = x + (q-T)*ln((T+x)/T)
                let q_minus_t = q_f64 - t_f64;
                cost = cost + (q_minus_t * log_term);
            } else {
                // q < T인 경우, cost = x - (T-q)*ln((T+x)/T)
                let t_minus_q = t_f64 - q_f64;
                // 언더플로우 검사
                if (t_minus_q * log_term) > cost {
                    return Err(error!(RangeBetError::MathUnderflow));
                }
                cost = cost - (t_minus_q * log_term);
            }
        }
        
        // u64로 변환 (반올림)
        let result = cost.round() as u64;
        Ok(result)
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
        
        // 탐색 범위 설정
        let mut left = 0u64;
        let mut right = if q > 0 {
            let calc = (t as u128 * cost as u128) / (q as u128);
            if calc > u64::MAX as u128 {
                u64::MAX
            } else {
                calc as u64
            }
        } else {
            u64::MAX
        };
        
        // 이진 탐색 (최대 64회 반복)
        for _ in 0..64 {
            let mid = left + (right - left) / 2;
            
            // 중간값의 비용 계산
            let calculated_cost = Self::calculate_cost(mid, q, t)?;
            
            // 정확한 값을 찾거나 정밀도 한계에 도달
            if calculated_cost == cost || right - left <= 1 {
                if right - left <= 1 {
                    let left_cost = Self::calculate_cost(left, q, t)?;
                    let right_cost = Self::calculate_cost(right, q, t)?;
                    
                    if left_cost == cost { return Ok(left); }
                    if right_cost == cost { return Ok(right); }
                    
                    // 목표 비용에 가장 가까운 값 반환
                    if cost > left_cost && cost > right_cost {
                        return Ok(if right_cost > left_cost { right } else { left });
                    } else {
                        return Ok(if cost - left_cost < right_cost - cost { left } else { right });
                    }
                }
                return Ok(mid);
            }
            
            // 탐색 범위 조정
            if calculated_cost < cost {
                left = mid;
            } else {
                right = mid;
            }
        }
        
        // 64회 반복 후 최선의 근사값 반환
        let left_cost = Self::calculate_cost(left, q, t)?;
        let right_cost = Self::calculate_cost(right, q, t)?;
        
        if (cost as i128 - left_cost as i128).abs() < (right_cost as i128 - cost as i128).abs() {
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
        require!(x < t, RangeBetError::CannotSellEntireSupply);

        // f64로 변환하여 계산
        let x_f64 = x as f64;
        let q_f64 = q as f64;
        let t_f64 = t as f64;

        // 계산: x + (q-T)*ln(T/(T-x))
        let mut revenue = x_f64;
        
        if q != t {
            // T/(T-x) 계산
            let ratio = t_f64 / (t_f64 - x_f64);
            // ln(T/(T-x)) 계산
            let log_term = ratio.ln();
            
            if q > t {
                // q > T인 경우, 추가 수익
                let q_minus_t = q_f64 - t_f64;
                revenue = revenue + (q_minus_t * log_term);
            } else {
                // q < T인 경우, 수익 감소
                let t_minus_q = t_f64 - q_f64;
                
                // 언더플로우 검사
                if (t_minus_q * log_term) > revenue {
                    return Err(error!(RangeBetError::SellCalculationUnderflow));
                }
                
                revenue = revenue - (t_minus_q * log_term);
            }
        }
        
        // u64로 변환 (반올림)
        let result = revenue.round() as u64;
        Ok(result)
    }
} 