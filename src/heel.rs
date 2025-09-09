use defmt::info;
use embassy_nrf::{
    bind_interrupts,
    gpio::AnyPin,
    peripherals::{P0_28, P0_30, TWISPI0},
    twim, Peri,
};
use static_cell::ConstStaticCell;

//Having a pretty hard time determining the difference between TWISPI0 and TWISPI1, to the best of my knowledge they are both interrupts associated with using TWI (I2C)
//if we have problems with the accelerometer I'll swithc this to TWISPI1 to see if it makes a difference
bind_interrupts!(
    struct Irqs {
        TWISPI0 => twim::InterruptHandler<TWISPI0>;
    }
);

#[embassy_executor::task]
async fn init_heel(
    twi_p: Peri<'static, TWISPI0>,
    scl_p: Peri<'static, P0_28>,
    sda_p: Peri<'static, P0_30>,
) {
    info!("Initializing accelorometer twi...");
    let config = twim::Config::default();

    //need to determine if a buffer is really required for our use case
    static RAM_BUFFER: static_cell::ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let mut twi = twim::Twim::new(twi_p, Irqs, sda_p, scl_p, config, RAM_BUFFER.take());
}
