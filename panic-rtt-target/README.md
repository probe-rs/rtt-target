# panic-rtt-target

[![crates.io](https://meritbadge.herokuapp.com/panic-rtt-target)](https://crates.io/crates/panic-rtt-target) [![documentation](https://docs.rs/panic-rtt-target/badge.svg)](https://docs.rs/panic-rtt-target)

Logs panic messages over RTT. A companion crate for rtt-target.

## [Documentation](https://docs.rs/panic-rtt-target)

RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a linker error at compile time.

Panics are always logged on channel 0. Upon panicking the channel mode is also automatically set to `BlockIfFull`, so that the full message will always be logged. If the code somehow manages to panic at runtime before RTT is initialized (quite unlikely), or if channel 0 doesn't exist, nothing is logged. 

The panic handler runs in a non-returning [critical_section](https://docs.rs/critical-section/latest/critical_section/) which implementation should be provided by the user. 

# Usage

Cargo.toml:

```toml
[dependencies]
cortex-m = { version = "0.7.6", features = ["critical-section-single-core"]}
panic-rtt-target = { version = "x.y.z" }
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

## Implementation details

The provided interrupt handler checks if RTT channel 0 is configured, writes the `info` and enters an infinite loop. If RTT channel 0 is not configured, the panic handler enters the *failed to get channel* infinite loop. The final state can be observed by breaking/halting the target. 
```rust
fn panic(info: &PanicInfo) -> ! {
    critical_section::with(|_| {
        if let Some(mut channel) = unsafe { UpChannel::conjure(0) } {
            channel.set_mode(ChannelMode::BlockIfFull);

            writeln!(channel, "{}", info).ok();
        } else {
            // failed to get channel, but not much else we can do but spin
            loop {
                compiler_fence(SeqCst);
            }
        }

        // we should never leave critical section
        loop {
            compiler_fence(SeqCst);
        }
    })
}
```