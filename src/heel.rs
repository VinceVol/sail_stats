#![no_std]

use core::f32::consts::PI;

use defmt::dbg;
use defmt::info;
use defmt::println;
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

use crate::micro_sd::num_to_buffer;
use crate::micro_sd::BUFFER_LENGTH;
use crate::micro_sd::MICRO_QUEU;

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
    //Initializing I2C, will need to do this in main or elsewhere if we want to share
    //the i2c bus
    info!("Initializing accelorometer twi...");
    let config = twim::Config::default();
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

    info!("Starting to pull tilt data!");
    loop {
        Timer::after_millis(1000).await;
        let mut roll: f32 = 0.0;
        let mut pitch: f32 = 0.0;
        if sensor.accel_status().is_ok_and(|s| s.xyz_new_data()) {
            let data = sensor.acceleration().unwrap();
            let x = data.x_mg() as f32;
            let y = data.y_mg() as f32;
            let z = data.z_mg() as f32;

            //represents x tilt
            pitch = 180f32
                * libm::atan2f(x, libm::sqrtf(libm::powf(y, 2f32) + libm::powf(z, 2f32)))
                / PI;
            //represents y tilt
            roll = 180f32 * libm::atan2f(y, libm::sqrtf(libm::powf(x, 2f32) + libm::powf(z, 2f32)))
                / PI;

            let mut roll_buf: [u8; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
            num_to_buffer(roll, &mut roll_buf, 2);
            let mut pitch_buf: [u8; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
            num_to_buffer(pitch, &mut pitch_buf, 2);
            println!("Roll : {} \nPitch {}", roll_buf, pitch_buf);

            MICRO_QUEU.send((1, roll_buf)).await;
            MICRO_QUEU.send((2, pitch_buf)).await;
        }
    }
}
