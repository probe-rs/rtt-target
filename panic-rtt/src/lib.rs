//! Logs panic messages over RTT.
//!
//! RTT must have been initialized by using one of the `rtt_init` macros. Otherwise you will get a
//! linker error at compile time. Panics are always logged on a channel 0. Upon panicking the
//! channel mode is also automatically set to `BlockIfFull`, so that the full message will always be
//! logged.
//!
//! A platform feature such as `cortex-m` is required to use this crate.
//!
//! # Usage
//!
//! Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! panic-rtt = { version = "x.y.z", features = ["cortex-m"] }
//! ```
//!
//! main.rs:
//!
//! ```ignore
//! #![no_std]
//!
//! use panic_rtt as _;
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
    sync::atomic::{compiler_fence, Ordering::SeqCst},
    fmt::Write,
    panic::PanicInfo,
};

#[allow(unused)]
use rtt_target::{ChannelMode, UpChannel};

#[cfg(feature = "cortex-m")]
#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use cortex_m::interrupt;

    interrupt::disable();

    if let Some(mut channel) = unsafe { UpChannel::conjure(0) } {
        channel.set_mode(ChannelMode::BlockIfFull);

        writeln!(channel, "{}", info).ok();
    }

    loop {
        compiler_fence(SeqCst);
    }
}

#[cfg(not(any(feature = "cortex-m")))]
compile_error!("You must specify a platform feature for panic-rtt, such as 'cortex-m'.");
