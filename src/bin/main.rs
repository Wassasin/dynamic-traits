use dynamic_traits::{
    consumer::{self, AsPins, Dependency, Pins},
    hal::{
        Peripherals,
        gpio::{Input, Output},
        uart::Uart,
    },
    traits::AsIoReadWriteDevice,
};
use embedded_hal::digital::OutputPin;

macro_rules! impl_board {
    ($board:ident, $pin_rx:ident, $pin_tx:ident, $uart:ident) => {
        struct $board<'a> {
            pins: consumer::Pins<
                &'a mut dynamic_traits::hal::peripherals::$pin_rx,
                &'a mut dynamic_traits::hal::peripherals::$pin_tx,
            >,
            uart: &'a mut dynamic_traits::hal::peripherals::$uart,
        }

        impl<'a> AsPins for $board<'a> {
            type RX = &'a mut dynamic_traits::hal::peripherals::$pin_rx;
            type TX = &'a mut dynamic_traits::hal::peripherals::$pin_tx;

            fn as_pins<'b>(&'b mut self) -> &'b mut consumer::Pins<Self::RX, Self::TX> {
                &mut self.pins
            }
        }

        impl AsIoReadWriteDevice for $board<'_> {
            type Target<'a>
                = Uart<'a>
            where
                Self: 'a;

            fn as_io_read_write(&mut self) -> Self::Target<'_> {
                Uart::new(&mut self.uart, &mut self.pins.rx, &mut self.pins.tx)
            }
        }

        impl Dependency for $board<'_> {}
    };
}

impl_board!(BoardA, PIN_A, PIN_B, UART0);
impl_board!(BoardB, PIN_C, PIN_D, UART1);

fn main() {
    let mut p = unsafe { Peripherals::steal() };
    {
        let mut output = Output::new(&mut p.PIN_A);
        output.set_high().unwrap();

        let _input = Input::new(&mut p.PIN_A);
    }

    let board = BoardA {
        pins: Pins {
            rx: &mut p.PIN_A,
            tx: &mut p.PIN_B,
        },
        uart: &mut p.UART0,
    };

    let future = consumer::run(board);
    drop(future);
}
