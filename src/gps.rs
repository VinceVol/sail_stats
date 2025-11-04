//File meant for uarte communication with arduino GPS Module

use defmt::println;
use embassy_nrf::{
    Peri, bind_interrupts,
    gpio::AnyPin,
    peripherals::UARTE0,
    uarte::{self, Baudrate, Uarte},
};

//baud rate for arduino gps is defaulted to 9600
struct Gps {}

bind_interrupts!(struct Irua {
    UARTE0 => uarte::InterruptHandler<UARTE0>;
});

const GPS_BUF_SIZE: usize = 64; //starting with this till issues arise

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
        let mut buffer: [u8; GPS_BUF_SIZE] = [' ' as u8; 64];
        let _ = uart.read(&mut buffer).await;
        println!("{}", core::str::from_utf8(&buffer).unwrap());
    }
}
