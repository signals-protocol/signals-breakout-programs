use anchor_lang::prelude::*;
use crate::errors::RangeBetError;
use brine_fp::{self, UnsignedNumeric};

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

        // brine-fp 고정소수점 계산 구현
        let x_fp = UnsignedNumeric::new(x as u128).unwrap();
        let q_fp = UnsignedNumeric::new(q as u128).unwrap();
        let t_fp = UnsignedNumeric::new(t as u128).unwrap();
        
        // 계산: x + (q-T)*ln((T+x)/T)
        let mut cost = x_fp.clone();
        
        if q != t {
            // (T+x)/T 계산
            let ratio = match t_fp.checked_add(&x_fp) {
                Some(sum) => match sum.checked_div(&t_fp) {
                    Some(r) => r,
                    None => return Err(error!(RangeBetError::MathOverflow))
                },
                None => return Err(error!(RangeBetError::MathOverflow))
            };
            
            // ln((T+x)/T) 계산
            let log_term = match ratio.log() {
                Some(l) => l,
                None => return Err(error!(RangeBetError::MathOverflow))
            };
            
            if q > t {
                // q > T인 경우, cost = x + (q-T)*ln((T+x)/T)
                let q_minus_t = q_fp.signed().checked_sub(&t_fp.signed()).unwrap();
                let additional_cost = match q_minus_t.checked_mul(&log_term) {
                    Some(c) => c,
                    None => return Err(error!(RangeBetError::MathOverflow))
                };
                
                // 음수 검사
                if !additional_cost.is_negative {
                    // 양수일 경우 추가
                    cost = match cost.checked_add(&additional_cost.value) {
                        Some(c) => c,
                        None => return Err(error!(RangeBetError::MathOverflow))
                    };
                } else {
                    // 음수일 경우 차감
                    let abs_additional_cost = additional_cost.negate().value;
                    if abs_additional_cost.greater_than(&cost) {
                        return Err(error!(RangeBetError::MathUnderflow));
                    }
                    cost = match cost.checked_sub(&abs_additional_cost) {
                        Some(c) => c,
                        None => return Err(error!(RangeBetError::MathUnderflow))
                    };
                }
            } else {
                // q < T인 경우, cost = x - (T-q)*ln((T+x)/T)
                let t_minus_q = t_fp.signed().checked_sub(&q_fp.signed()).unwrap();
                let reduction_cost = match t_minus_q.checked_mul(&log_term) {
                    Some(c) => c,
                    None => return Err(error!(RangeBetError::MathOverflow))
                };
                
                // 음수 검사
                if !reduction_cost.is_negative {
                    // 양수일 경우 차감
                    let unsigned_reduction = reduction_cost.value;
                    if unsigned_reduction.greater_than(&cost) {
                        return Err(error!(RangeBetError::MathUnderflow));
                    }
                    cost = match cost.checked_sub(&unsigned_reduction) {
                        Some(c) => c,
                        None => return Err(error!(RangeBetError::MathUnderflow))
                    };
                } else {
                    // 음수일 경우 추가
                    cost = match cost.checked_add(&reduction_cost.negate().value) {
                        Some(c) => c,
                        None => return Err(error!(RangeBetError::MathOverflow))
                    };
                }
            }
        }
        
        // u64로 변환 (반올림)
        match cost.to_imprecise() {
            Some(precise_cost) => {
                if precise_cost > u64::MAX as u128 {
                    return Err(error!(RangeBetError::MathOverflow));
                }
                Ok(precise_cost as u64)
            },
            None => Err(error!(RangeBetError::MathOverflow))
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
                        let diff_left = (cost as i128) - (left_cost as i128);
                        let diff_right = (right_cost as i128) - (cost as i128);
                        return Ok(if diff_left < diff_right { left } else { right });
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

        // 고정소수점으로 변환하여 계산
        let x_fp = UnsignedNumeric::new(x as u128).unwrap();
        let q_fp = UnsignedNumeric::new(q as u128).unwrap();
        let t_fp = UnsignedNumeric::new(t as u128).unwrap();

        // 계산: x + (q-T)*ln(T/(T-x))
        let mut revenue = x_fp.clone();
        
        if q != t {
            // T/(T-x) 계산
            let t_minus_x = match t_fp.checked_sub(&x_fp) {
                Some(diff) => diff,
                None => return Err(error!(RangeBetError::MathUnderflow))
            };
            
            let ratio = match t_fp.checked_div(&t_minus_x) {
                Some(r) => r,
                None => return Err(error!(RangeBetError::MathOverflow))
            };
            
            // ln(T/(T-x)) 계산
            let log_term = match ratio.log() {
                Some(l) => l,
                None => return Err(error!(RangeBetError::MathOverflow))
            };
            
            if q > t {
                // q > T인 경우, 추가 수익
                let q_minus_t = q_fp.signed().checked_sub(&t_fp.signed()).unwrap();
                let additional_revenue = match q_minus_t.checked_mul(&log_term) {
                    Some(r) => r,
                    None => return Err(error!(RangeBetError::MathOverflow))
                };
                
                // 음수 검사
                if !additional_revenue.is_negative {
                    // 양수일 경우 추가
                    revenue = match revenue.checked_add(&additional_revenue.value) {
                        Some(r) => r,
                        None => return Err(error!(RangeBetError::MathOverflow))
                    };
                } else {
                    // 음수일 경우 차감
                    let abs_additional_revenue = additional_revenue.negate().value;
                    if abs_additional_revenue.greater_than(&revenue) {
                        return Err(error!(RangeBetError::SellCalculationUnderflow));
                    }
                    revenue = match revenue.checked_sub(&abs_additional_revenue) {
                        Some(r) => r,
                        None => return Err(error!(RangeBetError::SellCalculationUnderflow))
                    };
                }
            } else {
                // q < T인 경우, 수익 감소
                let t_minus_q = t_fp.signed().checked_sub(&q_fp.signed()).unwrap();
                let reduction_revenue = match t_minus_q.checked_mul(&log_term) {
                    Some(r) => r,
                    None => return Err(error!(RangeBetError::MathOverflow))
                };
                
                // 음수 검사
                if !reduction_revenue.is_negative {
                    // 양수일 경우 차감
                    let unsigned_reduction = reduction_revenue.value;
                    if unsigned_reduction.greater_than(&revenue) {
                        return Err(error!(RangeBetError::SellCalculationUnderflow));
                    }
                    revenue = match revenue.checked_sub(&unsigned_reduction) {
                        Some(r) => r,
                        None => return Err(error!(RangeBetError::SellCalculationUnderflow))
                    };
                } else {
                    // 음수일 경우 추가
                    revenue = match revenue.checked_add(&reduction_revenue.negate().value) {
                        Some(r) => r,
                        None => return Err(error!(RangeBetError::MathOverflow))
                    };
                }
            }
        }
        
        // u64로 변환
        match revenue.to_imprecise() {
            Some(precise_revenue) => {
                if precise_revenue > u64::MAX as u128 {
                    return Err(error!(RangeBetError::MathOverflow));
                }
                Ok(precise_revenue as u64)
            },
            None => Err(error!(RangeBetError::MathOverflow))
        }
    }
} 