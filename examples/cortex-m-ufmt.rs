#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt2::rtt_init_default;
use ufmt::uwriteln;

#[entry]
fn main() -> ! {
    let mut channels = rtt_init_default!();

    let mut i = 0;
    loop {
        uwriteln!(channels.up.0, "Hello from RTT! {}", i).ok();

        i += 1;
    }
}
