//The purpose of this crate it to take care of anything related to data
//storage for the user to then take to their computer and crunch the
//results

use cortex_m::iprintln;
use defmt::{info, println};
use embassy_nrf::{
    bind_interrupts,
    peripherals::{P0_01, P0_13, P0_17, SPI2},
    spim, Peri,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Delay, Timer};
use embedded_sdmmc::{
    Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};

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
) {
    //Init SPIM --> may need to be something we do in main, especially if we want
    //to share the SPI bus
    info!("Initializing external spi bus...");
    let config = spim::Config::default();
    let spi = spim::Spim::new(spi_p, Irqs, sck_p, miso_p, mosi_p, config);

    let exclusive_spi =
        embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs_p, embassy_time::Delay).unwrap();

    //sdcard crate is nice enough to spam the sd card for us to get it in spi mode
    let sdcard = SdCard::new(exclusive_spi, Delay);

    info!("Card size is {} bytes", sdcard.num_bytes().unwrap());
    let time_source = RTCWrapper::new();
    let volume_mgr = VolumeManager::new(sdcard, time_source);
    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let root_dir = volume0.open_root_dir().unwrap();
    let my_other_file = root_dir
        .open_file_in_dir("MY_DATA.CSV", Mode::ReadWriteCreateOrAppend)
        .unwrap();
    my_other_file.write(b"Timestamp,Signal,Value\n").unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:00Z,TEMP,25.0\n")
        .unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:01Z,TEMP,25.1\n")
        .unwrap();
    my_other_file
        .write(b"2025-01-01T00:00:02Z,TEMP,25.2\n")
        .unwrap();
    // Don't forget to flush the file so that the directory entry is updated
    my_other_file.flush().unwrap();
}

//Trying to have a channel that implements a few functions like 'contains' and what not
//to drive what the sensors are doing --basically stop reading data once you've queued the
//channel
static MICRO_QUEU: Channel<CriticalSectionRawMutex, ([char; 4], [u8; 10]), 10> = Channel::new();
pub struct MicroQueu {
    //TODO
}

//Not sure what to contain within the struct given that the methods surrounding this wrapper
// are initialized in main || using var functional to call out whether the embassy time crate
// is currently functioning
struct RTCWrapper {
    _functional: bool,
}

impl RTCWrapper {
    fn new() -> RTCWrapper {
        //find TEST for whether or not embassy time is working properly
        RTCWrapper { _functional: true }
    }
}

//need a shitty function to convert ticks from embassy_time to hour/min/sec it's a fair
//assumption to say this sailing tracker wouldn't run on the dinghy for over 24hrs
fn sec_to_time(seconds: u64) -> (u8, u8, u8) {
    let hours: f64 = (seconds as f64) / (60f64 * 60f64);
    let minutes = (hours - ((hours as u8) as f64)) * 60f64;
    let seconds = (minutes - ((minutes as u8) as f64)) * 60f64;
    return (hours as u8, minutes as u8, seconds as u8);
}

impl TimeSource for RTCWrapper {
    fn get_timestamp(&self) -> embedded_sdmmc::Timestamp {
        let now = embassy_time::Instant::now().as_secs();
        let (hour, min, sec) = sec_to_time(now);
        if let Ok(time_stamp) = Timestamp::from_calendar(2025, 9, 19, hour, min, sec) {
            return time_stamp;
        } else {
            //Guaranteed realed time (SHOULD TEST TODO)
            return Timestamp::from_calendar(2025, 7, 28, 0, 0, 0).unwrap();
        }
    }
}
