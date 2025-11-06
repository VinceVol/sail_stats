//File meant for uarte communication with arduino GPS Module

use core::f32;

use defmt::println;
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
        let res = uart.read(&mut buffer).await;

        if buffer != [0; GPS_BUF_SIZE] {
            println!("{}", core::str::from_utf8(&buffer).unwrap());
        }
    }
}
