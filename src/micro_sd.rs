//The purpose of this crate it to take care of anything related to data
//storage for the user to then take to their computer and crunch the
//results

use cortex_m::prelude::_embedded_hal_digital_InputPin;
use defmt::info;
use embassy_nrf::{
    bind_interrupts,
    peripherals::{P0_01, P0_13, P0_17, SPI2},
    spim, Peri,
};
use embassy_time::Delay;
use embedded_sdmmc::{Error, Mode, SdCard, SdCardError, TimeSource, VolumeIdx, VolumeManager};

//Just like I2C not really sure what the difference between spi2 and spi3 is
//and when to use one or the other
bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<SPI2>;
});

#[embassy_executor::task]
async fn init_save(
    spi_p: Peri<'static, SPI2>,
    miso_p: Peri<'static, P0_01>,
    mosi_p: Peri<'static, P0_13>,
    sck_p: Peri<'static, P0_17>,
    cs_p: embassy_nrf::gpio::Output<'static>,
) {
    //Init SPIM --> may need to be something we do in main, especially if we want
    //to share the SPI bus
    info!("Initializing external spi bus...");
    let config = spim::Config::default();
    let mut spi = spim::Spim::new(spi_p, Irqs, sck_p, miso_p, mosi_p, config);

    let exclusive_spi =
        embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs_p, embassy_time::Delay).unwrap();

    let sdcard = SdCard::new(exclusive_spi, Delay);
}
