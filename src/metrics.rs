use crate::position::{Side, Trade};

#[derive(Clone, Debug)]
pub struct BacktestMetrics {
    pub total_pnl: f64,
    pub num_trades: usize,
    pub num_wins: usize,
    pub num_losses: usize,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub max_drawdown: f64,
    pub max_drawdown_pct: f64,
    pub sharpe_ratio: f64,
    pub avg_holding_time_secs: f64,
    pub num_long: usize,
    pub num_short: usize,
}

pub fn compute_metrics(trades: &[Trade], equity_curve: &[f64]) -> BacktestMetrics {
    let num_trades = trades.len();
    if num_trades == 0 {
        return BacktestMetrics {
            total_pnl: 0.0,
            num_trades: 0,
            num_wins: 0,
            num_losses: 0,
            win_rate: 0.0,
            profit_factor: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            max_drawdown: 0.0,
            max_drawdown_pct: 0.0,
            sharpe_ratio: 0.0,
            avg_holding_time_secs: 0.0,
            num_long: 0,
            num_short: 0,
        };
    }

    let total_pnl: f64 = trades.iter().map(|t| t.pnl).sum();
    let mut gross_profit = 0.0_f64;
    let mut gross_loss = 0.0_f64;
    let mut num_wins = 0usize;
    let mut num_losses = 0usize;
    let mut largest_win = 0.0_f64;
    let mut largest_loss = 0.0_f64;
    let mut total_holding_us = 0i64;
    let mut num_long = 0usize;
    let mut num_short = 0usize;

    for t in trades {
        if t.pnl > 0.0 {
            num_wins += 1;
            gross_profit += t.pnl;
            if t.pnl > largest_win {
                largest_win = t.pnl;
            }
        } else if t.pnl < 0.0 {
            num_losses += 1;
            gross_loss += t.pnl.abs();
            if t.pnl < largest_loss {
                largest_loss = t.pnl;
            }
        }
        total_holding_us += t.exit_time_us - t.entry_time_us;
        match t.side {
            Side::Long => num_long += 1,
            Side::Short => num_short += 1,
            _ => {}
        }
    }

    let win_rate = num_wins as f64 / num_trades as f64;
    let profit_factor = if gross_loss > 0.0 {
        gross_profit / gross_loss
    } else if gross_profit > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };
    let avg_win = if num_wins > 0 { gross_profit / num_wins as f64 } else { 0.0 };
    let avg_loss = if num_losses > 0 { -(gross_loss / num_losses as f64) } else { 0.0 };
    let avg_holding_time_secs = (total_holding_us as f64 / num_trades as f64) / 1_000_000.0;

    // Max drawdown from equity curve
    let (max_drawdown, max_drawdown_pct) = calc_max_drawdown(equity_curve);

    // Sharpe ratio from per-trade returns
    let trade_pnls: Vec<f64> = trades.iter().map(|t| t.pnl).collect();
    let sharpe_ratio = calc_sharpe(&trade_pnls);

    BacktestMetrics {
        total_pnl,
        num_trades,
        num_wins,
        num_losses,
        win_rate,
        profit_factor,
        avg_win,
        avg_loss,
        largest_win,
        largest_loss,
        max_drawdown,
        max_drawdown_pct,
        sharpe_ratio,
        avg_holding_time_secs,
        num_long,
        num_short,
    }
}

fn calc_max_drawdown(equity: &[f64]) -> (f64, f64) {
    if equity.is_empty() {
        return (0.0, 0.0);
    }
    let mut peak = equity[0];
    let mut max_dd = 0.0_f64;
    let mut max_dd_pct = 0.0_f64;

    for &eq in equity {
        if eq > peak {
            peak = eq;
        }
        let dd = peak - eq;
        if dd > max_dd {
            max_dd = dd;
        }
        if peak > 0.0 {
            let dd_pct = dd / peak;
            if dd_pct > max_dd_pct {
                max_dd_pct = dd_pct;
            }
        }
    }
    (max_dd, max_dd_pct * 100.0)
}

fn calc_sharpe(pnls: &[f64]) -> f64 {
    if pnls.len() < 2 {
        return 0.0;
    }
    let n = pnls.len() as f64;
    let mean = pnls.iter().sum::<f64>() / n;
    let var = pnls.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = var.sqrt();
    if std == 0.0 {
        return 0.0;
    }
    // Annualize: assume ~252 trading days, ~20 trades/day as rough approximation
    // Or just use sqrt(n) for total-period normalization
    (mean / std) * (252.0_f64).sqrt()
}
