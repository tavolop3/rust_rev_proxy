use std::sync::atomic::{AtomicUsize, Ordering};

pub trait LoadBalanceStrategy: Sync + Send {
    fn next<'a>(&self, backends: &'a [&str]) -> &'a str;
}

pub struct RoundRobinBalancer {
    counter: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        RoundRobinBalancer {
            counter: AtomicUsize::new(0),
        }
    }
}

impl LoadBalanceStrategy for RoundRobinBalancer {
    fn next<'a>(&self, backends: &'a [&str]) -> &'a str {
        // Backend wil remain same size for the duration of the program
        let len = backends.len();
        // Relaxed ordering will maintain atomicity
        // fetch_add wraps around on overflow
        let i = self.counter.fetch_add(1, Ordering::Relaxed) % len;
        backends[i]
    }
}
