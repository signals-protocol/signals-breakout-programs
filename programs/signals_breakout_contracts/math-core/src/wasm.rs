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