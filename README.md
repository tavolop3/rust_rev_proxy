# Async TCP Reverse Proxy (Rust + Tokio)

A minimal asynchronous TCP reverse proxy written in Rust using Tokio.  
Taking in mind TCP half closes, backpressure, memory and CPU efficiency.
*Disclaimer*: This project is intended for learning purposes only.

## Overview

This project is a layer-4 (TCP) reverse proxy that:

- Listens for incoming client connections
- Selects a backend server using a load balancing strategy
- Proxies data bidirectionally between client and several backends
- Tracks active connections per backend (Least Connections strategy)

The architecture is fully asynchronous and built on Tokio’s non-blocking runtime.

## Future Improvements

* ~~Fix thundering herd issue (e.g., P2C strategy)~~
* Improve error handling
* Graceful shutdown handling
* Add more load balancing strategies (random, hash-based, weighted RR)
* Benchmarks
* Add rate limiting
* Implement circuit breaker
* Add metrics and structured logging
* Introduce connection queueing under high load
* Add health checks
* Support external config file (optional hot reload)
* Optional TLS support

