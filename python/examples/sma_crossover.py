"""SMA Crossover Strategy — bar-based example.

Goes long when fast SMA > slow SMA, short when fast SMA < slow SMA.
"""
import sys
import os
import numpy as np

# Add parent dir so we can import backtest
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from backtest import run_backtest, print_report, plot_equity, Strategy


class SmaCrossover(Strategy):
    def __init__(self, fast_period=10, slow_period=30):
        self.fast_period = fast_period
        self.slow_period = slow_period

    def on_bars(self, bars):
        close = np.array(bars["close"])
        n = int(bars["num_bars"])
        signals = np.zeros(n, dtype=np.int32)

        # Compute SMAs
        fast_sma = np.full(n, np.nan)
        slow_sma = np.full(n, np.nan)

        # Cumulative sum for efficient SMA
        cumsum = np.cumsum(close)
        for i in range(self.fast_period - 1, n):
            if i < self.fast_period:
                fast_sma[i] = cumsum[i] / self.fast_period
            else:
                fast_sma[i] = (cumsum[i] - cumsum[i - self.fast_period]) / self.fast_period

        for i in range(self.slow_period - 1, n):
            if i < self.slow_period:
                slow_sma[i] = cumsum[i] / self.slow_period
            else:
                slow_sma[i] = (cumsum[i] - cumsum[i - self.slow_period]) / self.slow_period

        # Generate signals
        for i in range(self.slow_period, n):
            if fast_sma[i] > slow_sma[i]:
                signals[i] = 1  # Long
            elif fast_sma[i] < slow_sma[i]:
                signals[i] = -1  # Short

        return signals.tolist()


def main():
    scid_path = os.path.join(os.path.dirname(__file__), "..", "..", "data", "ESU24_FUT_CME.scid")
    scid_path = os.path.abspath(scid_path)

    if not os.path.exists(scid_path):
        print(f"SCID file not found: {scid_path}")
        sys.exit(1)

    strategy = SmaCrossover(fast_period=10, slow_period=30)

    print(f"Running SMA Crossover backtest on 5m bars...")
    print(f"  Fast SMA: {strategy.fast_period}, Slow SMA: {strategy.slow_period}")
    print(f"  Data: {scid_path}")
    print()

    results = run_backtest(scid_path, "5m", strategy.on_bars, commission=2.50)

    print_report(results)
    plot_equity(results, title="SMA Crossover (5m bars, 10/30)")


if __name__ == "__main__":
    main()
