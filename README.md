# Sierra Backtest

High-performance futures trading backtesting framework. The performance-critical engine (SCID parsing, bar aggregation, position tracking, metrics) is written in Rust and exposed to Python via PyO3. Strategies are written in Python.

## Features

- **Memory-mapped SCID reader** — reads Sierra Chart binary tick data (40 bytes/tick) without loading entire files into RAM
- **Tick-to-bar aggregation** — 1s, 5s, 10s, 30s, 1m, 5m, 15m, 30m, 1h, 4h, 1d intervals
- **Two backtest modes** — bar-based (vectorized, all bars at once) and tick-based (batched, configurable batch size)
- **Position tracking** — state machine (Flat → Long/Short → Flat) with position flipping support
- **Metrics** — Sharpe ratio, max drawdown, profit factor, win rate, per-side breakdowns, equity curve
- **Reporting** — console summary and equity/drawdown chart generation

## Prerequisites

- Rust toolchain (stable)
- Python >= 3.10 with a virtual environment
- [maturin](https://github.com/PyO3/maturin) (`pip install maturin`)

## Setup

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin numpy matplotlib
maturin develop
```

## Usage

### Running example strategies

```bash
python python/examples/sma_crossover.py
python python/examples/tick_momentum.py
```

### Writing a bar-based strategy

Subclass `Strategy` and implement `on_bars`. Receive a dict of numpy arrays (OHLCV + volume split), return an int32 signal array where `1` = long, `-1` = short, `0` = flat.

```python
from backtest import run_backtest, print_report, plot_equity, Strategy
import numpy as np

class MyStrategy(Strategy):
    def on_bars(self, bars):
        close = np.array(bars["close"])
        n = int(bars["num_bars"])
        signals = np.zeros(n, dtype=np.int32)
        # ... strategy logic ...
        return signals.tolist()

results = run_backtest("data/ESU24_FUT_CME.scid", "5m", MyStrategy().on_bars, commission=2.50, point_value=50.0)
print_report(results)
plot_equity(results, title="My Strategy")
```

### Writing a tick-based strategy

Implement `on_ticks` instead. Tick batches include `price`, `bid`, `ask`, `volume`, `bid_volume`, `ask_volume`, and `timestamp` arrays.

```python
from backtest import run_tick_backtest, print_report, Strategy
import numpy as np

class MyTickStrategy(Strategy):
    def on_bars(self, bars):
        raise NotImplementedError

    def on_ticks(self, ticks):
        n = int(ticks["num_ticks"])
        signals = np.zeros(n, dtype=np.int32)
        # ... strategy logic ...
        return signals.tolist()

results = run_tick_backtest("data/ESU24_FUT_CME.scid", MyTickStrategy().on_ticks, batch_size=100_000, commission=2.50)
print_report(results)
```

### Loading data without backtesting

```python
from backtest import load_scid, load_bars

ticks = load_scid("data/ESU24_FUT_CME.scid")       # raw tick data
bars  = load_bars("data/ESU24_FUT_CME.scid", "1m")  # aggregated bars
```

## API Reference

| Function | Description |
|---|---|
| `run_backtest(path, interval, callback, commission=0.0, point_value=50.0)` | Run bar-based backtest |
| `run_tick_backtest(path, callback, batch_size=100000, commission=0.0, point_value=50.0)` | Run tick-based backtest |
| `load_scid(path)` | Load raw ticks as dict of numpy arrays |
| `load_bars(path, interval)` | Load aggregated bars as dict of numpy arrays |
| `print_report(results)` | Print formatted results to console |
| `plot_equity(results, title, save_path)` | Save equity curve + drawdown chart |

### Backtest results dict

| Key | Type | Description |
|---|---|---|
| `total_pnl` | float | Total profit and loss |
| `num_trades` | int | Total number of round-trip trades |
| `win_rate` | float | Fraction of winning trades |
| `profit_factor` | float | Gross profit / gross loss |
| `sharpe_ratio` | float | Risk-adjusted return |
| `max_drawdown` | float | Largest peak-to-trough decline ($) |
| `max_drawdown_pct` | float | Largest peak-to-trough decline (%) |
| `equity_curve` | numpy array | Cumulative P&L per bar/tick |
| `trades` | list[dict] | Individual trades with entry/exit times, prices, side, P&L |

## Project Structure

```
src/
  lib.rs         PyO3 module — exposes functions to Python
  scid.rs        Memory-mapped SCID binary file reader
  bar.rs         Tick-to-bar aggregation
  engine.rs      Backtest execution (bar and tick modes)
  position.rs    Position state machine and trade recording
  metrics.rs     Performance metrics computation
python/
  backtest/
    __init__.py  Public API
    strategy.py  Abstract Strategy base class
    report.py    Console reporting and chart generation
  examples/
    sma_crossover.py      SMA crossover on 5m bars
    tick_momentum.py       Volume imbalance on raw ticks
data/                      SCID files from Sierra Chart (not included)
```

## Conventions

- **Point values**: ES = 50.0, NQ = 20.0 (dollar multiplier per price point)
- **Commission**: specified per round-trip trade
- **SCID prices**: stored as integers x100, converted to float on read
- **Timestamps**: Unix microseconds internally, Unix seconds in Python
