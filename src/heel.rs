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

//Having a pretty hard time determining the difference between TWISPI0 and TWISPI1, to the best of my knowledge they are both peripherals associated with using TWI (I2C)
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

    //So turns out we NEED the ram buffer to be able to store the accel status and know whether it has
    //data to be read lol
    static RAM_BUFFER: static_cell::ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = twim::Twim::new(twi_p, Irqs, sda_p, scl_p, config, RAM_BUFFER.take());

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
        Timer::after_millis(500).await;
        let mut roll: f32 = 0.0;
        let mut pitch: f32 = 0.0;
        if sensor.accel_status().unwrap().xyz_new_data() {
            let data = sensor.acceleration().unwrap();

            //represents x tilt
            pitch = libm::atan2f(
                data.x_raw().into(),
                (data.y_raw().pow(2) + data.z_raw().pow(2)).into(),
            );
            //represents y tilt
            roll = libm::atan2f(
                data.y_raw().into(),
                (data.x_raw().pow(2) + data.z_raw().pow(2)).into(),
            );
        }
        info!("Roll: {}\nPitch: {}", roll, pitch)
    }
}
