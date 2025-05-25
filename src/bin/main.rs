use dynamic_traits::{
    consumer::{self, AsPinsMut, AsUartMut, Dependency, Pins},
    dynamic::{Constructor, DynEither, DynThief, Owned, OwnedEraseable},
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

        impl<'a> OwnedEraseable<'a> for $board<'a> {
            unsafe fn magick() -> dynamic_traits::dynamic::Owned<'a, Self> {
                unsafe {
                    Owned::new($board {
                        pins: Pins {
                            rx: dynamic_traits::hal::peripherals::$pin_rx::steal(),
                            tx: dynamic_traits::hal::peripherals::$pin_tx::steal(),
                        },
                        uart: dynamic_traits::hal::peripherals::$uart::steal(),
                    })
                }
            }
        }

        impl<'a> Constructor<'a, $board<'a>> for UartConstructor {
            type To = Uart<'a>;

            fn convert(from: Owned<'a, $board<'a>>) -> Owned<'a, Self::To> {
                from.map(|from| Uart::new(from.uart, from.pins.rx, from.pins.tx))
            }
        }

        impl<'a> Constructor<'a, $board<'a>> for PinsConstructor {
            type To = Pins<DynPin<'a>, DynPin<'a>>;

            fn convert(from: Owned<'a, $board<'a>>) -> Owned<'a, Self::To> {
                from.map(|from| Pins {
                    rx: DynPin::new(from.pins.rx),
                    tx: DynPin::new(from.pins.tx),
                })
            }
        }

        impl<'a> Into<DynBoard<'a>> for $board<'a> {
            fn into(self) -> DynBoard<'a> {
                DynBoard {
                    inner: DynEither::new_owned::<_, PinsConstructor, UartConstructor>(Owned::new(
                        self,
                    )),
                }
            }
        }
    };
}

impl_board!(BoardA, PIN_A, PIN_B, UART0);
impl_board!(BoardB, PIN_B, PIN_C, UART1);
impl_board!(BoardC, PIN_C, PIN_D, UART2);

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

/// How to construct Uart device from a Board.
struct UartConstructor;

/// How to construct the discrete pin set from a Board.
struct PinsConstructor;

#[derive(Debug)]
enum Boards {
    A,
    B,
    C,
}

impl<'a> DynBoard<'a> {
    pub fn select(p: &'a mut Peripherals, board: Boards) -> Self {
        match board {
            Boards::A => BoardA {
                pins: Pins {
                    rx: p.PIN_A.reborrow(),
                    tx: p.PIN_B.reborrow(),
                },
                uart: p.UART0.reborrow(),
            }
            .into(),
            Boards::B => BoardB {
                pins: Pins {
                    rx: p.PIN_B.reborrow(),
                    tx: p.PIN_C.reborrow(),
                },
                uart: p.UART1.reborrow(),
            }
            .into(),
            Boards::C => BoardC {
                pins: Pins {
                    rx: p.PIN_C.reborrow(),
                    tx: p.PIN_D.reborrow(),
                },
                uart: p.UART2.reborrow(),
            }
            .into(),
        }
    }
}

struct DynPin<'a>(DynEither<'a, Input<'a>, Output<'a>>);

impl<'a> Into<DynEither<'a, Input<'a>, Output<'a>>> for DynPin<'a> {
    fn into(self) -> DynEither<'a, Input<'a>, Output<'a>> {
        self.0
    }
}

impl<'a> AsInput<'a> for DynPin<'a> {
    type Target = Input<'a>;

    fn as_input(value: Owned<'a, Self>) -> Self::Target {
        let value: Owned<'a, DynEither<'a, Input<'a>, Output<'a>>> = Owned::into(value);
        let value: DynEither<'a, Input<'a>, Output<'a>> = Into::into(value);
        let value: DynThief<'a, Input<'a>> = value.left();
        let value: Owned<'a, Input<'a>> = value.build();
        Into::into(value)
    }
}

impl<'a> AsOutput<'a> for DynPin<'a> {
    type Target = Output<'a>;

    fn as_output(value: Owned<'a, Self>) -> Self::Target {
        let value: Owned<'a, DynEither<'a, Input<'a>, Output<'a>>> = Owned::into(value);
        let value: DynEither<'a, Input<'a>, Output<'a>> = Into::into(value);
        let value: DynThief<'_, Output<'a>> = value.right();
        let value: Owned<'_, Output<'_>> = value.build();
        Into::into(value)
    }
}

struct DynBoard<'a> {
    inner: DynEither<'a, Pins<DynPin<'a>, DynPin<'a>>, Uart<'a>>,
}

impl<'a> Into<DynEither<'a, Pins<DynPin<'a>, DynPin<'a>>, Uart<'a>>> for DynBoard<'a> {
    fn into(self) -> DynEither<'a, Pins<DynPin<'a>, DynPin<'a>>, Uart<'a>> {
        self.inner
    }
}

impl AsPinsMut for DynBoard<'_> {
    type RX<'a>
        = DynPin<'a>
    where
        Self: 'a;
    type TX<'a>
        = DynPin<'a>
    where
        Self: 'a;

    fn as_pins_mut<'a>(&mut self) -> Pins<Owned<'a, Self::RX<'a>>, Owned<'a, Self::TX<'a>>>
    where
        Self: 'a,
    {
        let value: DynEither<'_, _, _> = self.inner.reborrow();
        let value: DynThief<'_, Pins<DynPin<'a>, _>> = value.left();
        let value: Owned<'_, Pins<DynPin<'a>, DynPin<'a>>> = value.build();
        let value: Pins<DynPin<'a>, DynPin<'a>> = Into::into(value);

        Pins {
            rx: Owned::new(value.rx),
            tx: Owned::new(value.tx),
        }
    }
}

struct UartPrecursor<'a>(Uart<'a>);

impl AsUartMut for DynBoard<'_> {
    type Target<'a>
        = UartPrecursor<'a>
    where
        Self: 'a;

    fn as_uart_mut<'a, 'b: 'a>(&'b mut self) -> Self::Target<'a>
    where
        Self: 'a,
    {
        let value: DynEither<'b, _, _> = self.inner.reborrow();
        let value: DynThief<'b, Uart<'a>> = value.right();
        let value: Owned<'b, Uart<'a>> = value.build();
        let value: Uart<'a> = Into::into(value);

        UartPrecursor(value)
    }
}

impl<'a> From<UartPrecursor<'a>> for Uart<'a> {
    fn from(value: UartPrecursor<'a>) -> Self {
        value.0
    }
}

impl<'a> AsIoReadWriteDevice<'a> for UartPrecursor<'a> {
    type Target = Uart<'a>;
    fn as_io_read_write(self) -> Self::Target
    where
        Self: 'a,
    {
        Into::into(self)
    }
}

impl Dependency for DynBoard<'_> {}

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

            let board = DynBoard::select(&mut p, board);
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
