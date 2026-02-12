#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Flat,
    Long,
    Short,
}

#[derive(Clone, Debug)]
pub struct Trade {
    pub entry_time_us: i64,
    pub exit_time_us: i64,
    pub side: Side,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
}

#[derive(Clone, Debug)]
pub struct PositionTracker {
    pub side: Side,
    pub entry_price: f64,
    pub entry_time_us: i64,
    pub commission: f64,
    pub point_value: f64,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub running_pnl: f64,
}

impl PositionTracker {
    pub fn new(commission: f64, point_value: f64) -> Self {
        PositionTracker {
            side: Side::Flat,
            entry_price: 0.0,
            entry_time_us: 0,
            commission,
            point_value,
            trades: Vec::new(),
            equity_curve: Vec::new(),
            running_pnl: 0.0,
        }
    }

    /// Process a signal at the given price and time.
    /// signal: 1 = long, -1 = short, 0 = flat
    pub fn process_signal(&mut self, signal: i32, price: f64, timestamp_us: i64) {
        let desired = match signal {
            1 => Side::Long,
            -1 => Side::Short,
            _ => Side::Flat,
        };

        if desired == self.side {
            // No change
            self.equity_curve.push(self.running_pnl + self.unrealized_pnl(price));
            return;
        }

        // Close current position if not flat
        if self.side != Side::Flat {
            let pnl = self.calc_pnl(price) - self.commission;
            self.running_pnl += pnl;
            self.trades.push(Trade {
                entry_time_us: self.entry_time_us,
                exit_time_us: timestamp_us,
                side: self.side,
                entry_price: self.entry_price,
                exit_price: price,
                pnl,
            });
            self.side = Side::Flat;
        }

        // Open new position if not flat
        if desired != Side::Flat {
            self.side = desired;
            self.entry_price = price;
            self.entry_time_us = timestamp_us;
        }

        self.equity_curve.push(self.running_pnl);
    }

    fn calc_pnl(&self, exit_price: f64) -> f64 {
        let diff = exit_price - self.entry_price;
        match self.side {
            Side::Long => diff * self.point_value,
            Side::Short => -diff * self.point_value,
            Side::Flat => 0.0,
        }
    }

    fn unrealized_pnl(&self, current_price: f64) -> f64 {
        self.calc_pnl(current_price)
    }

    /// Force-close any open position at the given price/time.
    pub fn close_position(&mut self, price: f64, timestamp_us: i64) {
        if self.side != Side::Flat {
            self.process_signal(0, price, timestamp_us);
        }
    }
}
