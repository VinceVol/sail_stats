use core::any::Any;

use defmt::dbg;
use defmt::info;
use embassy_nrf::{
    bind_interrupts,
    peripherals::{P0_08, P0_16, TWISPI0},
    twim, Peri,
};
use embassy_time::Delay;
use embassy_time::Timer;
use lsm303agr::AccelOutputDataRate;
use lsm303agr::Lsm303agr;
use static_cell::ConstStaticCell;

const ACCEL_ADDY: u8 = 0x19;
const MAG_ADDY: u8 = 0x1E;
//Having a pretty hard time determining the difference between TWISPI0 and TWISPI1, to the best of my knowledge they are both interrupts associated with using TWI (I2C)
//if we have problems with the accelerometer I'll swithc this to TWISPI1 to see if it makes a difference
bind_interrupts!(
    struct Irqs {
        TWISPI0 => twim::InterruptHandler<TWISPI0>;
    }
);

#[embassy_executor::task]
pub async fn init_heel(
    twi_p: Peri<'static, TWISPI0>,
    scl_p: Peri<'static, P0_08>,
    sda_p: Peri<'static, P0_16>,
) {
    info!("Initializing accelorometer twi...");
    let config = twim::Config::default();

    //I think buffer size is just null because it has to do with size of write which we'll only be reading
    static RAM_BUFFER: static_cell::ConstStaticCell<[u8; 0]> = ConstStaticCell::new([0; 0]);
    let mut twi = twim::Twim::new(twi_p, Irqs, sda_p, scl_p, config, RAM_BUFFER.take());

    let mut rx_buf = [0u8; 16];
    info!("Reading...");

    //Following https://github.com/eldruin/lsm303agr-rs/blob/HEAD/examples/microbit-v2.rs this example for implementing accelorometer driver
    let mut sensor = Lsm303agr::new_with_i2c(twi);
    sensor.init().unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut Delay,
            lsm303agr::AccelMode::LowPower,
            AccelOutputDataRate::Hz10,
        )
        .unwrap();
    loop {
        //This loop is obsolete once driver is fully implemented
        let res = twi.read(ACCEL_ADDY, &mut rx_buf).await;
        match res {
            Ok(T) => info!("Read: {=[u8]:x}", rx_buf),
            Err(E) => info!("Error reading accelorometer: {:?}", E),
        }
        Timer::after_millis(400).await;
    }
}
