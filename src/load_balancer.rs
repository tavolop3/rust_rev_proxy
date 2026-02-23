use std::{
    net::SocketAddr,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
};

// Enum to save pointer indirection of dynamic dispatch
// generics are not (can't decide on runtime with config)
pub enum Balancer {
    RoundRobin {
        servers: Vec<SocketAddr>,
        counter: AtomicUsize,
    },
    // This one gonna have a thundering thurd effect because multiple tasks may read at the same
    // time and come to the same conclussion, making a unique heavily server saturated (see p2c
    // strategy)
    LeastConnections {
        servers: Vec<ServerConnections>,
    },
}

// TODO: see repr(align(64)) to avoid false sharing (also drawbacks with cache contention)
pub struct ServerConnections {
    pub addr: SocketAddr,
    pub active_conns: AtomicUsize,
}

impl Balancer {
    pub fn next(&self) -> Option<(SocketAddr, usize)> {
        match self {
            Balancer::RoundRobin { servers, counter } => {
                let len = servers.len();
                if len == 0 {
                    return None;
                }
                // Relaxed ordering will maintain atomicity
                // fetch_add wraps around on overflow
                let i = counter.fetch_add(1, Ordering::Relaxed) % len;
                Some((servers[i], i))
            }

            Balancer::LeastConnections { servers } => servers
                .iter()
                .enumerate()
                .min_by_key(|(_, serv_conn)| serv_conn.active_conns.load(Ordering::Relaxed))
                .map(|(i, serv_conn)| {
                    serv_conn.active_conns.fetch_add(1, Ordering::Relaxed);
                    (serv_conn.addr, i)
                }),
        }
    }

    // Decrement active connections
    pub fn release(&self, index: usize) {
        if let Balancer::LeastConnections { servers } = self {
            servers[index].active_conns.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

pub struct ConnectionGuard {
    pub balancer: Arc<Balancer>,
    pub server_index: usize,
}

// this ensures that if connection is closed then it correctly handles it for the load balancer
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.balancer.release(self.server_index);
    }
}
