mod bar;
mod engine;
mod metrics;
mod position;
mod scid;

use numpy::PyArray1;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use bar::{aggregate_bars, BarInterval};
use scid::ScidFile;

/// Load raw ticks from an SCID file. Returns a dict of numpy arrays.
#[pyfunction]
fn load_scid(py: Python<'_>, path: &str) -> PyResult<Py<PyDict>> {
    let scid = ScidFile::open(path).map_err(|e| pyo3::exceptions::PyIOError::new_err(e))?;
    let n = scid.num_records;

    let mut timestamps = Vec::with_capacity(n);
    let mut prices = Vec::with_capacity(n);
    let mut bids = Vec::with_capacity(n);
    let mut asks = Vec::with_capacity(n);
    let mut volumes = Vec::with_capacity(n);
    let mut bid_vols = Vec::with_capacity(n);
    let mut ask_vols = Vec::with_capacity(n);
    let mut num_trades = Vec::with_capacity(n);

    for i in 0..n {
        let tick = scid.tick(i);
        timestamps.push(tick.timestamp_us as f64 / 1_000_000.0);
        prices.push(tick.price);
        bids.push(tick.bid);
        asks.push(tick.ask);
        volumes.push(tick.volume as f64);
        bid_vols.push(tick.bid_volume as f64);
        ask_vols.push(tick.ask_volume as f64);
        num_trades.push(tick.num_trades as f64);
    }

    let d = PyDict::new(py);
    d.set_item("timestamp", PyArray1::from_vec(py, timestamps))?;
    d.set_item("price", PyArray1::from_vec(py, prices))?;
    d.set_item("bid", PyArray1::from_vec(py, bids))?;
    d.set_item("ask", PyArray1::from_vec(py, asks))?;
    d.set_item("volume", PyArray1::from_vec(py, volumes))?;
    d.set_item("bid_volume", PyArray1::from_vec(py, bid_vols))?;
    d.set_item("ask_volume", PyArray1::from_vec(py, ask_vols))?;
    d.set_item("num_trades", PyArray1::from_vec(py, num_trades))?;
    d.set_item("num_records", n)?;

    Ok(d.into())
}

/// Load SCID data aggregated into bars. Returns dict of numpy arrays.
#[pyfunction]
fn load_bars(py: Python<'_>, path: &str, interval: &str) -> PyResult<Py<PyDict>> {
    let scid = ScidFile::open(path).map_err(|e| pyo3::exceptions::PyIOError::new_err(e))?;
    let bar_interval =
        BarInterval::from_str(interval).map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?;
    let bars = aggregate_bars(&scid, bar_interval);

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
        timestamps.push(bar.timestamp_us as f64 / 1_000_000.0);
        opens.push(bar.open);
        highs.push(bar.high);
        lows.push(bar.low);
        closes.push(bar.close);
        volumes.push(bar.volume as f64);
        bid_vols.push(bar.bid_volume as f64);
        ask_vols.push(bar.ask_volume as f64);
    }

    let d = PyDict::new(py);
    d.set_item("timestamp", PyArray1::from_vec(py, timestamps))?;
    d.set_item("open", PyArray1::from_vec(py, opens))?;
    d.set_item("high", PyArray1::from_vec(py, highs))?;
    d.set_item("low", PyArray1::from_vec(py, lows))?;
    d.set_item("close", PyArray1::from_vec(py, closes))?;
    d.set_item("volume", PyArray1::from_vec(py, volumes))?;
    d.set_item("bid_volume", PyArray1::from_vec(py, bid_vols))?;
    d.set_item("ask_volume", PyArray1::from_vec(py, ask_vols))?;
    d.set_item("num_bars", n)?;

    Ok(d.into())
}

/// Run a bar-based backtest with a Python strategy callback.
/// point_value: dollar value per 1.0 point move (ES=50, NQ=20)
#[pyfunction]
#[pyo3(signature = (path, interval, callback, commission=0.0, point_value=50.0))]
fn run_backtest(
    py: Python<'_>,
    path: &str,
    interval: &str,
    callback: &Bound<'_, PyAny>,
    commission: f64,
    point_value: f64,
) -> PyResult<Py<PyDict>> {
    let results = engine::run_bar_backtest(py, path, interval, callback, commission, point_value)?;
    results_to_dict(py, results)
}

/// Run a tick-based backtest with a Python strategy callback.
/// point_value: dollar value per 1.0 point move (ES=50, NQ=20)
#[pyfunction]
#[pyo3(signature = (path, callback, batch_size=100000, commission=0.0, point_value=50.0))]
fn run_tick_backtest(
    py: Python<'_>,
    path: &str,
    callback: &Bound<'_, PyAny>,
    batch_size: usize,
    commission: f64,
    point_value: f64,
) -> PyResult<Py<PyDict>> {
    let results =
        engine::run_tick_backtest(py, path, batch_size, callback, commission, point_value)?;
    results_to_dict(py, results)
}

fn results_to_dict(py: Python<'_>, results: engine::BacktestResults) -> PyResult<Py<PyDict>> {
    let m = &results.metrics;
    let d = PyDict::new(py);
    d.set_item("total_pnl", m.total_pnl)?;
    d.set_item("num_trades", m.num_trades)?;
    d.set_item("num_wins", m.num_wins)?;
    d.set_item("num_losses", m.num_losses)?;
    d.set_item("win_rate", m.win_rate)?;
    d.set_item("profit_factor", m.profit_factor)?;
    d.set_item("avg_win", m.avg_win)?;
    d.set_item("avg_loss", m.avg_loss)?;
    d.set_item("largest_win", m.largest_win)?;
    d.set_item("largest_loss", m.largest_loss)?;
    d.set_item("max_drawdown", m.max_drawdown)?;
    d.set_item("max_drawdown_pct", m.max_drawdown_pct)?;
    d.set_item("sharpe_ratio", m.sharpe_ratio)?;
    d.set_item("avg_holding_time_secs", m.avg_holding_time_secs)?;
    d.set_item("num_long", m.num_long)?;
    d.set_item("num_short", m.num_short)?;
    d.set_item(
        "equity_curve",
        PyArray1::from_vec(py, results.equity_curve),
    )?;

    // Trade list
    let trades: Vec<Py<PyDict>> = results
        .trades
        .iter()
        .map(|t| {
            let td = PyDict::new(py);
            td.set_item("entry_time", t.entry_time_us as f64 / 1_000_000.0)
                .unwrap();
            td.set_item("exit_time", t.exit_time_us as f64 / 1_000_000.0)
                .unwrap();
            td.set_item(
                "side",
                match t.side {
                    position::Side::Long => "long",
                    position::Side::Short => "short",
                    position::Side::Flat => "flat",
                },
            )
            .unwrap();
            td.set_item("entry_price", t.entry_price).unwrap();
            td.set_item("exit_price", t.exit_price).unwrap();
            td.set_item("pnl", t.pnl).unwrap();
            td.into()
        })
        .collect();
    d.set_item("trades", trades)?;

    Ok(d.into())
}

/// PyO3 module
#[pymodule]
fn _engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load_scid, m)?)?;
    m.add_function(wrap_pyfunction!(load_bars, m)?)?;
    m.add_function(wrap_pyfunction!(run_backtest, m)?)?;
    m.add_function(wrap_pyfunction!(run_tick_backtest, m)?)?;
    Ok(())
}
