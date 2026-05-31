/// Estimate cost for given token counts and per-million-token prices.
/// Returns 0.0 if either price is 0.0.
pub fn estimate_cost(
    input_tokens: u64,
    output_tokens: u64,
    input_token_cost: f64,
    output_token_cost: f64,
) -> f64 {
    let input_cost = input_tokens as f64 * input_token_cost / 1_000_000.0;
    let output_cost = output_tokens as f64 * output_token_cost / 1_000_000.0;
    input_cost + output_cost
}
