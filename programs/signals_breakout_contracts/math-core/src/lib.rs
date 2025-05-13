use anchor_lang::prelude::*;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
pub mod tests;

#[error_code]
pub enum MathError {
    #[msg("Math operation overflow")]
    MathOverflow,
    #[msg("Invalid bin state")]
    InvalidBinState,
    #[msg("Cannot sell more than bin")]
    CannotSellMoreThanBin,
    #[msg("Cannot sell more than supply")]
    CannotSellMoreThanSupply,
    #[msg("Sell calculation underflow")]
    SellCalculationUnderflow,
}

/// Range-Bet Math library
pub struct RangeBetMath;

impl RangeBetMath {
    /// Token purchase cost calculation function
    /// Formula: ∫(q+t)/(T+t) dt = x + (q-T)*ln((T+x)/T)
    /// @param x Amount of tokens to purchase
    /// @param q Current token quantity in the bin
    /// @param t Total token quantity in the market
    /// @return Collateral token cost
    pub fn calculate_bin_buy_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // Error if q is greater than t (impossible state)
        require!(q <= t, MathError::InvalidBinState);

        if x == 0 {
            return Ok(0);
        }
        if t == 0 {
            return Ok(x); // First purchase
        }
        
        // If q = t, simply return x (coefficient of log term is 0)
        if q == t {
            return Ok(x);
        }
        
        // Calculate directly with precise f64 for more stable calculation
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        let x_f64 = x as f64;
        
        // Calculate ratio: (t+x)/t = 1 + x/t
        let ratio = (t_f64 + x_f64) / t_f64;
        // Calculate natural logarithm
        let ln_ratio = ratio.ln();
        
        // For q < t case: x - (t-q)*ln((t+x)/t)
        let reduction = (t_f64 - q_f64) * ln_ratio;
        
        // Check to prevent underflow
        let cost_f64 = if reduction > x_f64 {
            // In extreme cases, return minimum unit 1
            1.0
        } else {
            x_f64 - reduction
        };
        
