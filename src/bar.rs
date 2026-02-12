use crate::scid::{ScidFile, Tick};

#[derive(Clone, Copy, Debug)]
pub struct Bar {
    /// Bar open timestamp (Unix microseconds)
    pub timestamp_us: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub bid_volume: u64,
    pub ask_volume: u64,
    pub num_trades: u64,
}

/// Bar interval in seconds.
#[derive(Clone, Copy, Debug)]
pub struct BarInterval(pub u64);

impl BarInterval {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "1s" => Ok(BarInterval(1)),
            "5s" => Ok(BarInterval(5)),
            "10s" => Ok(BarInterval(10)),
            "30s" => Ok(BarInterval(30)),
            "1m" => Ok(BarInterval(60)),
            "3m" => Ok(BarInterval(180)),
            "5m" => Ok(BarInterval(300)),
            "15m" => Ok(BarInterval(900)),
            "30m" => Ok(BarInterval(1800)),
            "1h" => Ok(BarInterval(3600)),
            "4h" => Ok(BarInterval(14400)),
            "1d" => Ok(BarInterval(86400)),
            _ => Err(format!("Unknown interval: {s}")),
        }
    }

    /// Return the bar boundary (start of the bar) for a given unix-us timestamp.
    #[inline]
    pub fn bar_start(&self, timestamp_us: i64) -> i64 {
        let secs = timestamp_us / 1_000_000;
        let bar_secs = secs - (secs % self.0 as i64);
        bar_secs * 1_000_000
    }
}

pub fn aggregate_bars(scid: &ScidFile, interval: BarInterval) -> Vec<Bar> {
    if scid.num_records == 0 {
        return Vec::new();
    }

    let mut bars: Vec<Bar> = Vec::with_capacity(scid.num_records / 100);
    let mut current_bar_start: i64 = i64::MIN;
    let mut bar = Bar {
        timestamp_us: 0,
        open: 0.0,
        high: f64::MIN,
        low: f64::MAX,
        close: 0.0,
        volume: 0,
        bid_volume: 0,
        ask_volume: 0,
        num_trades: 0,
    };

    for i in 0..scid.num_records {
        let tick: Tick = scid.tick(i);
        if tick.price <= 0.0 {
            continue;
        }
        let bs = interval.bar_start(tick.timestamp_us);

        if bs != current_bar_start {
            if current_bar_start != i64::MIN {
                bars.push(bar);
            }
            current_bar_start = bs;
            bar = Bar {
                timestamp_us: bs,
                open: tick.price,
                high: tick.price,
                low: tick.price,
                close: tick.price,
                volume: tick.volume as u64,
                bid_volume: tick.bid_volume as u64,
                ask_volume: tick.ask_volume as u64,
                num_trades: tick.num_trades as u64,
            };
        } else {
            if tick.price > bar.high {
                bar.high = tick.price;
            }
            if tick.price < bar.low {
                bar.low = tick.price;
            }
            bar.close = tick.price;
            bar.volume += tick.volume as u64;
            bar.bid_volume += tick.bid_volume as u64;
            bar.ask_volume += tick.ask_volume as u64;
            bar.num_trades += tick.num_trades as u64;
        }
    }
    // Push the last bar
    if current_bar_start != i64::MIN {
        bars.push(bar);
    }
    bars
}
