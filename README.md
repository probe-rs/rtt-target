# rtt-target

[![crates.io](https://img.shields.io/crates/v/rtt-target.svg)](https://crates.io/crates/rtt-target) [![documentation](https://docs.rs/rtt-target/badge.svg)](https://docs.rs/rtt-target)

Target side implementation of the RTT (Real-Time Transfer) I/O protocol. RTT implements input and output via a debug probe using in-memory ring buffers and polling. This enables debug logging from the microcontroller with minimal delays and no blocking, making it usable even in real-time applications where e.g. semihosting delays cannot be tolerated.

## [Documentation](https://docs.rs/rtt-target)

## Platform support

A platform-specific [`critical-section`](https://github.com/rust-embedded/critical-section) implementation is needed to use this library.

Output directly to a channel object with `write!` or the binary `write` method does not require locking and therefore does not need any platform-specific critical section.

## Usage

With a platform-specific critical section in use, printing is as simple as:

```rust
use rtt_target::{rtt_init_print, rprintln};

fn main() {
    rtt_init_print!();
    loop {
        rprintln!("Hello, world!");
    }
}
```

`rtt-target` also supports initializing multiple RTT channels, and even has a logger implementations
for [`log`](https://docs.rs/log/latest/log/) and [`defmt`](https://defmt.ferrous-systems.com/) that can be used in conjunction with arbitrary
channel setups.

The `defmt` integration requires setting `features = ["defmt"]`. Furthermore, you have to either invoke `rtt_init_defmt!` or set up your channel(s) manually and invoke `set_defmt_channel` before using `defmt`.

The `log` integration requires setting `features = ["log"]`. Furthermore, you have to either invoke `rtt_init_log!` or set up your channel(s) manually and invoke `init_logger`/`init_logger_with_level` before using `log`.

**Note**: For your platform, particularly if you're using a multi-core MCU, external logger implementations might be better suited than the one provided by this crate via the `log`/`defmt` feature.

For more information, please check out the [documentation](https://docs.rs/rtt-target).

## Development

The examples-cortex-m and panic-test crates come with build files for the venerable STM32F103C8xx by default, but can be easily adapted for any chip as they contain only minimal platform-specific runtime code to get `fn main` to run.
