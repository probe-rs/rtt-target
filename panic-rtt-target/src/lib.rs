//! Logs panic messages over RTT. A companion crate for rtt-target.
//!
//! RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a
//! linker error at compile time.
//!
//! Panics are always logged to the print channel. Upon panicking the channel mode is also
//! automatically set to `BlockIfFull`, so that the full message will always be logged.
//! If the code somehow manages to panic at runtime before RTT is initialized (quite unlikely),
//! or if the print channel doesn't exist, nothing is logged.
//!
//! The panic handler runs in a non-returning [critical_section](https://docs.rs/critical-section)
//! which implementation should be provided by the user.
//!
//! # Usage
//!
//! Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! rtt-target = "x.y.z"
//! panic-rtt-target = "x.y.z"
//! ```
//!
//! main.rs:
//!
//! ```no_run
//! #![no_std]
//!
//! use panic_rtt_target as _;
//! use rtt_target::rtt_init_default;
//!
//! fn main() -> ! {
//!     // you can use `rtt_init_print`, `rtt_init_defmt` or you can call `set_print_channel` after initialization.
//!     rtt_init_default!();
//!
//!     panic!("Something has gone terribly wrong");
//! }
//! ```

#![no_std]

use core::{fmt::Write, panic::PanicInfo};
use portable_atomic::{compiler_fence, Ordering};

use rtt_target::{with_terminal_channel, ChannelMode};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    critical_section::with(|_| {
        with_terminal_channel(|term| {
            term.set_mode(ChannelMode::BlockIfFull);
            let mut channel = term.write(0);

            writeln!(channel, "{}", info).ok();
        });

        // we should never leave critical section
        loop {
            compiler_fence(Ordering::SeqCst);
        }
    })
}
