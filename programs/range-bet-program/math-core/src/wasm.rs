#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
use super::RangeBetMath;

// 1-bin buy
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateBinBuyCost)]
pub fn calculate_bin_buy_cost(x: u64, q: u64, t: u64) -> u64 {
    RangeBetMath::calculate_bin_buy_cost(x, q, t).unwrap()
}

// 1-bin sell
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateBinSellCost)]
pub fn calculate_bin_sell_cost(x: u64, q: u64, t: u64) -> u64 {
    RangeBetMath::calculate_bin_sell_cost(x, q, t).unwrap()
}

// multi-bin buy
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateMultiBinsBuyCost)]
pub fn calculate_multi_bins_buy_cost(x: u64, qs: Vec<u64>, t: u64) -> u64 {
    RangeBetMath::calculate_multi_bins_buy_cost(x, &qs, t).unwrap()
}

// multi-bin sell
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateMultiBinsSellCost)]
pub fn calculate_multi_bins_sell_cost(x: u64, qs: Vec<u64>, t: u64) -> u64 {
    RangeBetMath::calculate_multi_bins_sell_cost(x, &qs, t).unwrap()
}

// inverse
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateXForMultiBins)]
pub fn calculate_x_for_multi_bins(budget: u64, qs: Vec<u64>, t: u64) -> u64 {
    RangeBetMath::calculate_x_for_multi_bins(budget, &qs, t).unwrap()
} 

// ===== EVM uint256 compatible interfaces (using strings) =====

#[cfg(feature = "wasm")]
fn parse_u64_from_string(s: &str) -> Result<u64, String> {
    s.parse().map_err(|_| format!("Failed to parse '{}' as u64", s))
}

#[cfg(feature = "wasm")]
fn parse_u64_vec_from_string(s: &str) -> Result<Vec<u64>, String> {
    if s.trim().is_empty() {
        return Ok(Vec::new());
    }
    
    let parts: Result<Vec<u64>, _> = s
        .trim_matches(|c| c == '[' || c == ']')
        .split(',')
        .map(|part| part.trim().parse::<u64>())
        .collect();
    
    parts.map_err(|_| format!("Failed to parse '{}' as Vec<u64>", s))
}

// EVM-compatible 1-bin buy (using strings for uint256 compatibility)
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateBinBuyCostEvm)]
pub fn calculate_bin_buy_cost_evm(x_str: &str, q_str: &str, t_str: &str) -> Result<String, String> {
    let x = parse_u64_from_string(x_str)?;
    let q = parse_u64_from_string(q_str)?;
    let t = parse_u64_from_string(t_str)?;
    
    match RangeBetMath::calculate_bin_buy_cost(x, q, t) {
        Ok(result) => Ok(result.to_string()),
        Err(e) => Err(format!("Calculation error: {:?}", e))
    }
}

// EVM-compatible 1-bin sell
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateBinSellCostEvm)]
pub fn calculate_bin_sell_cost_evm(x_str: &str, q_str: &str, t_str: &str) -> Result<String, String> {
    let x = parse_u64_from_string(x_str)?;
    let q = parse_u64_from_string(q_str)?;
    let t = parse_u64_from_string(t_str)?;
    
    match RangeBetMath::calculate_bin_sell_cost(x, q, t) {
        Ok(result) => Ok(result.to_string()),
        Err(e) => Err(format!("Calculation error: {:?}", e))
    }
}

// EVM-compatible multi-bin buy
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateMultiBinsBuyCostEvm)]
pub fn calculate_multi_bins_buy_cost_evm(x_str: &str, qs_str: &str, t_str: &str) -> Result<String, String> {
    let x = parse_u64_from_string(x_str)?;
    let qs = parse_u64_vec_from_string(qs_str)?;
    let t = parse_u64_from_string(t_str)?;
    
    match RangeBetMath::calculate_multi_bins_buy_cost(x, &qs, t) {
        Ok(result) => Ok(result.to_string()),
        Err(e) => Err(format!("Calculation error: {:?}", e))
    }
}

// EVM-compatible multi-bin sell
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateMultiBinsSellCostEvm)]
pub fn calculate_multi_bins_sell_cost_evm(x_str: &str, qs_str: &str, t_str: &str) -> Result<String, String> {
    let x = parse_u64_from_string(x_str)?;
    let qs = parse_u64_vec_from_string(qs_str)?;
    let t = parse_u64_from_string(t_str)?;
    
    match RangeBetMath::calculate_multi_bins_sell_cost(x, &qs, t) {
        Ok(result) => Ok(result.to_string()),
        Err(e) => Err(format!("Calculation error: {:?}", e))
    }
}

// EVM-compatible inverse calculation
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = calculateXForMultiBinsEvm)]
pub fn calculate_x_for_multi_bins_evm(budget_str: &str, qs_str: &str, t_str: &str) -> Result<String, String> {
    let budget = parse_u64_from_string(budget_str)?;
    let qs = parse_u64_vec_from_string(qs_str)?;
    let t = parse_u64_from_string(t_str)?;
    
    match RangeBetMath::calculate_x_for_multi_bins(budget, &qs, t) {
        Ok(result) => Ok(result.to_string()),
        Err(e) => Err(format!("Calculation error: {:?}", e))
    }
}

// ===== Helper functions for EVM integration =====

// Check if a string represents a number within u64 range
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = isWithinU64Range)]
pub fn is_within_u64_range(value_str: &str) -> bool {
    match value_str.parse::<u64>() {
        Ok(_) => true,
        Err(_) => false
    }
}

// Get the maximum u64 value as string (for EVM integration reference)
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = getMaxU64)]
pub fn get_max_u64() -> String {
    u64::MAX.to_string()
}

// Validate if EVM uint256 values fit in u64 range
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = validateEvmValues)]
pub fn validate_evm_values(budget_str: &str, qs_str: &str, t_str: &str) -> Result<String, String> {
    if !is_within_u64_range(budget_str) {
        return Err(format!("Budget '{}' exceeds u64 maximum ({})", budget_str, get_max_u64()));
    }
    
    if !is_within_u64_range(t_str) {
        return Err(format!("Total supply '{}' exceeds u64 maximum ({})", t_str, get_max_u64()));
    }
    
    let qs = parse_u64_vec_from_string(qs_str)?;
    for (i, &q) in qs.iter().enumerate() {
        if q > u64::MAX {
            return Err(format!("Bin quantity at index {} exceeds u64 maximum", i));
        }
    }
    
    Ok("All values are within u64 range".to_string())
} 