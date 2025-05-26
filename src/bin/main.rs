use dynamic_traits::{
    consumer::{self, AsPinsMut, AsUartMut, Pins},
    hal::{
        Peri, Peripherals,
        gpio::{Input, Instance, Output},
        uart::Uart,
    },
    traits::{AsInput, AsOutput},
};
use embassy_executor::Executor;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use static_cell::StaticCell;

macro_rules! impl_board {
    ($board:ident, $pin_rx:ident, $pin_tx:ident, $uart:ident) => {
        struct $board<'a> {
            pins: consumer::Pins<
                PinWrapper<'a, dynamic_traits::hal::peripherals::$pin_rx>,
                PinWrapper<'a, dynamic_traits::hal::peripherals::$pin_tx>,
            >,
            uart: Peri<'a, dynamic_traits::hal::peripherals::$uart>,
        }

        impl AsUartMut for $board<'_> {
            type Target<'a>
                = Uart<'a>
            where
                Self: 'a;

            fn as_uart(&mut self) -> Self::Target<'_> {
                Uart::new(
                    self.uart.reborrow(),
                    self.pins.rx.0.reborrow(),
                    self.pins.tx.0.reborrow(),
                )
            }
        }

        impl AsPinsMut for $board<'_> {
            type RX<'a>
                = &'a mut dyn DynPin
            where
                Self: 'a;
            type TX<'a>
                = &'a mut dyn DynPin
            where
                Self: 'a;

            fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
                Pins {
                    rx: &mut self.pins.rx,
                    tx: &mut self.pins.tx,
                }
            }
        }

        impl DynBoard for $board<'_> {
            fn as_pins_compat(&mut self) -> Pins<&'_ mut dyn DynPin, &'_ mut dyn DynPin> {
                AsPinsMut::as_pins(self)
            }

            fn as_uart_compat(&mut self) -> Uart<'_> {
                AsUartMut::as_uart(self)
            }
        }
    };
}

impl_board!(BoardA, PIN_A, PIN_B, UART0);
impl_board!(BoardB, PIN_B, PIN_C, UART1);
impl_board!(BoardC, PIN_C, PIN_D, UART2);

#[derive(Debug)]
enum Boards {
    A,
    B,
    C,
}

pub struct PinWrapper<'a, T: Instance>(Peri<'a, T>);

impl<'a, T: Instance> DynPin for PinWrapper<'a, T> {
    fn as_input_compat(&mut self) -> Input<'_> {
        Input::new(self.0.reborrow())
    }

    fn as_output_compat(&mut self) -> Output<'_> {
        Output::new(self.0.reborrow())
    }
}

trait DynPin {
    fn as_input_compat(&mut self) -> Input<'_>;
    fn as_output_compat(&mut self) -> Output<'_>;
}

impl<'a> AsInput for &'a mut dyn DynPin {
    type Target = Input<'a>;

    fn as_input(self) -> Self::Target {
        self.as_input_compat()
    }
}

impl<'a> AsOutput for &'a mut dyn DynPin {
    type Target = Output<'a>;

    fn as_output(self) -> Self::Target {
        self.as_output_compat()
    }
}

trait DynBoard {
    fn as_pins_compat(&mut self) -> Pins<&mut dyn DynPin, &mut dyn DynPin>;
    fn as_uart_compat(&mut self) -> Uart<'_>;
}

impl AsPinsMut for &mut dyn DynBoard {
    type RX<'a>
        = &'a mut dyn DynPin
    where
        Self: 'a;

    type TX<'a>
        = &'a mut dyn DynPin
    where
        Self: 'a;

    fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
        self.as_pins_compat()
    }
}

impl AsUartMut for &mut dyn DynBoard {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_uart(&mut self) -> Self::Target<'_> {
        self.as_uart_compat()
    }
}

#[embassy_executor::task]
async fn run() {
    let mut p = unsafe { Peripherals::steal() };
    {
        let mut output = Output::new(p.PIN_A.reborrow());
        output.set_high().unwrap();

        let _input = Input::new(p.PIN_A.reborrow());
    }

    loop {
        for board in [Boards::A, Boards::B, Boards::C] {
            log::info!("Board {:?}", board);

            let board: &mut dyn DynBoard = match board {
                Boards::A => &mut BoardA {
                    pins: Pins {
                        rx: PinWrapper(p.PIN_A.reborrow()),
                        tx: PinWrapper(p.PIN_B.reborrow()),
                    },
                    uart: p.UART0.reborrow(),
                },
                Boards::B => &mut BoardB {
                    pins: Pins {
                        rx: PinWrapper(p.PIN_B.reborrow()),
                        tx: PinWrapper(p.PIN_C.reborrow()),
                    },
                    uart: p.UART1.reborrow(),
                },
                Boards::C => &mut BoardC {
                    pins: Pins {
                        rx: PinWrapper(p.PIN_C.reborrow()),
                        tx: PinWrapper(p.PIN_D.reborrow()),
                    },
                    uart: p.UART2.reborrow(),
                },
            };

            embassy_futures::select::select(consumer::run(board), Timer::after_millis(100)).await;
        }

        Timer::after_secs(1).await;
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();
    });
}
