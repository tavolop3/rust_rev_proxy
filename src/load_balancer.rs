use std::{
    net::SocketAddr,
    sync::Arc,
    sync::atomic::{AtomicUsize, Ordering},
};

// use rand::RngExt;

// Enum to save pointer indirection of dynamic dispatch
// generics are not (can't decide on runtime with config)
pub enum Balancer {
    RoundRobin {
        servers: Vec<RRSlot>,
        counter: AtomicUsize,
    },
    // This one may have a thundering herd effect because multiple tasks may read at the same
    // time and choose the same server, making a unique server heavily saturated (see p2c
    // strategy)
    LeastConnections {
        servers: Vec<ServerConnections>,
    },
    // PowerOfTwoChoices {
    //     servers: Vec<ServerConnections>,
    // },
}

// TODO: see repr(align(64)) to avoid false sharing (also drawbacks with cache contention)
pub struct ServerConnections {
    pub addr: SocketAddr,
    pub active_conns: AtomicUsize,
    pub generation: usize,
}

// may be better to create a context struct in a future
pub struct RRSlot {
    pub addr: SocketAddr,
    pub generation: usize,
}

// Generational index
#[derive(Clone, Copy)]
pub struct ServerId {
    index: usize,
    generation: usize,
}

impl Balancer {
    pub fn next(&self) -> Option<(SocketAddr, ServerId)> {
        match self {
            Balancer::RoundRobin { servers, counter } => {
                let len = servers.len();
                if len == 0 {
                    return None;
                }
                // Relaxed ordering will maintain atomicity
                // fetch_add wraps around on overflow
                let i = counter.fetch_add(1, Ordering::Relaxed) % len;
                Some((
                    servers[i].addr,
                    ServerId {
                        index: i,
                        generation: servers[i].generation,
                    },
                ))
            }

            Balancer::LeastConnections { servers } => servers
                .iter()
                .enumerate()
                .min_by_key(|(_, serv_conn)| serv_conn.active_conns.load(Ordering::Relaxed))
                .map(|(i, serv_conn)| {
                    serv_conn.active_conns.fetch_add(1, Ordering::Relaxed);
                    (
                        serv_conn.addr,
                        ServerId {
                            index: i,
                            generation: serv_conn.generation,
                        },
                    )
                }),
            // Balancer::PowerOfTwoChoices { servers } => {
            //     let len = servers.len();
            //     if len == 0 {
            //         return None;
            //     }
            //
            //     let mut rng = rand::rng();
            //     let i_c1 = rng.random_range(0..len);
            //     let i_c2 = rng.random_range(0..len);
            //     let s1 = &servers[i_c1];
            //     let s2 = &servers[i_c2];
            //
            //     let n1 = s1.active_conns.load(Ordering::Relaxed);
            //     let n2 = s2.active_conns.load(Ordering::Relaxed);
            //
            //     let win = if n1 < n2 { s1 } else { s2 };
            //     win.active_conns.fetch_add(1, Ordering::Relaxed);
            //
            //     Some((win.addr,))
            // }
        }
    }

    // Decrement active connections
    pub fn release(&self, server_id: &ServerId) {
        if let Balancer::LeastConnections { servers } = self {
            let s = &servers[server_id.index];
            if server_id.generation != s.generation {
                return;
            }
            s.active_conns.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

pub struct ConnectionGuard {
    pub balancer: Arc<Balancer>,
    pub server_id: ServerId,
}

// this ensures that if connection is closed then it correctly handles it for the load balancer
impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.balancer.release(&self.server_id);
    }
}
