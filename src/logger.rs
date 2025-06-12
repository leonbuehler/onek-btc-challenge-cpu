use bnum::BUint;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};

pub const LOG_PER_SEC: u64 = 3;
use super::CHALLENGE;

pub struct Logger {
    counter: Arc<AtomicUsize>,
}

impl Logger {
    pub fn new() -> Self {
        log::info!("cracking challenge {CHALLENGE}");
        let mut lower_bound: BUint<4> = BUint::ZERO;
        lower_bound.set_bit(CHALLENGE - 1, true);
        let mut upper_bound: BUint<4> = BUint::ZERO;
        upper_bound.set_bit(CHALLENGE, true);

        log::info!("range: {:x}:{:x}", lower_bound, upper_bound);

        let logger = Logger {
            counter: Arc::new(AtomicUsize::new(0)),
        };
        let counter_clone = logger.counter.clone();
        thread::spawn(|| pring_log(counter_clone));
        return logger;
    }

    pub fn increase(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

fn pring_log(counter: Arc<AtomicUsize>) {
    let mut last_measurement = 0;
    loop {
        let current = counter.load(Ordering::Relaxed);
        sleep(Duration::from_secs(LOG_PER_SEC));
        let speed = (current - last_measurement) / 3;
        log::info!("running at {speed} keys/s. (Total: {current})");
        last_measurement = current;
    }
}
