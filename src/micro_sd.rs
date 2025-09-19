//The purpose of this crate it to take care of anything related to data
//storage for the user to then take to their computer and crunch the
//results

use core::{
    borrow::{Borrow, BorrowMut},
    fmt::Debug,
};

use defmt::info;
use embassy_nrf::{
    bind_interrupts,
    peripherals::{P0_01, P0_13, P0_17, RTC0, SPI2},
    spim, Peri,
};
use embassy_time::Delay;
use embedded_sdmmc::{Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager};

//Just like I2C not really sure what the difference between spi2 and spi3 is
//and when to use one or the other
bind_interrupts!(struct Irqs {
    SPI2 => spim::InterruptHandler<SPI2>;
});

#[embassy_executor::task]
pub async fn init_save(
    spi_p: Peri<'static, SPI2>,
    miso_p: Peri<'static, P0_01>,
    mosi_p: Peri<'static, P0_13>,
    sck_p: Peri<'static, P0_17>,
    cs_p: embassy_nrf::gpio::Output<'static>,
    timesrc_p: Peri<'static, RTC0>,
) {
    //Init SPIM --> may need to be something we do in main, especially if we want
    //to share the SPI bus
    info!("Initializing external spi bus...");
    let config = spim::Config::default();
    let mut spi = spim::Spim::new(spi_p, Irqs, sck_p, miso_p, mosi_p, config);

    let exclusive_spi =
        embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs_p, embassy_time::Delay).unwrap();

    let sdcard = SdCard::new(exclusive_spi, Delay);

    info!("Card size is {} bytes", sdcard.num_bytes().unwrap());
    let volume_mgr = VolumeManager::new(sdcard, timesrc_p);
    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    info!("Volume 0: {:?}", volume0);
    let root_dir = volume0.open_root_dir().unwrap();
    info!("we actually made it here!");
}

struct RTCWrapper {
    rtc: Peri<'static, RTC0>,
}

//need a shitty function to convert ticks from embassy_time to hour/min/sec it's a fair
//assumption to say this sailing tracker wouldn't run on the dinghy for over 24hrs
fn tick_to_time(ticks: u64) -> (u8, u8, u8) {
    let hours:f64 = (ticks as f64)/32768f64;
    let minutes = (hours - ((hours as u8) as f64)) * 60f64;
    let seconds = (minutes - ((minutes as u8) as f64)) * 60f64;    
    return (hours as u8,minutes as u8,seconds as u8);
}

impl  TimeSource for RTCWrapper {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        let now = embassy_time::Instant::now();
        let (hour,min,sec) = tick_to_time(now);
        let time_stamp = Timestamp::from_calendar(2025,9 ,19 ,6 hour, min, sec);
    }
}
