# panic-rtt-target

[![crates.io](https://meritbadge.herokuapp.com/panic-rtt-target)](https://crates.io/crates/panic-rtt-target) [![documentation](https://docs.rs/panic-rtt-target/badge.svg)](https://docs.rs/panic-rtt-target)

Logs panic messages over RTT. A companion crate for rtt-target.

## [Documentation](https://docs.rs/panic-rtt-target)

RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a linker error at compile time.

Panics are always logged on channel 0. Upon panicking the channel mode is also automatically set to `BlockIfFull`, so that the full message will always be logged. If the code somehow manages to panic at runtime before RTT is initialized (quite unlikely), or if channel 0 doesn't exist, nothing is logged.

A platform feature such as `cortex-m` is required to use this crate.

# Usage

Cargo.toml:

```toml
[dependencies]
panic-rtt-target = { version = "x.y.z", features = ["cortex-m"] }
```

main.rs:

```rust
#![no_std]

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

fn main() -> ! {
    // you can use any init macro as long as it creates channel 0
    rtt_init_default!();

    panic!("Something has gone terribly wrong");
}
```
