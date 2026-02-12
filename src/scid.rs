use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

const HEADER_SIZE: usize = 56;
const RECORD_SIZE: usize = 40;
/// Microseconds between 1899-12-30 and 1970-01-01 (Unix epoch).
const EPOCH_OFFSET_US: i64 = 2_209_161_600_000_000;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct RawScidRecord {
    pub sc_datetime: i64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub num_trades: u32,
    pub total_volume: u32,
    pub bid_volume: u32,
    pub ask_volume: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Tick {
    /// Unix timestamp in microseconds
    pub timestamp_us: i64,
    pub price: f64,
    pub bid: f64,
    pub ask: f64,
    pub volume: u32,
    pub bid_volume: u32,
    pub ask_volume: u32,
    pub num_trades: u32,
}

pub struct ScidFile {
    _mmap: Mmap,
    ptr: *const u8,
    pub num_records: usize,
}

// Safety: the mmap is read-only and lives as long as ScidFile
unsafe impl Send for ScidFile {}
unsafe impl Sync for ScidFile {}

impl ScidFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let file = File::open(path.as_ref()).map_err(|e| format!("open: {e}"))?;
        let mmap = unsafe { Mmap::map(&file) }.map_err(|e| format!("mmap: {e}"))?;

        // Validate header
        if mmap.len() < HEADER_SIZE {
            return Err("File too small for SCID header".into());
        }
        if &mmap[0..4] != b"SCID" {
            return Err("Invalid SCID magic bytes".into());
        }

        let data_len = mmap.len() - HEADER_SIZE;
        if data_len % RECORD_SIZE != 0 {
            return Err(format!(
                "Data length {data_len} not divisible by record size {RECORD_SIZE}"
            ));
        }
        let num_records = data_len / RECORD_SIZE;
        let ptr = mmap.as_ptr();

        Ok(ScidFile {
            _mmap: mmap,
            ptr,
            num_records,
        })
    }

    #[inline]
    pub fn raw_record(&self, index: usize) -> &RawScidRecord {
        debug_assert!(index < self.num_records);
        unsafe {
            let offset = HEADER_SIZE + index * RECORD_SIZE;
            &*(self.ptr.add(offset) as *const RawScidRecord)
        }
    }

    #[inline]
    pub fn tick(&self, index: usize) -> Tick {
        let r = self.raw_record(index);
        let sc_dt = r.sc_datetime;
        let close = r.close;
        let high = r.high;
        let low = r.low;
        let total_volume = r.total_volume;
        let bid_volume = r.bid_volume;
        let ask_volume = r.ask_volume;
        let num_trades = r.num_trades;
        Tick {
            timestamp_us: sc_dt - EPOCH_OFFSET_US,
            price: close as f64 / 100.0,
            bid: low as f64 / 100.0,
            ask: high as f64 / 100.0,
            volume: total_volume,
            bid_volume,
            ask_volume,
            num_trades,
        }
    }

    pub fn ticks(&self) -> Vec<Tick> {
        (0..self.num_records).map(|i| self.tick(i)).collect()
    }
}
