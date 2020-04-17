//! Target side implementation of the RTT (Real-Time Transfer) I/O protocol
//!
//! RTT implements input and output to/from a debug probe using in-memory ring buffers and memory
//! polling. This enables debug logging from the microcontroller with minimal delays and no
//! blocking, making it usable even in real-time applications where e.g. semihosting delays cannot
//! be tolerated.
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
//! Different channel objects can safely be used concurrently in different contexts without locking.
//! In an interrupt-based application with realtime constraints you could use a separate channel for
//! every interrupt context to allow for lock-free logging.
//!
//! # Channel modes
//!
//! By default, channels start in [`NoBlockSkip`](ChannelMode::NoBlockSkip) mode, which discards
//! data if the buffer is full. This enables RTT to not crash the application if there is no debug
//! probe attached or if the host is not reading the buffers. However if the application outputs
//! faster than the host can read (which is easy to do, because writing is very fast), messages will
//! be lost. Channels can be set to blocking mode if this is desirable, however in that case the
//! application will likely freeze eventually if the debugger is not attached.
//!
//! The channel mode can also be changed on the fly by the debug probe. Therefore it might be
//! advantageous to use a non-blocking mode in your microcontroller code, and set a blocking mode as
//! needed when debugging. That way you will never end up with an application that freezes without a
//! debugger connected.
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
//! The macros also support an extended syntax to print to different RTT virtual terminals.
//!
//! Please note that because a critical section is used, printing into a blocking channel will cause
//! the application to block and freeze when the buffer is full.

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
/// [`write`] as well as the ufmt crate's `uwrite` macros (use the `u` method).
///
/// Note that the formatted writing implementations diverge slightly from the trait definitions in
/// that if the channel is in non-blocking mode, writing will *not* block.
pub struct UpChannel(*mut rtt::RttChannel);

unsafe impl Send for UpChannel {}

impl UpChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut rtt::RttChannel) -> Self {
        UpChannel(channel)
    }

    fn channel(&self) -> &mut rtt::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Writes `buf` to the channel and returns the number of bytes written. Behavior when the
    /// buffer is full is subject to the channel blocking mode.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        let mut writer = self.channel().writer();
        writer.write(buf);
        writer.commit()
    }

    /// Creates a writer for formatted writing with ufmt.
    ///
    /// The correct way to use this method is to call it once for each write operation. This is so
    /// that non blocking modes will work correctly.
    ///
    /// ```
    /// let mut output = channels.up.0;
    /// uwriteln!(output.u(), "Hello, ufmt!");
    /// ```
    pub fn u(&mut self) -> uWriter {
        uWriter(self.channel().writer())
    }

    /// Gets the current blocking mode of the channel. The default is `NoBlockSkip`.
    pub fn mode(&self) -> ChannelMode {
        self.channel().mode()
    }

    /// Sets the blocking mode of the channel
    pub fn set_mode(&mut self, mode: ChannelMode) {
        self.channel().set_mode(mode)
    }

    /// Converts the channel into a virtual terminal that can be used for writing into multiple
    /// virtual terminals.
    pub fn into_terminal(self) -> TerminalChannel {
        TerminalChannel::new(self)
    }
}

impl fmt::Write for UpChannel {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.channel().writer().write_str(s)
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> Result<(), fmt::Error> {
        self.channel().writer().write_fmt(args)
    }
}

/// Writer for ufmt. Don't store an instance of this, but rather create a new one for every write.
#[allow(non_camel_case_types)]
pub struct uWriter<'c>(rtt::RttWriter<'c>);

impl uWrite for uWriter<'_> {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write(s.as_bytes());
        Ok(())
    }
}

/// RTT down (host to target) channel
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

    /// Reads up to `buf.len()` bytes from the channel and return the number of bytes read. This
    /// method never blocks.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.channel().read(buf)
    }
}

/// Specifies what to do when a channel doesn't have enough buffer space for a complete write.
#[derive(Eq, PartialEq)]
#[repr(usize)]
pub enum ChannelMode {
    /// Skip writing the data completely if it doesn't fit in its entirety.
    NoBlockSkip = 0,

    /// Write as much as possible of the data and ignore the rest.
    NoBlockTrim = 1,

    /// Block (spin) if the buffer is full. If within a critical section such as inside
    /// [`rprintln`], this will cause the application to freeze until the host reads from the
    /// buffer.
    BlockIfFull = 2,
}

/// An up channel that supports writing into multiple virtual terminals within the same buffer.
///
/// An [`UpChannel`] can be turned into a `TerminalChannel` by using the
/// [`into_terminal`](UpChannel::into_terminal()) method.
///
/// Virtual terminals allow you to share one buffer for writing multiple streams. The virtual
/// terminals number from 0 to 15 and are implemented with a simple "terminal switch" sequence on
/// the fly, so there is no need to declare them in advance. You could, for example, use different
/// terminal numbers for messages of different priorities to separate them in a viewer program.
/// Printing uses a `TerminalChannel` internally.
pub struct TerminalChannel {
    channel: UpChannel,
    current: u8,
}

impl TerminalChannel {
    pub(crate) fn new(channel: UpChannel) -> Self {
        Self {
            channel,
            current: 0,
        }
    }

    /// Creates a writer to write a message to the virtual terminal specified by `number`.
    ///
    /// The correct way to use this method is to call it once for each write operation. This is so
    /// that non blocking modes will work correctly.
    ///
    /// The writer supports formatted writing with the standard `write` and ufmt's `uwrite`.
    pub fn write(&mut self, number: u8) -> TerminalWriter {
        const TERMINAL_ID: [u8; 16] = *b"0123456789ABCDEF";

        let mut writer = self.channel.channel().writer();

        if number != self.current {
            // The terminal switch command must be sent in full so the mode cannot be NoBlockTrim
            let mode = self.channel.mode();
            let mode = if mode == ChannelMode::NoBlockTrim {
                ChannelMode::NoBlockSkip
            } else {
                mode
            };

            writer.write_with_mode(mode, &[0xff, TERMINAL_ID[(number & 0x0f) as usize]]);

            self.current = number;
        }

        TerminalWriter { writer, number, current: &mut self.current }
    }

    /// Gets the current blocking mode of the channel. The default is `NoBlockSkip`.
    pub fn mode(&self) -> ChannelMode {
        self.channel.mode()
    }

    /// Sets the blocking mode of the channel
    pub fn set_mode(&mut self, mode: ChannelMode) {
        self.channel.set_mode(mode)
    }
}

/// Formatted writing operation. Don't store an instance of this, but rather create a new one for
/// every write.
pub struct TerminalWriter<'c> {
    writer: rtt::RttWriter<'c>,
    number: u8,
    current: &'c mut u8,
}

impl fmt::Write for TerminalWriter<'_> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.writer.write(s.as_bytes());
        Ok(())
    }
}

impl uWrite for TerminalWriter<'_> {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        self.writer.write(s.as_bytes());
        Ok(())
    }
}

impl Drop for TerminalWriter<'_> {
    fn drop(&mut self) {
        if !self.writer.is_failed() {
            *self.current = self.number;
        }
    }
}