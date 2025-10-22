#![no_std]
#![no_main]

mod fmt;
mod heel;
mod micro_sd;

use core::borrow::Borrow;

use defmt::dbg;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
use static_cell::{ConstStaticCell, StaticCell};
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Input, Level, Output, OutputDrive},
    interrupt::Interrupt::I2S,
    peripherals::{P0_13, TWISPI0},
    twim::{self, Twim},
};
use embassy_time::Timer;
use fmt::info;

//I2C init for all
pub type TwiBus = Mutex<NoopRawMutex, Twim<'static, TWISPI0>>;
bind_interrupts!(
    struct Irqs {
        TWISPI0 => twim::InterruptHandler<TWISPI0>;
    }
);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    //Need to initialize i2c in main loop so that multiple peripherals can use it
    // DROP A LINK TO EXAMPLE SINCE GENTEX BLOCKS EVERYTHING!
    let config = twim::Config::default();
    static RAM_BUFFER: static_cell::ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = twim::Twim::new(p.TWISPI0, Irqs, p.P0_16, p.P0_08, config, RAM_BUFFER.take());
    static TWI_BUS: StaticCell<TwiBus> = StaticCell::new();
    let twi_bus = TWI_BUS.init(Mutex::new(twi));
    let res = spawner.spawn(heel::init_heel(twi_bus));

    //I think we may need to preinitialized micro sd card
    let cs_pin = Output::new(p.P1_02, Level::High, OutputDrive::Standard);
    let _ = spawner.spawn(micro_sd::init_save(
        p.SPI2, p.P0_01, p.P0_13, p.P0_17, cs_pin,
    ));
}
