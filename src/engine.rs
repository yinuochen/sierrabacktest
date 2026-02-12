use numpy::PyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::bar::{aggregate_bars, BarInterval};
use crate::metrics::{compute_metrics, BacktestMetrics};
use crate::position::PositionTracker;
use crate::scid::ScidFile;

/// Run a bar-based backtest. The Python callback receives dict-of-arrays for all bars
/// up to the current index and returns a signal (1=long, -1=short, 0=flat).
pub fn run_bar_backtest(
    py: Python<'_>,
    path: &str,
    interval: &str,
    callback: &Bound<'_, PyAny>,
    commission: f64,
    point_value: f64,
) -> PyResult<BacktestResults> {
    let scid = ScidFile::open(path).map_err(|e| pyo3::exceptions::PyIOError::new_err(e))?;
    let bar_interval =
        BarInterval::from_str(interval).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;

    let bars = aggregate_bars(&scid, bar_interval);
    if bars.is_empty() {
        return Err(pyo3::exceptions::PyValueError::new_err("No bars generated"));
    }

    // Pre-allocate arrays
    let n = bars.len();
    let mut timestamps = Vec::with_capacity(n);
    let mut opens = Vec::with_capacity(n);
    let mut highs = Vec::with_capacity(n);
    let mut lows = Vec::with_capacity(n);
    let mut closes = Vec::with_capacity(n);
    let mut volumes = Vec::with_capacity(n);
    let mut bid_vols = Vec::with_capacity(n);
    let mut ask_vols = Vec::with_capacity(n);

    for bar in &bars {
        timestamps.push(bar.timestamp_us as f64 / 1_000_000.0); // Unix seconds
        opens.push(bar.open);
        highs.push(bar.high);
        lows.push(bar.low);
        closes.push(bar.close);
        volumes.push(bar.volume as f64);
        bid_vols.push(bar.bid_volume as f64);
        ask_vols.push(bar.ask_volume as f64);
    }

    // Convert to numpy arrays
    let ts_arr = PyArray1::from_vec(py, timestamps);
    let open_arr = PyArray1::from_vec(py, opens);
    let high_arr = PyArray1::from_vec(py, highs);
    let low_arr = PyArray1::from_vec(py, lows);
    let close_arr = PyArray1::from_vec(py, closes);
    let vol_arr = PyArray1::from_vec(py, volumes);
    let bid_arr = PyArray1::from_vec(py, bid_vols);
    let ask_arr = PyArray1::from_vec(py, ask_vols);

    // Build a dict of arrays
    let bar_data = PyDict::new(py);
    bar_data.set_item("timestamp", ts_arr)?;
    bar_data.set_item("open", open_arr)?;
    bar_data.set_item("high", high_arr)?;
    bar_data.set_item("low", low_arr)?;
    bar_data.set_item("close", close_arr)?;
    bar_data.set_item("volume", vol_arr)?;
    bar_data.set_item("bid_volume", bid_arr)?;
    bar_data.set_item("ask_volume", ask_arr)?;
    bar_data.set_item("num_bars", n)?;

    // Call the strategy once with all bars — strategy returns signal array
    let result = callback.call1((bar_data,))?;
    let signals: Vec<i32> = result.extract()?;

    if signals.len() != n {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Signal array length {} != bar count {}",
            signals.len(),
            n
        )));
    }

    // Simulate
    let mut tracker = PositionTracker::new(commission, point_value);
    for (i, bar) in bars.iter().enumerate() {
        tracker.process_signal(signals[i], bar.close, bar.timestamp_us);
    }
    // Close any open position at end
    let last = bars.last().unwrap();
    tracker.close_position(last.close, last.timestamp_us);

    let metrics = compute_metrics(&tracker.trades, &tracker.equity_curve);

    Ok(BacktestResults {
        metrics,
        trades: tracker.trades,
        equity_curve: tracker.equity_curve,
    })
}

/// Run a tick-based backtest. Sends batches of ticks to the callback.
pub fn run_tick_backtest(
    py: Python<'_>,
    path: &str,
    batch_size: usize,
    callback: &Bound<'_, PyAny>,
    commission: f64,
    point_value: f64,
) -> PyResult<BacktestResults> {
    let scid = ScidFile::open(path).map_err(|e| pyo3::exceptions::PyIOError::new_err(e))?;

    let mut tracker = PositionTracker::new(commission, point_value);
    let total = scid.num_records;
    let mut offset = 0usize;

    while offset < total {
        let end = (offset + batch_size).min(total);
        let batch_len = end - offset;

        let mut timestamps = Vec::with_capacity(batch_len);
        let mut prices = Vec::with_capacity(batch_len);
        let mut bids = Vec::with_capacity(batch_len);
        let mut asks = Vec::with_capacity(batch_len);
        let mut volumes = Vec::with_capacity(batch_len);
        let mut bid_vols = Vec::with_capacity(batch_len);
        let mut ask_vols = Vec::with_capacity(batch_len);

        for i in offset..end {
            let tick = scid.tick(i);
            if tick.price <= 0.0 {
                continue;
            }
            timestamps.push(tick.timestamp_us as f64 / 1_000_000.0);
            prices.push(tick.price);
            bids.push(tick.bid);
            asks.push(tick.ask);
            volumes.push(tick.volume as f64);
            bid_vols.push(tick.bid_volume as f64);
            ask_vols.push(tick.ask_volume as f64);
        }

        let actual_len = timestamps.len();
        if actual_len == 0 {
            offset = end;
            continue;
        }

        let tick_data = PyDict::new(py);
        tick_data.set_item("timestamp", PyArray1::from_vec(py, timestamps))?;
        tick_data.set_item("price", PyArray1::from_vec(py, prices))?;
        tick_data.set_item("bid", PyArray1::from_vec(py, bids))?;
        tick_data.set_item("ask", PyArray1::from_vec(py, asks))?;
        tick_data.set_item("volume", PyArray1::from_vec(py, volumes))?;
        tick_data.set_item("bid_volume", PyArray1::from_vec(py, bid_vols))?;
        tick_data.set_item("ask_volume", PyArray1::from_vec(py, ask_vols))?;
        tick_data.set_item("num_ticks", actual_len)?;

        let result = callback.call1((tick_data,))?;
        let signals: Vec<i32> = result.extract()?;

        if signals.len() != actual_len {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Signal array length {} != tick batch size {}",
                signals.len(),
                actual_len
            )));
        }

        // Process signals
        let mut tick_idx = 0;
        for i in offset..end {
            let tick = scid.tick(i);
            if tick.price <= 0.0 {
                continue;
            }
            tracker.process_signal(signals[tick_idx], tick.price, tick.timestamp_us);
            tick_idx += 1;
        }

        offset = end;
    }

    // Close any open position
    if total > 0 {
        let last = scid.tick(total - 1);
        tracker.close_position(last.price, last.timestamp_us);
    }

    let metrics = compute_metrics(&tracker.trades, &tracker.equity_curve);

    Ok(BacktestResults {
        metrics,
        trades: tracker.trades,
        equity_curve: tracker.equity_curve,
    })
}

pub struct BacktestResults {
    pub metrics: BacktestMetrics,
    pub trades: Vec<crate::position::Trade>,
    pub equity_curve: Vec<f64>,
}
