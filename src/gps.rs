//File meant for uarte communication with arduino GPS Module

use core::f32;

use defmt::println;
use emb_txt_hndlr::{BUF_LENGTH, BufTxt};
use embassy_nrf::{
    Peri, bind_interrupts,
    gpio::AnyPin,
    peripherals::UARTE0,
    uarte::{self, Baudrate, Uarte},
};

//baud rate for arduino gps is defaulted to 9600
//GPGGA,113727.00,4303.16727,N,08612.65632,W,1,07,1.43,197.6,M,-34.5,M,,*60
struct Gps {
    deg_north: BufTxt,
    deg_west: BufTxt,
    date_time: BufTxt, //utc time
    altitude: BufTxt,
    altitude_units: BufTxt,
}

bind_interrupts!(struct Irua {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

const GPS_BUF_SIZE: usize = 256; //starting with this till issues arise

fn parse_gpgga(gpgga: [BufTxt; 15]) -> Option<Gps> {
    let utc_time = gpgga[1];
    let lat_raw = core::str::from_utf8(&gpgga[2].characters)
        .unwrap()
        .parse()
        .ok()?;
    let long_raw = core::str::from_utf8(&gpgga[4].characters)
        .unwrap()
        .parse()
        .ok()?;
    let altitude = gpgga[9];
    let altitude_units = gpgga[10];

    //Convert Latitude to decimal degrees
    let lat_degrees = ((lat_raw / 100f64) as u64) as f64;
    let lat_minutes = lat_raw % 100f64;
    let mut latitude = lat_degrees + (lat_minutes / 60f64);
    if gpgga[3] == BufTxt::from_str("S").unwrap() {
        latitude *= -1;
    }

    //Convert Longitude to decimal degrees
    let long_degrees = ((long_raw / 100f64) as u64) as f64;
    let long_minutes = long_raw % 100f64;
    let mut longitude = long_degrees + (long_minutes / 60f64);
    if gpgga[3] == BufTxt::from_str("W").unwrap() {
        longitude *= -1;
    }
    return Some(());

    // latitude_hemisphere = fields[3]
    // longitude_hemisphere = fields[5]

    // # Convert latitude to decimal degrees
    // lat_degrees = int(latitude_raw / 100)
    // lat_minutes = latitude_raw % 100
    // latitude = lat_degrees + (lat_minutes / 60)
    // if latitude_hemisphere == 'S':
    //     latitude *= -1

    // # Convert longitude to decimal degrees
    // lon_degrees = int(longitude_raw / 100)
    // lon_minutes = longitude_raw % 100
    // longitude = lon_degrees + (lon_minutes / 60)
    // if longitude_hemisphere == 'W':
    //     longitude *= -1
    //
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
                if split_chunk[0].characters[BUF_LENGTH - 5..BUF_LENGTH]
                    == ['G' as u8, 'P' as u8, 'G' as u8, 'G' as u8, 'A' as u8]
                    && split_chunk[14] != BufTxt::default()
                {
                    for i in 0..15 {
                        println!(
                            "[{}]: {}",
                            i,
                            core::str::from_utf8(
                                &split_chunk[i].characters[BUF_LENGTH - 5..BUF_LENGTH]
                            )
                            .unwrap()
                        );
                    }
                }
            }
        }
    }
}
