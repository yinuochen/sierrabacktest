# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Sierra Backtest is a high-performance futures trading backtesting framework. Rust handles the performance-critical engine (SCID parsing, bar aggregation, position tracking, metrics) and exposes a Python API via PyO3. Strategies are written in Python.

## Build & Run

```bash
# Build Rust extension and install into Python env (required after any Rust change)
maturin develop

# Run a strategy
python3 python/examples/sma_crossover.py
python3 python/examples/tick_momentum.py
python3 python/examples/volatility_delta_range.py
```

Virtual environment at `.venv/` (Python 3.14). No formal test suite exists—example strategies serve as validation.

## Architecture

**Rust core** (`src/`):
- `lib.rs` — PyO3 module definition, exposes `run_backtest` and `run_tick_backtest` to Python
- `scid.rs` — Memory-mapped SCID binary file reader (40 bytes/tick, Sierra Chart datetime epoch 1899)
- `bar.rs` — Tick-to-bar aggregation (1s through 1d intervals)
- `engine.rs` — Backtest execution: bar mode (vectorized, all bars at once) and tick mode (batched, default 100k)
- `position.rs` — Position state machine (Flat→Long/Short→Flat), signal-driven, handles flipping
- `metrics.rs` — Sharpe, drawdown, profit factor, win rate, per-side breakdowns

**Python layer** (`python/backtest/`):
- `strategy.py` — Abstract `Strategy` base class with `on_bars(bars) -> np.ndarray` and `on_ticks(ticks) -> np.ndarray`
- `report.py` — `print_report(results)` for console output, `plot_equity(results)` for charts saved to `charts/`
- `__init__.py` — Public API: `run_backtest`, `run_tick_backtest`, `print_report`, `plot_equity`, `Strategy`

## Key Conventions

- **Signals**: int32 arrays where `1`=long, `-1`=short, `0`=flat
- **Point values**: ES=50.0, NQ=20.0 (dollar multiplier per price point)
- **Commission**: per round-trip trade
- **SCID prices**: stored as integers ×100, converted to float on read
- **Timestamps**: Unix microseconds internally, seconds in Python
- **Bar strategies** receive the entire dataset and return all signals at once (vectorized)
- **Tick strategies** process in configurable batches

## Data

`data/` contains SCID binary files from Sierra Chart (multi-GB). These are memory-mapped, not loaded into RAM.
