use anchor_lang::prelude::*;
use crate::errors::RangeBetError;

/// Range-Bet Math library
pub struct RangeBetMath;

impl RangeBetMath {
    /// Token purchase cost calculation function
    /// Formula: ∫(q+t)/(T+t) dt = x + (q-T)*ln((T+x)/T)
    /// @param x Amount of tokens to purchase
    /// @param q Current token quantity in the bin
    /// @param t Total token quantity in the market
    /// @return Collateral token cost
    pub fn calculate_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // Error if q is greater than t (impossible state)
        require!(q <= t, RangeBetError::InvalidBinState);

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
    
    /// Calculate token quantity purchasable for a given cost (binary search)
    /// @param cost Available collateral cost
    /// @param q Current token quantity in the bin
    /// @param t Total token quantity in the market
    /// @return Purchasable token quantity
    pub fn calculate_x_for_cost(cost: u64, q: u64, t: u64) -> Result<u64> {
        if cost == 0 {
            return Ok(0);
        }
        if t == 0 {
            return Ok(cost); // First purchase
        }
        
        // Calculate tolerance
        let epsilon_abs: u64 = 10_000;                    // 1e-5 USDC (about $0.00001)
        let epsilon_rel: u64 = cost / 2_000;              // 0.05% relative error
        let epsilon: u64 = epsilon_abs.max(epsilon_rel).max(1); // Ensure minimum 1 lamport
        
        let mut right: u64 = u64::MAX;
        let mut left: u64 = 0;
        
        // Binary search (maximum 32 iterations)
        for _ in 0..32 {
            // Exit if search range is within tolerance
            if right - left <= epsilon {
                break;
            }
            
            let mid = left + (right - left) / 2;
            
            // Calculate cost of middle value
            let calculated_cost = match Self::calculate_cost(mid, q, t) {
                Ok(c) => c,
                Err(_) => {
                    // Reduce range if overflow occurs
                    right = mid;
                    continue;
                }
            };
            
            // Return immediately if calculated cost is within tolerance of target cost
            if (calculated_cost as i128 - cost as i128).abs() as u64 <= epsilon {
                return Ok(mid);
            }
            
            // Adjust search range
            if calculated_cost < cost {
                left = mid;
            } else {
                right = mid;
            }
        }
        
        // Calculate costs for final left and right values, choose the closer one
        let left_cost = match Self::calculate_cost(left, q, t) {
            Ok(c) => c,
            Err(_) => return Ok(right), // Return right if left overflows
        };
        
        let right_cost = match Self::calculate_cost(right, q, t) {
            Ok(c) => c,
            Err(_) => return Ok(left), // Return left if right overflows
        };
        
        // Choose value closer to target cost
        let left_diff = (cost as i128 - left_cost as i128).abs();
        let right_diff = (right_cost as i128 - cost as i128).abs();
        
        if left_diff < right_diff {
            Ok(left)
        } else {
            Ok(right)
        }
    }
    
    /// Calculate revenue from token sales
    /// Formula: ∫(q-t)/(T-t) dt = x + (q-T)*ln(T/(T-x))
    /// @param x Amount of tokens to sell
    /// @param q Current token quantity in the bin
    /// @param t Total token quantity in the market
    /// @return Sale revenue
    pub fn calculate_sell_cost(x: u64, q: u64, t: u64) -> Result<u64> {
        // Input validation
        if x == 0 {
            return Ok(0);
        }
        
        require!(x <= q, RangeBetError::CannotSellMoreThanBin);
        require!(x <= t, RangeBetError::CannotSellMoreThanSupply);
        require!(q <= t, RangeBetError::InvalidBinState);

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
            return Err(error!(RangeBetError::SellCalculationUnderflow));
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
}