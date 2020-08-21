#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::rtt_init;

#[entry]
fn main() -> ! {
    let channels = rtt_init! {
        up: {
            0: {
                size: 512
                mode: BlockIfFull
                name: "Output zero"
            }
            1: {
                size: 128
                name: "Output one"
            }
        }
        down: {
            0: {
                size: 512
                mode: BlockIfFull
                name: "Input zero"
            }
        }
    };

    let mut output = channels.up.0;
    let mut input = channels.down.0;
    let mut buf = [0u8; 512];

    let mut output2 = channels.up.1;
    writeln!(
        output2,
        "Hi! I will turn anything you type on channel 0 into upper case."
    )
    .ok();

    loop {
        let count = input.read(&mut buf[..]);
        if count > 0 {
            for c in buf.iter_mut() {
                c.make_ascii_uppercase();
            }

            let mut p = 0;
            while p < count {
                p += output.write(&buf[p..count]);
            }
        }
    }
}
