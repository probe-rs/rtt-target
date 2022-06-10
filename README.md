# rtt-target

[![crates.io](https://meritbadge.herokuapp.com/rtt-target)](https://crates.io/crates/rtt-target) [![documentation](https://docs.rs/rtt-target/badge.svg)](https://docs.rs/rtt-target)

Target side implementation of the RTT (Real-Time Transfer) I/O protocol. RTT implements input and output via a debug probe using in-memory ring buffers and polling. This enables debug logging from the microcontroller with minimal delays and no blocking, making it usable even in real-time applications where e.g. semihosting delays cannot be tolerated.

## [Documentation](https://docs.rs/rtt-target)

## Platform support

While this crate is platform agnostic, some platform-specific code is needed for locking if you want to use the global `rprintln!` macro.

If using Cortex-M or RISC-V, there is built-in support with a feature flag:

```toml
# Cargo.toml
rtt-target = { version = "x.y.z", features = ["cortex-m"] }
# or
rtt-target = { version = "x.y.z", features = ["riscv"] }
```

Otherwise, check the documentation for the `set_print_channel_cs` function.

Output directly to a channel object with `write!` or the binary `write` method does not require locking and therefore does not need any platform-specific code.

## Usage

With a platform support feature, printing is as simple as:

```rust
use rtt_target::{rtt_init_print, rprintln};

fn main() {
    rtt_init_print!();
    loop {
        rprintln!("Hello, world!");
    }
}
```

## Development

The examples-cortex-m and panic-test crates come with build files for the venerable STM32F103C8xx by default, but can be easily adapted for any chip as they contain only minimal platform-specific runtime code to get `fn main` to run.
