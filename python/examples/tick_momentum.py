"""Tick Momentum Strategy — volume imbalance based.

Goes long when cumulative bid/ask volume imbalance exceeds a threshold,
short when it goes below the negative threshold, flat otherwise.
"""
import sys
import os
import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from backtest import run_tick_backtest, print_report, plot_equity, Strategy


class TickMomentum(Strategy):
    def __init__(self, lookback=5000, threshold=0.1):
        self.lookback = lookback
        self.threshold = threshold
        self._cum_bid = 0.0
        self._cum_ask = 0.0

    def on_bars(self, bars):
        raise NotImplementedError("This is a tick-based strategy")

    def on_ticks(self, ticks):
        bid_vol = np.array(ticks["bid_volume"])
        ask_vol = np.array(ticks["ask_volume"])
        n = int(ticks["num_ticks"])
        signals = np.zeros(n, dtype=np.int32)

        # Rolling volume imbalance
        for i in range(n):
            self._cum_bid += bid_vol[i]
            self._cum_ask += ask_vol[i]

            total = self._cum_bid + self._cum_ask
            if total > 0:
                imbalance = (self._cum_bid - self._cum_ask) / total
            else:
                imbalance = 0.0

            if imbalance > self.threshold:
                signals[i] = 1   # Buyers dominant → long
            elif imbalance < -self.threshold:
                signals[i] = -1  # Sellers dominant → short
            else:
                signals[i] = 0

            # Decay: reset accumulators periodically
            if (i + 1) % self.lookback == 0:
                self._cum_bid = 0.0
                self._cum_ask = 0.0

        return signals.tolist()


def main():
    scid_path = os.path.join(os.path.dirname(__file__), "..", "..", "data", "ESU24_FUT_CME.scid")
    scid_path = os.path.abspath(scid_path)

    if not os.path.exists(scid_path):
        print(f"SCID file not found: {scid_path}")
        sys.exit(1)

    strategy = TickMomentum(lookback=5000, threshold=0.1)

    print(f"Running Tick Momentum backtest...")
    print(f"  Lookback: {strategy.lookback}, Threshold: {strategy.threshold}")
    print(f"  Data: {scid_path}")
    print()

    results = run_tick_backtest(
        scid_path,
        strategy.on_ticks,
        batch_size=100_000,
        commission=2.50,
    )

    print_report(results)
    plot_equity(results, title="Tick Momentum (Volume Imbalance)")


if __name__ == "__main__":
    main()
