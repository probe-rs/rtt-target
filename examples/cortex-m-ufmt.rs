#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rtt_init_default, ChannelMode};
use ufmt::uwriteln;

#[entry]
fn main() -> ! {
    let mut channels = rtt_init_default!();
    channels.up.0.set_mode(ChannelMode::BlockIfFull);

    let mut i = 0;
    loop {
        uwriteln!(channels.up.0, "Hello from RTT! {}", i).ok();

        i += 1;
    }
}
