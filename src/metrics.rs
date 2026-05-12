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

pub mod risk {
    pub fn volatility(log_returns: &[f64]) -> f64 {
        let n = log_returns.len() as f64;
        let mean = log_returns.iter().sum::<f64>() / n;
        let variance = log_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt() * 252.0_f64.sqrt()
    }

    pub fn max_drawdown(prices: &[f64]) -> f64 {
        prices
            .iter()
            .fold((f64::NEG_INFINITY, 0.0_f64), |(peak, max_dd), &price| {
                let new_peak = peak.max(price);
                let dd = (price - new_peak) / new_peak;
                (new_peak, max_dd.min(dd))
            })
            .1
    }
}

pub mod ratios {
    pub fn sharpe(annualized_return: f64, volatility: f64, risk_free_rate: f64) -> f64 {
        (annualized_return - risk_free_rate) / volatility
    }

    pub fn sortino(annualized_return: f64, log_returns: &[f64], risk_free_rate: f64) -> f64 {
        let n = log_returns.len() as f64;
        let downside_variance = log_returns
            .iter()
            .map(|&r| if r < 0.0 { r.powi(2) } else { 0.0 })
            .sum::<f64>()
            / n;
        let downside_vol = downside_variance.sqrt() * 252.0_f64.sqrt();
        (annualized_return - risk_free_rate) / downside_vol
    }

    pub fn calmar(annualized_return: f64, max_drawdown: f64) -> f64 {
        annualized_return / max_drawdown.abs()
    }
}

pub mod diversification {}

pub mod benchmark {}
