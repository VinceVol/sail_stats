#![no_std]
#![no_main]

mod fmt;
mod heel;
mod micro_sd;

use core::borrow::Borrow;

use defmt::dbg;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{AnyPin, Input, Level, Output, OutputDrive},
    peripherals::P0_13,
};
use embassy_time::Timer;
use fmt::info;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let _ = spawner.spawn(button(p.P0_14.into(), ButtonSide::A));
    let _ = spawner.spawn(button(p.P0_23.into(), ButtonSide::B));
    //let res = spawner.spawn(heel::init_heel(p.TWISPI0, p.P0_08, p.P0_16));

    //I think we may need to preinitialized micro sd card
    let cs_pin = Output::new(p.P1_02, Level::High, OutputDrive::Standard);
    let _ = spawner.spawn(micro_sd::init_save(
        p.SPI2, p.P0_01, p.P0_13, p.P0_17, cs_pin,
    ));
}

//Putting this as a gut check to make sure the board is working and interacting
enum ButtonSide {
    A,
    B,
}

#[embassy_executor::task(pool_size = 2)]
async fn button(pin: embassy_nrf::Peri<'static, AnyPin>, button_side: ButtonSide) {
    let mut button = Input::new(pin, embassy_nrf::gpio::Pull::None);
    loop {
        button.wait_for_low().await;
        match button_side {
            ButtonSide::A => {
                info!("you've pressed button a!");
            }
            ButtonSide::B => {
                info!("you've pressed button b!");
            }
        }
        embassy_time::Timer::after_millis(150).await;
        button.wait_for_high().await;
    }
}
