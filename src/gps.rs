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
    deg_north: f32,
    deg_west: f32,
    time: f32,
    date: f32,
}

bind_interrupts!(struct Irua {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

const GPS_BUF_SIZE: usize = 256; //starting with this till issues arise

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
                    //TODO
                    // utc_time = fields[1]
                    // latitude_raw = float(fields[2])
                    // latitude_hemisphere = fields[3]
                    // longitude_raw = float(fields[4])
                    // longitude_hemisphere = fields[5]
                    // gps_quality = int(fields[6])
                    // num_satellites = int(fields[7])
                    // hdop = float(fields[8])
                    // altitude = float(fields[9])
                    // altitude_units = fields[10]

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
