//! Token 计费系统

/// 价格表
pub struct PricingTable;

impl PricingTable {
    /// 计算费用
    pub fn calculate_cost(
        provider: &str,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> f64 {
        let (input_price, output_price) = Self::get_prices(provider, model);
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_price;
        input_cost + output_cost
    }

    fn get_prices(provider: &str, model: &str) -> (f64, f64) {
        match provider {
            "openai" => match model {
                "gpt-4o" => (2.50, 10.00),
                "gpt-4o-mini" => (0.15, 0.60),
                _ => (0.0, 0.0),
            },
            "openrouter" => {
                if model.contains("free") {
                    (0.0, 0.0)
                } else {
                    (0.50, 0.50)
                }
            }
            "ollama" | "lmstudio" => (0.0, 0.0),
            _ => (0.0, 0.0),
        }
    }
}
