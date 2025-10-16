//The purpose of this crate it to take care of anything related to data
//storage for the user to then take to their computer and crunch the
//results

use core::{fmt::Pointer, num};

use cortex_m::iprintln;
use defmt::{info, println};
use emb_txt_hndlr::{BufError, BufTxt};
use embassy_nrf::{
    Peri, bind_interrupts,
    peripherals::{P0_01, P0_13, P0_17, SPI2},
    spim,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Delay, Instant, Timer};
use embedded_sdmmc::{
    Error, Mode, SdCard, SdCardError, TimeSource, Timestamp, VolumeIdx, VolumeManager,
};
use num_traits::{Float, FromPrimitive};

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
    let time_source = RTCWrapper::new();
    let volume_mgr = VolumeManager::new(sdcard, time_source);
    let volume0 = volume_mgr.open_volume(VolumeIdx(0)).unwrap();
    let root_dir = volume0.open_root_dir().unwrap();
    let my_other_file = root_dir
        .open_file_in_dir("MY_DATA.CSV", Mode::ReadWriteCreateOrAppend)
        .unwrap();

    //Sd card has been initialized
    for header in CSV_HEADERS {
        my_other_file.write(header).unwrap();
        my_other_file.write(b",").unwrap();
    }
    my_other_file.write(b"\n").unwrap();
    loop {
        //setting the refresh rate of all data collected written to micro_sd
        Timer::after_secs(1).await;
        if !MICRO_QUEU.is_empty() {
            let mut empty_q: [(u8, BufTxt); Q_SIZE] =
                [((Q_SIZE as u8) + 1, BufTxt::new(" ").unwrap()); Q_SIZE];

            //using current _q var thinking it prevents the q from changing size as you're
            //reading it. Maybe this isn't possible anyway?
            let mut i = 0;
            loop {
                empty_q[i] = MICRO_QUEU.receive().await;
                i += 1;
                if MICRO_QUEU.is_empty() {
                    break;
                }
            }

            //get the time
            empty_q[Q_SIZE - 1] = (0, buf_time_now().unwrap());

            //sort by col
            empty_q.sort_unstable_by_key(|d| d.0);

            //150 marking the maximum length of each line? not sure what this really needs to be
            let mut line: [u8; 150] = [0; 150];
            let mut l_p = 1; //index within line
            let mut act_col = 0; //if data is missing we need to add a ghost col -- this keeps track

            // info!("Start Read!");
            for (col, data) in empty_q {
                //     println!(
                //         "col: {} \nData: {}",
                //         col,
                //         core::str::from_utf8(&data).unwrap()
                //     );
                if data == BufTxt::default() {
                    continue;
                }
                while col > act_col {
                    line[l_p] = ',' as u8;
                    l_p += 1;
                    act_col += 1;
                }
                for c in data.characters {
                    //if c isnt 0 in buf u8
                    if c != ' ' as u8 {
                        line[l_p] = c;
                        l_p += 1;
                    }
                }
            }
            line[l_p] = '\n' as u8;
            // info!("End Read!");
            my_other_file.write(&line).unwrap();
            // Don't forget to flush the file so that the directory entry is updated
            my_other_file.flush().unwrap();
        }
    }
}

pub static BUFFER_LENGTH: usize = 8;
const Q_SIZE: usize = 20;
pub static MICRO_QUEU: Channel<CriticalSectionRawMutex, (u8, BufTxt), Q_SIZE> = Channel::new();
//we don't have hashmaps in a no_std environment so it's easier to sidestep this and hardcode in the

//header values and columns TODO -> add macro for header names
const CSV_HEADERS: [&[u8; 5]; 3] = [b"Time ", b"Roll ", b"Pitch"];
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

//Removed this from RTCWrapper since it only needs embassy time
fn buf_time_now() -> Result<BufTxt, BufError> {
    //Grab time and clunky convert to u8 -- should probably generate a small crate so we can test
    let now = embassy_time::Instant::now().as_secs();
    let clock_time = sec_to_time(now);

    //Initialized everything to buf txt
    let hr = BufTxt::from_u(clock_time.0)?;
    let min = BufTxt::from_u(clock_time.1)?;
    let sec = BufTxt::from_u(clock_time.2)?;
    let colon = BufTxt::new(":")?;

    //clunky concat all together in one line -- could use a function for this in EmbTxt
    let hr_min_sec = BufTxt::concat(
        BufTxt::concat(BufTxt::concat(BufTxt::concat(hr, colon)?, min)?, colon)?,
        sec,
    );
    return hr_min_sec;
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
