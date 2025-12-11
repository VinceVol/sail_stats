//File meant for uarte communication with arduino GPS Module

use defmt::println;
use emb_txt_hndlr::{BUF_LENGTH, BufTxt};
use embassy_nrf::{
    Peri, bind_interrupts,
    gpio::AnyPin,
    peripherals::UARTE0,
    uarte::{self, Baudrate, Uarte},
};

use crate::micro_sd::MICRO_QUEU;

//baud rate for arduino gps is defaulted to 9600
//GPGGA,113727.00,4303.16727,N,08612.65632,W,1,07,1.43,197.6,M,-34.5,M,,*60
struct Gps {
    deg_lat: BufTxt,
    deg_long: BufTxt,
    date_time: BufTxt, //utc time
    altitude: BufTxt,
    altitude_units: BufTxt,
}

bind_interrupts!(struct Irua {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

const GPS_BUF_SIZE: usize = 256; //starting with this till issues arise

fn parse_gpgga(gpgga: [BufTxt; 15]) -> Option<Gps> {
    let utc_t_r = gpgga[1].to_str()?;
    let utc_time = BufTxt::concat_list(&[
        BufTxt::from_str(&utc_t_r[0..2]).ok()?,
        BufTxt::from_str(":").unwrap(),
        BufTxt::from_str(&utc_t_r[2..4]).ok()?,
        BufTxt::from_str(":").unwrap(),
        BufTxt::from_str(&utc_t_r[4..6]).ok()?,
    ])
    .ok()?;

    let lat_raw: f64 = gpgga[2].to_str()?.parse().ok()?;
    let long_raw: f64 = gpgga[4].to_str()?.parse().ok()?;
    let altitude = gpgga[9];
    let altitude_units = gpgga[10];

    //Convert Latitude to decimal degrees
    let lat_degrees = ((lat_raw / 100f64) as u64) as f64;
    let lat_minutes = lat_raw % 100f64;
    let mut latitude = lat_degrees + (lat_minutes / 60f64);
    if gpgga[3] == BufTxt::from_str("S").unwrap() {
        latitude *= -1f64;
    }

    //Convert Longitude to decimal degrees
    let long_degrees = ((long_raw / 100f64) as u64) as f64;
    let long_minutes = long_raw % 100f64;
    let mut longitude = long_degrees + (long_minutes / 60f64);
    if gpgga[3] == BufTxt::from_str("W").unwrap() {
        longitude *= -1f64;
    }
    return Some(Gps {
        deg_lat: BufTxt::from_f(latitude, 7).ok()?,
        deg_long: BufTxt::from_f(longitude, 7).ok()?,
        date_time: utc_time,
        altitude,
        altitude_units,
    });
}

#[embassy_executor::task]
pub async fn init_gps(
    tx: Peri<'static, AnyPin>,
    rx: Peri<'static, AnyPin>,
    uart_p: Peri<'static, UARTE0>,
) {
    //initialize the UARTE communications
    let mut config = uarte::Config::default();
    config.baudrate = Baudrate::BAUD9600; //GPS Module defaults to this
    let mut uart = Uarte::new(uart_p, rx, tx, Irua, config);

    //dump the UARTE info into a buffer -- and zap/parse that info to sd card
    loop {
        let mut buffer: [u8; GPS_BUF_SIZE] = [0; GPS_BUF_SIZE];
        let _res = uart.read(&mut buffer).await;

        if buffer != [0; GPS_BUF_SIZE] {
            let data_chunk = BufTxt::from_u8(&buffer).unwrap();
            let mut chunks: [BufTxt; 15] = [BufTxt::default(); 15];
            let _ = data_chunk.split('\n' as u8, &mut chunks);
            for chunk in chunks {
                let mut split_chunk = [BufTxt::default(); 15];
                let _ = chunk.split(',' as u8, &mut split_chunk);
                if let Some(identifier) = split_chunk[0].to_str() {
                    if identifier == "$GPGGA" && split_chunk[14] != BufTxt::default() {
                        if let Some(gps_info) = parse_gpgga(split_chunk) {
                            MICRO_QUEU.send((0, gps_info.date_time)).await;
                            MICRO_QUEU.send((4, gps_info.deg_lat)).await;
                            MICRO_QUEU.send((5, gps_info.deg_long)).await;
                            MICRO_QUEU.send((6, gps_info.altitude)).await;
                            MICRO_QUEU.send((7, gps_info.altitude_units)).await;
                        }
                    }
                }
            }
        }
    }
}
