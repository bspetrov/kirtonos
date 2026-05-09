pub mod returns {
    pub fn calculate_simple_returns(prices: &[f64]) -> Vec<f64> {
        prices
            .windows(2) // Creates sliding windows of size 2
            .map(|w| (w[1] - w[0]) / w[0]) // (Current - Previous) / Previous
            .collect()
    }

    pub fn calculate_log_returns(prices: &[f64]) -> Vec<f64> {
        prices.windows(2).map(|w| (w[1] / w[0]).ln()).collect()
    }

    pub fn cumulative_return(returns: &[f64]) -> f64 {
        returns.iter().fold(1.0, |acc, r| acc * (1.0 + r)) - 1.0
    }

    pub fn annualized_return(returns: &[f64]) -> f64 {
        let n = returns.len() as f64;
        (1.0 + cumulative_return(returns)).powf(252.0 / n) - 1.0
    }
}

pub mod risk {}

pub mod ratios {}

pub mod diversification {}

pub mod benchmark {}
