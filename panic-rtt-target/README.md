# panic-rtt-target

[![crates.io](https://img.shields.io/crates/v/panic-rtt-target.svg)](https://crates.io/crates/panic-rtt-target) [![documentation](https://docs.rs/panic-rtt-target/badge.svg)](https://docs.rs/panic-rtt-target)

Logs panic messages over RTT. A companion crate for rtt-target.

## [Documentation](https://docs.rs/panic-rtt-target)

RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a linker error at compile time.

Panics are always logged to the print channel. Upon panicking the channel mode is also automatically set to `BlockIfFull`, so that the full message will always be logged. If the code somehow manages to panic at runtime before RTT is initialized (quite unlikely), or if the print channel doesn't exist, nothing is logged.

The panic handler runs in a non-returning [critical_section](https://docs.rs/critical-section) which implementation should be provided by the user.

# Usage

Cargo.toml:

```toml
[dependencies]
rtt-target = "x.y.z"
panic-rtt-target = "x.y.z"
```

main.rs:

```rust
#![no_std]

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

fn main() -> ! {
    // you can use `rtt_init_print` or you can call `set_print_channel` after initialization.
    rtt_init_default!();

    panic!("Something has gone terribly wrong");
}
```
