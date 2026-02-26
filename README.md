# auria-observability

Logging, metrics, and tracing for AURIA Runtime Core.

## Features

- Structured logging with tracing
- Metrics collection
- Distributed tracing support

## Usage

```rust
use auria_observability::{init_logging, log_info};

init_logging("info");
log_info("Auria Node started");
```
