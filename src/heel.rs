use core::f32::consts::PI;

use defmt::info;
use defmt::println;
use emb_txt_hndlr::BufTxt;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

use embassy_time::Delay;
use embassy_time::Timer;
use lsm303agr::AccelOutputDataRate;
use lsm303agr::Lsm303agr;

use crate::micro_sd::MICRO_QUEU;

//Having a pretty hard time determining the difference between TWISPI0 and TWISPI1, to the best of my knowledge they are both peripherals associated with using TWI (I2C)
//if we have problems with the accelerometer I'll swithc this to TWISPI1 to see if it makes a difference

#[embassy_executor::task]
pub async fn init_heel(twi_bus: &'static crate::TwiBus) {
    //Following https://github.com/eldruin/lsm303agr-rs/blob/HEAD/examples/microbit-v2.rs this example for implementing accelorometer driver

    let twi_d = I2cDevice::new(twi_bus);
    let mut sensor = Lsm303agr::new_with_i2c(twi_d);
    sensor.init().await.unwrap();
    sensor
        .set_accel_mode_and_odr(
            &mut Delay,
            lsm303agr::AccelMode::LowPower,
            AccelOutputDataRate::Hz10,
        )
        .await
        .unwrap();

    info!("Starting to pull tilt data!");
    loop {
        Timer::after_millis(1000).await;
        let roll: f32;
        let pitch: f32;
        if sensor.accel_status().await.is_ok_and(|s| s.xyz_new_data()) {
            let data = sensor.acceleration().await.unwrap();
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

            let roll_buf = BufTxt::from_f(roll as f64, 6).unwrap();
            let pitch_buf = BufTxt::from_f(pitch as f64, 6).unwrap();
            // println!("Roll : {} \nPitch {}", roll_buf, pitch_buf);

            MICRO_QUEU.send((1, roll_buf)).await;
            MICRO_QUEU.send((2, pitch_buf)).await;
            // MICRO_QUEU.send((1, *b"ROLLBUFF")).await;
            // MICRO_QUEU.send((2, *b"PITCHBUF")).await;
        }
    }
}

#[embassy_executor::task]
pub async fn init_mag(twi_bus: &'static crate::TwiBus) {
    //Following https://github.com/eldruin/lsm303agr-rs/blob/HEAD/examples/microbit-v2.rs this example for implementing accelorometer driver

    let twi_d = I2cDevice::new(twi_bus);
    let mut sensor = Lsm303agr::new_with_i2c(twi_d);
    sensor.init().await.unwrap();
    sensor
        .set_mag_mode_and_odr(
            &mut Delay,
            lsm303agr::MagMode::LowPower,
            lsm303agr::MagOutputDataRate::Hz100,
        )
        .await
        .unwrap();

    info!("Starting to pull Compass data!");
    loop {
        Timer::after_millis(1000).await; //refresh rate for sensor
        if sensor.mag_status().await.is_ok() {
            let data = sensor.magnetic_field().await.unwrap();
            let x = data.x_nt() as f64;
            let y = data.y_nt() as f64;
            // let z = data.z_nt() as f64;

            let heading = libm::atan2(y, x) * 180.0 / 3.1415;
            let heading_buf = BufTxt::from_f(heading, 6).unwrap();
            println!("heading : {}", heading);

            MICRO_QUEU.send((3, heading_buf)).await;
            // MICRO_QUEU.send((1, *b"ROLLBUFF")).await;
            // MICRO_QUEU.send((2, *b"PITCHBUF")).await;
        }
    }
}
