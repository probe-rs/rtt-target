//! Target side implementation of the RTT (Real-Time Transfer) I/O protocol
//!
//! RTT implements input and output to/from a debug probe using in-memory ring buffers and memory
//! polling. This enables debug logging from the microcontroller with minimal delays, making it
//! suitable for even real-time applications where e.g. semihosting delays would break things.
//!
//! # Hardware support
//!
//! This crate is platform agnostic and can be used on any chip that supports background memory
//! access via its debug interface. The printing macros require a critical section which is
//! platform-dependent. ARM Cortex-M has built-in support, which can be enabled with the "cortex-m"
//! feature flag.
//!
//! To interface with RTT from the host computer, a debug probe such as an ST-Link or J-Link is
//! required. The normal debug protocol (e.g. SWD) is used to access RTT, so no extra connections
//! such as SWO pins are needed.
//!
//! # Initialization
//!
//! RTT must be initialized at the start of your program using one of the init macros. See the
//! macros for more details.
//!
//! The initialization macros return channel objects that can be used for writing and reading.
//! Different channel objects can be safely used concurrently in different contexts without locking.
//! In an interrupt-based application with realtime constraints you could use a separate channel for
//! every interrupt context to allow for minimal delays.
//!
//! # Printing
//!
//! For no-hassle output the [`rprint`] and [`rprintln`] macros are provided. They use a single down
//! channel defined at initialization time, and a critical section for synchronization, and they
//! therefore work exactly like the standard `println` style macros. They can be used from any
//! context.
//!
//! ```
//! use rtt_target::{rtt_init_print, rprintln};
//!
//! fn main() -> ! {
//!     rtt_init_print!();
//!     loop {
//!         rprintln!("Hello, world!");
//!     }
//! }
//! ```
//!
//! Please note that because qcritical section is used, using printing with a blocking channel will
//! cause the application to freeze completely when the buffer is full.

#![no_std]

use core::convert::Infallible;
use core::fmt;
use ufmt_write::uWrite;

#[macro_use]
mod init;

/// Public due to access from macro
#[doc(hidden)]
pub mod rtt;

#[macro_use]
mod print;

pub use print::*;

/// RTT up (target to host) channel
///
/// Supports writing binary data directly, or writing strings via [`core::fmt`] macros such as
/// [`write`] as well as the ufmt crate's uwrite macros.
pub struct UpChannel(*mut rtt::RttChannel);

unsafe impl Send for UpChannel {}

impl UpChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut rtt::RttChannel) -> Self {
        UpChannel(channel)
    }

    fn channel(&mut self) -> &mut rtt::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Writes up to `buf.len()` bytes to the channel and returns the number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.channel().write(buf)
    }

    fn write_str(&mut self, mut s: &str) {
        while s.len() > 0 {
            let count = self.channel().write(s.as_bytes());

            s = &s[count..];
        }
    }
}

impl fmt::Write for UpChannel {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_str(s);

        Ok(())
    }
}

impl uWrite for UpChannel {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.write_str(s);

        Ok(())
    }
}

/// RTT down (target to host) channel
pub struct DownChannel(*mut rtt::RttChannel);

unsafe impl Send for DownChannel {}

impl DownChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut rtt::RttChannel) -> Self {
        DownChannel(channel)
    }

    fn channel(&mut self) -> &mut rtt::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Reads up to `buf.len()` bytes from the channel and return the number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.channel().read(buf)
    }
}
