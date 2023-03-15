//! Logs panic messages over RTT. A companion crate for rtt-target.
//!
//! RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a
//! linker error at compile time.
//!
//! Panics are always logged on channel 0. Upon panicking the channel mode is also automatically set
//! to `BlockIfFull`, so that the full message will always be logged. If the code somehow manages to
//! panic at runtime before RTT is initialized (quite unlikely), or if channel 0 doesn't exist,
//! nothing is logged.
//!
//! A platform feature such as `cortex-m` is required to use this crate.
//!
//! # Usage
//!
//! Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! panic-rtt-target = { version = "x.y.z", features = ["cortex-m"] }
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
//!     // you can use any init macro as long as it creates channel 0
//!     rtt_init_default!();
//!
//!     panic!("Something has gone terribly wrong");
//! }
//! ```

#![no_std]

// allow(unused) is used so that warnings when no platform feature is defined don't drown out the
// compile_error

#[allow(unused)]
use core::{
    fmt::Write,
    panic::PanicInfo,
    sync::atomic::{compiler_fence, Ordering::SeqCst},
};

#[allow(unused)]
use rtt_target::{ChannelMode, UpChannel};

#[inline(never)]
#[panic_handler]
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
