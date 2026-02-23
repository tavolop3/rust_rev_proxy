use std::sync::atomic::{AtomicUsize, Ordering};

pub trait LoadBalanceStrategy: Sync + Send {
    fn next(&self) -> Option<&str>;
}

pub struct RoundRobinBalancer {
    servers: Vec<String>,
    counter: AtomicUsize,
}

impl RoundRobinBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        RoundRobinBalancer {
            servers,
            counter: AtomicUsize::new(0),
        }
    }
}

impl LoadBalanceStrategy for RoundRobinBalancer {
    fn next(&self) -> Option<&str> {
        // Backend wil remain same size for the duration of the program
        let len = self.servers.len();
        if len == 0 {
            return None;
        };
        // Relaxed ordering will maintain atomicity
        // fetch_add wraps around on overflow
        let i = self.counter.fetch_add(1, Ordering::Relaxed) % len;
        Some(&self.servers[i])
    }
}

// Lifetimes ensure that the data referenced by a struct is valid for as long as the struct is.
struct CounterAndAddress {
    pub counter: AtomicUsize,
    pub address: String,
}

// This one is gonna have a thundering thurd effect because multiple tasks may read at the same
// time and come to the same conclussion, making a unique heavily server saturated (see p2c
// strategy)
pub struct LeastConnectionsBalancer {
    conn_per_serv: Vec<CounterAndAddress>,
}

impl LeastConnectionsBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        let conn_per_serv = servers
            .into_iter()
            .map(|s| CounterAndAddress {
                counter: AtomicUsize::new(0),
                address: s,
            })
            .collect();
        Self { conn_per_serv }
    }
}

impl LoadBalanceStrategy for LeastConnectionsBalancer {
    fn next(&self) -> Option<&str> {
        self.conn_per_serv
            .iter()
            .min_by_key(|s| s.counter.load(Ordering::Relaxed))
            .map(|s| s.address.as_str())
    }
}
