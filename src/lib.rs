#![no_std]

use core::fmt;

#[doc(hidden)]
pub mod implementation;

/// RTT up (target to host) channel
pub struct UpChannel(*mut implementation::RttChannel);

unsafe impl Send for UpChannel { }

impl UpChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut implementation::RttChannel) -> Self {
        UpChannel(channel)
    }

    fn channel(&mut self) -> &mut implementation::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Writes up to `buf.len()` bytes to the channel and returns the number of bytes written.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        self.channel().write(buf)
    }
}

impl fmt::Write for UpChannel {
    fn write_str(&mut self, mut s: &str) -> Result<(), fmt::Error> {
        while s.len() > 0 {
            let count = self.channel().write(s.as_bytes());

            s = &s[count..];
        }

        Ok(())
    }
}

/// RTT down (target to host) channel
pub struct DownChannel(*mut implementation::RttChannel);

unsafe impl Send for DownChannel { }

impl DownChannel {
    /// Public due to access from macro.
    #[doc(hidden)]
    pub unsafe fn new(channel: *mut implementation::RttChannel) -> Self {
        DownChannel(channel)
    }

    fn channel(&mut self) -> &mut implementation::RttChannel {
        unsafe { &mut *self.0 }
    }

    /// Reads up to `buf.len()` bytes from the channel and return the number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        self.channel().read(buf)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! rtt_init_repeat {
    ({ $($code:tt)+ } { $($acc:tt)* }; $n:literal: { $($_:tt)* } $($tail:tt)*) => {
        $crate::rtt_init_repeat!({ $($code)* } { $($code)* $($acc)* }; $($tail)*)
    };
    ({ $($code:tt)+ } { $($acc:tt)* };) => {
        ($($acc)*)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! rtt_channel_count {
    ($n:literal: { $($_:tt)* } $($tail:tt)*) => {
        1 + $crate::rtt_channel_count!($($tail)*)
    };
    () => { 0 };
}

#[macro_export]
#[doc(hidden)]
macro_rules! rtt_init_channels {
    (
        $field:expr;
        $number:literal: {
            size: $size:literal
            $(
                name: $name:literal
            )?
        }
        $($tail:tt)*
    ) => {
        let mut name: *const u8 = core::ptr::null();

        $(
            name = concat!($name, "\0").as_bytes().as_ptr();
        )?

        $field[$number].init(name, {
            static mut RTT_CHANNEL_BUFFER: MaybeUninit<[u8; $size]> = MaybeUninit::uninit();
            RTT_CHANNEL_BUFFER.as_mut_ptr()
        });

        $crate::rtt_init_channels!($field; $($tail)*);
    };
    ($field:expr;) => { };
}

#[macro_export]
#[doc(hidden)]
macro_rules! rtt_init_wrappers {
    ($field:expr; $cons:path; { $($acc:tt)* }; $n:literal: { $($_:tt)* } $($tail:tt)*) => {
        $crate::rtt_init_wrappers!(
            $field;
            $cons;
            {
                $($acc)*
                $cons(&mut $field[$n] as *mut _),
            };
            $($tail)*)
    };
    ($field:expr; $cons:path; { $($acc:tt)* };) => {
        ($($acc)*)
    };
}

/// Initializes RTT with the specified channels. Channel numbers, buffer sizes and names can be
/// defined.
///
/// The channel numbers must start from 0 and not skip any numbers, or otherwise odd things will
/// happen. The order does not matter. This macro should be called once within a function,
/// preferably close to the start of your entry point. The macro must only be called once - if it's
/// called twice in the same program a duplicate symbol error will occur.
///
/// ```
/// let channels = rtt_init! {
///     up: {
///         0: { // channel number
///             size: 1024 // buffer size in bytes
///             name: "Terminal" // optional name
///         }
///         1: {
///             size: 32
///         }
///     }
///     down: {
///         0: {
///             size: 16
///             name: "Terminal"
///         }
///     }
/// };
/// ```
///
/// At compile time the macro will reserve space for the RTT control block as well as all the
/// buffers as uninitialized static variables. At runtime the macro fills in the structures and
/// prepares them for use.
///
/// The macro returns a generate struct that contains the channels. The struct for the example above
/// would look as follows:
///
/// ```
/// struct Channels {
///     up: (UpChannel, UpChannel),
///     down: (DownChannel,),
/// }
/// ```
///
/// The channels can either be accessed by reference or moved out as needed. For example:
///
/// ```
/// use core::fmt::Write;
///
/// let channels = rtt_init! { ... };
/// let mut output = channels.up.0;
/// writeln!(output, "Hello, world!").ok();
/// ```
#[macro_export]
macro_rules! rtt_init {
    {
        $(up: { $($up:tt)* } )?
        $(down: { $($down:tt)* } )?
    } => {{
        use core::mem::MaybeUninit;
        use core::ptr;
        use $crate::UpChannel;
        use $crate::DownChannel;
        use $crate::implementation::*;

        #[repr(C)]
        pub struct RttControlBlock {
            header: RttHeader,
            $( up_channels: [RttChannel; $crate::rtt_init_repeat!({ 1 + } { 0 }; $($up)*)], )?
            $( down_channels: [RttChannel; $crate::rtt_init_repeat!({ 1 + } { 0 }; $($down)*)], )?
        }

        #[no_mangle]
        #[used]
        pub static mut SEGGER_RTT: MaybeUninit<RttControlBlock> = MaybeUninit::uninit();

        unsafe {
            ptr::write_bytes(SEGGER_RTT.as_mut_ptr(), 0, 1);

            let rtt = &mut *SEGGER_RTT.as_mut_ptr();

            rtt.header.init(rtt.up_channels.len(), rtt.down_channels.len());

            $( $crate::rtt_init_channels!(rtt.up_channels; $($up)*); )?
            $( $crate::rtt_init_channels!(rtt.down_channels; $($down)*); )?

            pub struct Channels {
                $( up: $crate::rtt_init_repeat!({ UpChannel, } {}; $($up)*), )?
                $( down: $crate::rtt_init_repeat!({ DownChannel, } {}; $($down)*), )?
            }

            Channels {
                $( up: $crate::rtt_init_wrappers!(rtt.up_channels; UpChannel::new; {}; $($up)*), )?
                $( down: $crate::rtt_init_wrappers!(rtt.down_channels; DownChannel::new; {}; $($down)*), )?
            }
        }
    }};
}

/// Initializes RTT with default channels.
///
/// The default channels are up channel 0 with a 1024 byte buffer and down channel 0 with a 16 byte
/// buffer. Both channels are called "Terminal". This macro is equivalent to:
///
/// ```
/// rtt_init! {
///     up: {
///         0: {
///             size: 1024
///             name: "Terminal"
///         }
///     }
///     down: {
///         0: {
///             size: 16
///             name: "Terminal"
///         }
///     }
/// };
/// ```
///
/// See [rtt_init](rtt_init) for more details.
#[macro_export]
macro_rules! rtt_init_default {
    () => {
        $crate::rtt_init! {
            up: {
                0: {
                    size: 1024
                    name: "Terminal"
                }
            }
            down: {
                0: {
                    size: 16
                    name: "Terminal"
                }
            }
        };
    };
}
