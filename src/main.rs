#![no_std]
#![no_main]

mod fmt;
mod heel;
mod micro_sd;

use defmt::dbg;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive};
use embassy_time::Timer;
use fmt::info;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    let _ = spawner.spawn(button(p.P0_14.into(), ButtonSide::A));
    let _ = spawner.spawn(button(p.P0_23.into(), ButtonSide::B));
    let res = spawner.spawn(heel::init_heel(p.TWISPI0, p.P0_08, p.P0_16));

    loop {
        info!("Hello, World!");
        led.set_high();
        Timer::after_secs(3).await;
        led.set_low();
        Timer::after_secs(3).await;
    }
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
