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
    #[msg("Can only sell entire supply if bin contains all tokens")]
    CanOnlySellEntireSupplyIfBinContainsAllTokens,
    #[msg("Underflow in sell calculation")]
    UnderflowInSellCalculation,
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
        // 1) Edge cases
        if x == 0 {
            return Ok(0); // If sell amount is 0, return 0 revenue
        }
        
        // Domain validation: q <= t first
        require!(q <= t, MathError::InvalidBinState);
        
        // Special case: If x == t, we're selling the entire market supply
        // This can only happen if q == t (the bin contains all tokens)
        if x == t {
            require!(q == t, MathError::CanOnlySellEntireSupplyIfBinContainsAllTokens);
            return Ok(t); // When selling all tokens, return the total supply value
        }
        
        // Domain validation: x <= q (after x == t check)
        require!(x <= q, MathError::CannotSellMoreThanBin);

        // 2) If q = t, simply return x (coefficient of log term is 0)
        if q == t {
            return Ok(x);
        }
        
        // 3) Calculate directly with precise f64 for more stable calculation
        let q_f64 = q as f64;
        let t_f64 = t as f64;
        let x_f64 = x as f64;
        
        // 4) Calculate ( T / (T - x) )
        let t_minus_x_f64 = t_f64 - x_f64;
        if t_minus_x_f64 <= 0.0 {
            return Err(error!(MathError::SellCalculationUnderflow));
        }
        
        let ratio = t_f64 / t_minus_x_f64;
        let ln_ratio = ratio.ln();
        
        // 5) Calculate x + (q - T)*ln( T / (T - x) )
        let mut revenue_f64 = x_f64;
        
        if q != t {
            // Since T >= q, we know (q - T) <= 0, so we subtract
            let t_minus_q = t_f64 - q_f64;
            let reduction = t_minus_q * ln_ratio;
            
            // Prevent underflow - if the log term is too large compared to x, error
            require!(reduction <= revenue_f64, MathError::UnderflowInSellCalculation);
            
            revenue_f64 = revenue_f64 - reduction;
        }
        // If q == T, the (q - T) term is 0, so revenue = x

        // Round and convert to u64
        let revenue = (revenue_f64 + 0.5) as u64;
        // Ensure minimum value of 1 when x > 0 (to prevent 0 revenue for valid sales)
        Ok(if revenue == 0 && x > 0 { 1 } else { revenue })
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
        
        // Start with a conservative estimate and expand systematically
        let mut left: u64 = 0;
        let mut right: u64 = 1;
        
        // Find a reasonable upper bound by doubling until we exceed budget
        loop {
            match Self::calculate_multi_bins_buy_cost(right, qs, t) {
                Ok(cost) if cost <= budget => {
                    left = right;
                    if right > budget / 2 { // Prevent excessive expansion
                        right = right.saturating_add(budget);
                    } else {
                        right = right.saturating_mul(2);
                    }
                    if right == left { // Overflow protection
                        break;
                    }
                },
                _ => break, // Found upper bound or overflow
            }
        }
        
        // Now we have left (valid) and right (invalid or overflow)
        // Perform precise binary search
        while left + 1 < right {
            let mid = left + (right - left) / 2;
            
            match Self::calculate_multi_bins_buy_cost(mid, qs, t) {
                Ok(calculated_cost) => {
                    if calculated_cost <= budget {
                        left = mid; // mid is valid
                    } else {
                        right = mid; // mid is too expensive
                    }
                },
                Err(_) => {
                    right = mid; // Overflow, mid is too large
                }
            }
        }
        
        // Final validation - ensure the result doesn't exceed budget
        match Self::calculate_multi_bins_buy_cost(left, qs, t) {
            Ok(cost) if cost <= budget => Ok(left),
            _ => Ok(0),
        }
}
}


 