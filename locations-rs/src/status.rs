use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{info, Instrument};
use maps::tiles::Tile;
use async_channel::Sender;
use tokio::time::Duration;

const LOG_INTERVAL_SECS: u64 = 5;

pub struct ProgramStatus {
    counters: RwLock<StatusCounters>,
    last_tile: RwLock<Arc<Tile>>,
}

#[derive(Default)]
struct StatusCounters {
    request_count: AtomicUsize,
    ratelimit_count: AtomicUsize,
    error_count: AtomicUsize,
    found_count: AtomicUsize,
    failed_count: AtomicUsize,
}

#[derive(Debug, Clone, Copy)]
pub enum CounterType {
    Request,
    Ratelimit,
    Error,
    Found,
    Failed,
}

impl ProgramStatus {

    fn default() -> Self {
        Self {
            counters: RwLock::new(StatusCounters::default()),
            last_tile: RwLock::new(Arc::new(maps::tiles::Tile{x: 0, y:0, zoom: 0})),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment(&self, counter: CounterType) {
        let counters = self.counters.read();
        match counter {
            CounterType::Request => counters.request_count.fetch_add(1, Ordering::Relaxed),
            CounterType::Ratelimit => counters.ratelimit_count.fetch_add(1, Ordering::Relaxed),
            CounterType::Error => counters.error_count.fetch_add(1, Ordering::Relaxed),
            CounterType::Found => counters.found_count.fetch_add(1, Ordering::Relaxed),
            CounterType::Failed => counters.failed_count.fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn increment_count(&self, counter: CounterType, count: usize) {
        let counters = self.counters.read();
        match counter {
            CounterType::Request => counters.request_count.fetch_add(count, Ordering::Relaxed),
            CounterType::Ratelimit => counters.ratelimit_count.fetch_add(count, Ordering::Relaxed),
            CounterType::Error => counters.error_count.fetch_add(count, Ordering::Relaxed),
            CounterType::Found => counters.found_count.fetch_add(count, Ordering::Relaxed),
            CounterType::Failed => counters.failed_count.fetch_add(count, Ordering::Relaxed),
        };
    }

    pub fn update_last_tile(&self, input: Tile) {
        *self.last_tile.write() = Arc::from(input);
    }

    pub fn get_and_reset(&self, counter: CounterType) -> usize {
        let counters = self.counters.read();
        match counter {
            CounterType::Request => counters.request_count.swap(0, Ordering::Relaxed),
            CounterType::Ratelimit => counters.ratelimit_count.swap(0, Ordering::Relaxed),
            CounterType::Error => counters.error_count.swap(0, Ordering::Relaxed),
            _ => panic!("Unsupported counter type for reset"),
        }
    }

    pub fn get(&self, counter: CounterType) -> usize {
        let counters = self.counters.read();
        match counter {
            CounterType::Found => counters.found_count.load(Ordering::Relaxed),
            CounterType::Failed => counters.failed_count.load(Ordering::Relaxed),
            _ => panic!("Unsupported counter type for get"),
        }
    }

    pub fn get_last_input(&self) -> Arc<Tile> {
        self.last_tile.read().clone()
    }

    fn get_metrics(&self) -> StatusMetrics {
        StatusMetrics {
            requests: self.get_and_reset(CounterType::Request),
            ratelimited: self.get_and_reset(CounterType::Ratelimit),
            errors: self.get_and_reset(CounterType::Error),
            found: self.get(CounterType::Found),
            failed: self.get(CounterType::Failed),
            last_tile: self.get_last_input(),
        }
    }
}

struct StatusMetrics {
    requests: usize,
    ratelimited: usize,
    errors: usize,
    found: usize,
    failed: usize,
    last_tile: Arc<Tile>,
}

pub struct ChannelInfo {
    pub tx_fetcher: Sender<Vec<Tile>>,
    pub tx_out: Sender<Vec<String>>,
}

pub async fn log_status(status: Arc<ProgramStatus>, channel_info: Arc<ChannelInfo>) {
    let mut interval = tokio::time::interval(Duration::from_secs(LOG_INTERVAL_SECS));
    loop {
        interval.tick().await;
        let metrics = status.get_metrics();
        let rps = metrics.requests as f64 / LOG_INTERVAL_SECS as f64;

        info!(
            rps = rps,
            last_tile_x = %metrics.last_tile.x,
            last_tile_y = %metrics.last_tile.y,
            last_tile_zoom = %metrics.last_tile.zoom,
            fetcher_queue = channel_info.tx_fetcher.len(),
            out_queue = channel_info.tx_out.len(),
            ratelimited_last_interval = metrics.ratelimited,
            error_last_interval = metrics.errors,
            found = metrics.found,
            failed = metrics.failed,
            "stats"
        );
    }
}

pub async fn run_status_logger(status: Arc<ProgramStatus>, channel_info: Arc<ChannelInfo>) {
    log_status(status, channel_info)
        .instrument(tracing::info_span!("status_logger"))
        .await;
}