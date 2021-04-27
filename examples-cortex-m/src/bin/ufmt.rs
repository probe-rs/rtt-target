#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rtt_init_default, ChannelMode};
use ufmt::uwriteln;

#[entry]
fn main() -> ! {
    let channels = rtt_init_default!();

    let mut output = channels.up.0;
    output.set_mode(ChannelMode::BlockIfFull);

    let mut i = 0;
    loop {
        uwriteln!(output.u(), "Hello from RTT! {}", i).ok();

        i += 1;
    }
}
