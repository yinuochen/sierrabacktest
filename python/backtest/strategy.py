from abc import ABC, abstractmethod
import numpy as np


class Strategy(ABC):
    """Base class for backtest strategies.

    Subclass and implement on_bars() for bar-based strategies
    or on_ticks() for tick-based strategies.
    """

    @abstractmethod
    def on_bars(self, bars: dict) -> np.ndarray:
        """Called with all bars as a dict of numpy arrays.

        Keys: timestamp, open, high, low, close, volume, bid_volume, ask_volume, num_bars

        Must return an int32 array of length num_bars with signals:
            1 = long, -1 = short, 0 = flat
        """
        raise NotImplementedError

    def on_ticks(self, ticks: dict) -> np.ndarray:
        """Called with a batch of ticks as a dict of numpy arrays.

        Keys: timestamp, price, bid, ask, volume, bid_volume, ask_volume, num_ticks

        Must return an int32 array of length num_ticks with signals:
            1 = long, -1 = short, 0 = flat
        """
        raise NotImplementedError("Tick strategy not implemented")
