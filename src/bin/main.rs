use std::{convert::Infallible, marker::PhantomData};

use dynamic_traits::{
    consumer::{self, AsPinsMut, Dependency, Pins},
    dynamic::{DynEither, DynThief, Owned, OwnedEraseable},
    hal::{
        Peri, Peripherals,
        gpio::{self, Input, Output},
        peripherals::{PIN_A, PIN_B, UART0},
        uart::Uart,
    },
    traits::{AsInput, AsIoReadWriteDevice, AsOutput},
};
use embassy_executor::Executor;
use embassy_hal_internal::PeripheralType;
use embassy_time::Timer;
use embedded_hal::digital::{InputPin, OutputPin};
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

        impl<'a> AsPinsMut for $board<'a> {
            type RX<'b>
                = Peri<'b, dynamic_traits::hal::peripherals::$pin_rx>
            where
                Self: 'b;
            type TX<'b>
                = Peri<'b, dynamic_traits::hal::peripherals::$pin_tx>
            where
                Self: 'b;

            fn as_pins_mut(&mut self) -> consumer::Pins<Self::RX<'_>, Self::TX<'_>> {
                consumer::Pins {
                    rx: self.pins.rx.reborrow(),
                    tx: self.pins.tx.reborrow(),
                }
            }
        }

        // impl<'a> From<$board<'a>> for DynBoard<'a> {
        //     fn from(value: $board<'a>) -> Self {
        //         Self {
        //             pins: Pins {
        //                 rx: DynPin::from(value.pins.rx),
        //                 tx: DynPin::from(value.pins.tx),
        //             },
        //             uart: DynBoard::new(value),
        //         }
        //     }
        // }

        impl AsIoReadWriteDevice for $board<'_> {
            type Target<'a>
                = Uart<'a>
            where
                Self: 'a;

            fn as_io_read_write(&mut self) -> Self::Target<'_> {
                Uart::new(
                    self.uart.reborrow(),
                    self.pins.rx.reborrow(),
                    self.pins.tx.reborrow(),
                )
            }
        }

        impl Dependency for $board<'_> {}
    };
}

impl_board!(BoardA, PIN_A, PIN_B, UART0);
impl_board!(BoardB, PIN_B, PIN_C, UART1);
impl_board!(BoardC, PIN_C, PIN_D, UART2);

impl OwnedEraseable for BoardA<'_> {
    unsafe fn steal<'a>() -> dynamic_traits::dynamic::Owned<'a, Self> {
        unsafe {
            Owned::new(BoardA {
                pins: Pins {
                    rx: PIN_A::steal(),
                    tx: PIN_B::steal(),
                },
                uart: UART0::steal(),
            })
        }
    }
}

impl<'a> From<BoardA<'a>> for DynBoard<'a> {
    fn from(mut value: BoardA<'a>) -> Self {
        Self {
            inner: DynEither::new(&mut value),
        }
        // Self {
        //     pins: Pins {
        //         rx: DynPin::from(value.pins.rx),
        //         tx: DynPin::from(value.pins.tx),
        //     },
        //     uart: DynBoard::new(value),
        // }
    }
}

#[derive(Debug)]
enum Boards {
    A,
    B,
    C,
}

impl<'a> DynBoard<'a> {
    pub fn select(p: &'a mut Peripherals, board: Boards) -> Self {
        // match board {
        //     Boards::A => BoardA {
        //         pins: Pins {
        //             rx: p.PIN_A.reborrow(),
        //             tx: p.PIN_B.reborrow(),
        //         },
        //         uart: p.UART0.reborrow(),
        //     }
        //     .into(),
        //     Boards::B => BoardB {
        //         pins: Pins {
        //             rx: p.PIN_B.reborrow(),
        //             tx: p.PIN_C.reborrow(),
        //         },
        //         uart: p.UART1.reborrow(),
        //     }
        //     .into(),
        //     Boards::C => BoardC {
        //         pins: Pins {
        //             rx: p.PIN_C.reborrow(),
        //             tx: p.PIN_D.reborrow(),
        //         },
        //         uart: p.UART2.reborrow(),
        //     }
        //     .into(),
        // }
        todo!()
    }
}

struct DynPin<'a>(DynEither<'a, Input<'a>, Output<'a>>);

impl AsInput for DynPin<'_> {
    type Target<'a>
        = Input<'a>
    where
        Self: 'a;

    fn as_input(&mut self) -> Self::Target<'_> {
        self.0.build_left().into()
    }
}

impl AsOutput for DynPin<'_> {
    type Target<'a>
        = Output<'a>
    where
        Self: 'a;

    fn as_output(&mut self) -> Self::Target<'_> {
        self.0.build_right().into()
    }
}

struct DynBoard<'a> {
    inner: DynEither<'a, Pins<DynPin<'a>, DynPin<'a>>, DynThief<'a, Uart<'a>>>,
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

    fn as_pins_mut(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
        let pins = unsafe { self.inner.build_left().into() };
        Pins {
            rx: pins.rx,
            tx: pins.tx,
        }
    }
}

impl AsIoReadWriteDevice for DynBoard<'_> {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_io_read_write(&mut self) -> Self::Target<'_> {
        unsafe { self.inner.build_right().build().into() }
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
