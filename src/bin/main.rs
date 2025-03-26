use dynamic_traits::{
    consumer::{self, AsPins, Dependency, Pins},
    hal::{
        Peripherals,
        gpio::{Input, Output},
        peripherals::{PIN_A, PIN_B, UART0},
        uart::Uart,
    },
    traits::AsIoReadWriteDevice,
};
use embedded_hal::digital::OutputPin;

struct PlatformManager<'a> {
    pins: consumer::Pins<&'a mut PIN_A, &'a mut PIN_B>,
    uart: &'a mut UART0,
}

impl<'a> AsPins for PlatformManager<'a> {
    type RX = &'a mut PIN_A;
    type TX = &'a mut PIN_B;

    fn as_pins<'b>(&'b mut self) -> &'b mut consumer::Pins<&'a mut PIN_A, &'a mut PIN_B> {
        &mut self.pins
    }
}

impl AsIoReadWriteDevice for PlatformManager<'_> {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_io_read_write<'a>(&'a mut self) -> Self::Target<'a> {
        Uart::new(&mut self.uart, &mut self.pins.rx, &mut self.pins.tx)
    }
}

impl Dependency for PlatformManager<'_> {}

fn main() {
    let mut p = unsafe { Peripherals::steal() };
    {
        let mut output = Output::new(&mut p.PIN_A);
        output.set_high().unwrap();

        let input = Input::new(&mut p.PIN_A);
        drop(input);
    }

    let platform = PlatformManager {
        pins: Pins {
            rx: &mut p.PIN_A,
            tx: &mut p.PIN_B,
        },
        uart: &mut p.UART0,
    };

    let future = consumer::run(platform);
    drop(future);
}
