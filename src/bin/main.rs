use dynamic_traits::{
    consumer::{self, AsPinsMut, AsUartMut, Pins},
    dynamic::{Constructor, DynEither, DynThief, Owned},
    hal::{
        Peri, Peripherals,
        gpio::{self, Input, Output},
        uart::Uart,
    },
    traits::{AsInput, AsIoReadWriteDevice, AsOutput},
};
use embassy_executor::Executor;
use embassy_time::Timer;
use embedded_hal::digital::OutputPin;
use static_cell::StaticCell;

macro_rules! impl_board {
    ($board:ident, $pin_rx:ident, $pin_tx:ident, $uart:ident) => {
        struct $board<'a> {
            pins: consumer::Pins<
                Peri<'a, dynamic_traits::hal::peripherals::$pin_rx>,
                Peri<'a, dynamic_traits::hal::peripherals::$pin_tx>,
            >,
            uart: Peri<'a, dynamic_traits::hal::peripherals::$uart>,
        }

        impl<'a> AsIoReadWriteDevice for $board<'a> {
            type Target = Uart<'a>;

            fn as_io_read_write(self) -> Self::Target {
                Uart::new(self.uart, self.pins.rx, self.pins.tx)
            }
        }

        impl AsUartMut for $board<'_> {
            type Target<'a>
                = Uart<'a>
            where
                Self: 'a;

            fn as_uart(&mut self) -> Self::Target<'_> {
                Uart::new(
                    self.uart.reborrow(),
                    self.pins.rx.reborrow(),
                    self.pins.tx.reborrow(),
                )
            }
        }

        impl AsPinsMut for $board<'_> {
            type RX<'a>
                = DynPin<'a>
            where
                Self: 'a;
            type TX<'a>
                = DynPin<'a>
            where
                Self: 'a;

            fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
                Pins {
                    rx: DynPin::new(self.pins.rx.reborrow()),
                    tx: DynPin::new(self.pins.tx.reborrow()),
                }
            }
        }

        impl DynCompatDualMode for $board<'_> {
            fn as_pins_compat(&mut self) -> Pins<DynPin<'_>, DynPin<'_>> {
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

struct InputConstructor;

impl<'a, T: gpio::Instance> Constructor<'a, Peri<'a, T>> for InputConstructor {
    type To = Input<'a>;

    fn convert(from: Owned<'a, Peri<'a, T>>) -> Owned<'a, Self::To> {
        from.map(|from| Input::new(from))
    }
}

struct OutputConstructor;

impl<'a, T: gpio::Instance> Constructor<'a, Peri<'a, T>> for OutputConstructor {
    type To = Output<'a>;

    fn convert(from: Owned<'a, Peri<'a, T>>) -> Owned<'a, Self::To> {
        from.map(|from| Output::new(from))
    }
}

impl<'a> DynPin<'a> {
    pub fn new(pin: Peri<'a, impl gpio::Instance>) -> Self {
        Self(DynEither::new_owned::<_, InputConstructor, OutputConstructor>(Owned::new(pin)))
    }
}

struct DynPin<'a>(DynEither<'a, Input<'a>, Output<'a>>);

impl<'a> Into<DynEither<'a, Input<'a>, Output<'a>>> for DynPin<'a> {
    fn into(self) -> DynEither<'a, Input<'a>, Output<'a>> {
        self.0
    }
}

impl<'a> AsInput for DynPin<'a> {
    type Target = Input<'a>;

    fn as_input(self) -> Self::Target {
        let value: DynThief<'a, Input<'a>> = self.0.left();
        let value: Owned<'a, Input<'a>> = value.build();
        Into::into(value)
    }
}

impl<'a> AsOutput for DynPin<'a> {
    type Target = Output<'a>;

    fn as_output(self) -> Self::Target {
        let value: DynThief<'_, Output<'a>> = self.0.right();
        let value: Owned<'_, Output<'_>> = value.build();
        Into::into(value)
    }
}

trait DynCompatDualMode {
    fn as_pins_compat(&mut self) -> Pins<DynPin<'_>, DynPin<'_>>;
    fn as_uart_compat(&mut self) -> Uart<'_>;
}

impl AsPinsMut for &mut dyn DynCompatDualMode {
    type RX<'a>
        = DynPin<'a>
    where
        Self: 'a;

    type TX<'a>
        = DynPin<'a>
    where
        Self: 'a;

    fn as_pins(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
        self.as_pins_compat()
    }
}

impl AsUartMut for &mut dyn DynCompatDualMode {
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

            let board: &mut dyn DynCompatDualMode = match board {
                Boards::A => &mut BoardA {
                    pins: Pins {
                        rx: p.PIN_A.reborrow(),
                        tx: p.PIN_B.reborrow(),
                    },
                    uart: p.UART0.reborrow(),
                },
                Boards::B => &mut BoardB {
                    pins: Pins {
                        rx: p.PIN_B.reborrow(),
                        tx: p.PIN_C.reborrow(),
                    },
                    uart: p.UART1.reborrow(),
                },
                Boards::C => &mut BoardC {
                    pins: Pins {
                        rx: p.PIN_C.reborrow(),
                        tx: p.PIN_D.reborrow(),
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