        // If result is less than 0, return 1 (minimum unit)
        if cost_f64 <= 0.0 {
            Ok(1)
        } else {
            // Round and convert to u64
            let cost = (cost_f64 + 0.5) as u64;
            // Return minimum value 1 if becomes 0
            Ok(if cost == 0 { 1 } else { cost })
        }
    }
    
    /// Calculate revenue from token sales
    /// Formula: ∫(q-t)/(T-t) dt = x + (q-T)*ln(T/(T-x))
    /// @param x Amount of tokens to sell
    /// @param q Current token quantity in the bin
    /// @param t Total token quantity in the market
    /// @return Sale revenue
    pub fn calculate_bin_sell_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // Input validation
        if x == 0 {
            return Ok(0);
        }
        
        require!(x <= q, MathError::CannotSellMoreThanBin);
        require!(x <= t, MathError::CannotSellMoreThanSupply);
        require!(q <= t, MathError::InvalidBinState);

        // If q = t, simply return x (coefficient of log term is 0)
        if q == t {
            return Ok(x);
        }
        
        // Calculate directly with precise f64 for more stable calculation
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        let x_f64 = x as f64;
        
        // Check if t-x is 0
        let t_minus_x_f64 = t_f64 - x_f64;
        if t_minus_x_f64 <= 0.0 {
            return Err(error!(MathError::SellCalculationUnderflow));
        }
        
        // Calculate ratio and natural logarithm
        let ratio = t_f64 / t_minus_x_f64;
        let ln_ratio = ratio.ln();
        
        // For q < t case: x - (t-q)*ln(t/(t-x))
        let reduction = (t_f64 - q_f64) * ln_ratio;
        
        // Check to prevent underflow
        let revenue_f64 = if reduction > x_f64 {
            // In extreme cases, return minimum unit 1
            1.0
        } else {
            x_f64 - reduction
        };
        
        // If result is less than 0, return 1 (minimum unit)
        if revenue_f64 <= 0.0 {
            Ok(1)
        } else {
            // Round and convert to u64
            let revenue = (revenue_f64 + 0.5) as u64;
            // Return minimum value 1 if becomes 0
            Ok(if revenue == 0 { 1 } else { revenue })
        }
    }
    
    /// Calculate cost of buying tokens in multiple bins sequentially
    /// @param x Amount of tokens to buy in each bin
    /// @param qs Current token quantities in each bin
    /// @param t Total token quantity in the market
    /// @return Total cost of buying tokens in all bins
    pub fn calculate_multi_bins_buy_cost(x: u64, qs: &[u64], t: u64) -> Result<u64> {
        if qs.is_empty() || x == 0 {
            return Ok(0);
        }
        
        let mut total_cost: u64 = 0;
        let mut current_t = t;
        
        for &q in qs {
            let cost = Self::calculate_bin_buy_cost(x, q, current_t)?;
            total_cost = total_cost.checked_add(cost).ok_or(error!(MathError::MathOverflow))?;
            current_t = current_t.checked_add(x).ok_or(error!(MathError::MathOverflow))?;
        }
        
        Ok(total_cost)
    }
    
    /// Calculate revenue from selling tokens in multiple bins sequentially
    /// @param x Amount of tokens to sell in each bin
    /// @param qs Current token quantities in each bin
    /// @param t Total token quantity in the market
    /// @return Total revenue from selling tokens in all bins
    pub fn calculate_multi_bins_sell_cost(x: u64, qs: &[u64], t: u64) -> Result<u64> {
        if qs.is_empty() || x == 0 {
            return Ok(0);
        }
        
        // Validate inputs for all bins first
        for &q in qs {
            require!(x <= q, MathError::CannotSellMoreThanBin);
        }
        
        let total_x = x.checked_mul(qs.len() as u64).ok_or(error!(MathError::MathOverflow))?;
        require!(total_x <= t, MathError::CannotSellMoreThanSupply);
        
        let mut total_revenue: u64 = 0;
        let mut current_t = t;
        
        for &q in qs {
            let revenue = Self::calculate_bin_sell_cost(x, q, current_t)?;
            total_revenue = total_revenue.checked_add(revenue).ok_or(error!(MathError::MathOverflow))?;
            current_t = current_t.checked_sub(x).ok_or(error!(MathError::MathOverflow))?;
        }
        
        Ok(total_revenue)
    }
    
    /// Calculate token quantity purchasable for a given budget across multiple bins
    /// @param budget Available collateral budget
    /// @param qs Current token quantities in each bin
    /// @param t Total token quantity in the market
    /// @return Purchasable token quantity per bin
    pub fn calculate_x_for_multi_bins(budget: u64, qs: &[u64], t: u64) -> Result<u64> {
        if budget == 0 || qs.is_empty() {
            return Ok(0);
        }
        
        // 연산 시작점 설정: 좌측은 0, 우측은 최대값
        let mut right: u64 = u64::MAX / (qs.len() as u64 + 1);  // 오버플로우 방지를 위한 최대값
        let mut left: u64 = 0;
        
        // 단순한 무한 이진 탐색 (종료 조건으로 제어)
        while right > left + 1 {
            let mid = left + (right - left) / 2;
            
            // Calculate cost of middle value for all bins
            match Self::calculate_multi_bins_buy_cost(mid, qs, t) {
                Ok(calculated_cost) => {
                    // 정확히 예산과 같은 비용을 가진 경우 즉시 반환
                    if calculated_cost == budget {
                        return Ok(mid);
                    }
                    
                    // Adjust search range
                    if calculated_cost < budget {
                        left = mid;
                    } else {
                        right = mid;
                    }
                },
                Err(_) => {
                    // Reduce range if overflow occurs
                    right = mid;
                }
            }
        }
        
        // 이진 탐색이 끝난 후, 최종 left와 right 값의 비용을 계산
        let left_cost = match Self::calculate_multi_bins_buy_cost(left, qs, t) {
            Ok(c) => c,
            Err(_) => 0 // 오류 발생 시 0으로 처리
        };
        
        let right_cost = match Self::calculate_multi_bins_buy_cost(right, qs, t) {
            Ok(c) => c,
            Err(_) => u64::MAX // 오류 발생 시 최대값으로 처리 (선택되지 않도록)
        };
        
        // 예산 이하의 최대 X 값 선택
        // left가 예산 내에 있는 경우
        if left_cost <= budget {
            // right도 예산 내에 있다면 더 큰 right 선택
            if right_cost <= budget {
                return Ok(right);
            }
            // left만 예산 내에 있다면 left 선택
            return Ok(left);
        }
        // right만 예산 내에 있는 경우 (이론적으로는 발생 안함)
        else if right_cost <= budget {
            return Ok(right);
        }
        
        // 어떤 경우에도 적절한 값을 찾지 못한 경우 (매우 작은 예산)
        Ok(0)
    }
}

 