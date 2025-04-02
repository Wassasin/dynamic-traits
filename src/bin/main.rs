use std::{convert::Infallible, marker::PhantomData};

use dynamic_traits::{
    consumer::{self, AsPinsMut, AsUartMut, Dependency, Pins},
    hal::{
        Peri, Peripherals,
        gpio::{Input, Output},
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

        impl<'b, 'a: 'b> From<&'a mut $board<'b>> for DynBoard<'a> {
            fn from(value: &'a mut $board<'b>) -> Self {
                Self {
                    pins: DynPins {
                        rx: &mut value.pins.rx,
                        tx: &mut value.pins.tx,
                    },
                }
            }
        }

        // impl AsIoReadWriteDevice for $board<'_> {
        //     type Target<'a>
        //         = Uart<'a>
        //     where
        //         Self: 'a;

        //     fn as_io_read_write(&mut self) -> Self::Target<'_> {
        //         Uart::new(
        //             self.uart.reborrow(),
        //             self.pins.rx.reborrow(),
        //             self.pins.tx.reborrow(),
        //         )
        //     }
        // }

        // impl AsUartMut for $board<'_> {}

        impl Dependency for $board<'_> {}
    };
}

impl<'a> AsIoReadWriteDevice for BoardA<'a> {
    type Target = Uart<'a>;

    fn as_io_read_write(self) -> Self::Target {
        Uart::new(self.uart, self.pins.rx, self.pins.tx)
    }
}

impl AsUartMut for BoardA<'_> {
    type Target<'b>
        = BoardA<'b>
    where
        Self: 'b;

    fn as_uart_mut(&mut self) -> Self::Target<'_> {
        BoardA {
            pins: Pins {
                rx: self.pins.rx.reborrow(),
                tx: self.pins.tx.reborrow(),
            },
            uart: self.uart.reborrow(),
        }
    }
}

// impl AsUartMut for BoardA<'_> {
//     type Target<'a>
//         = &'a mut BoardA<'a>
//     where
//         Self: 'a;

//     fn as_uart_mut(&mut self) -> Self::Target<'_> {
//         &mut self
//     }
// }

// trait ComboPin: InputPin<Error = Infallible> + OutputPin<Error = Infallible> {}
trait ComboPin<'a>: AsInput<Target = Input<'a>> + AsOutput<Target = Output<'a>>
where
    Self: 'a,
{
}

impl<'a, T: dynamic_traits::hal::gpio::Instance> ComboPin<'a> for Peri<'a, T> {}
impl<'a, T: dynamic_traits::hal::gpio::Instance> ComboPin<'a> for &'a mut Peri<'a, T> {}

struct DynPins<'a> {
    rx: &'a mut dyn ComboPin<'a>,
    tx: &'a mut dyn ComboPin<'a>,
}

trait ConcreteUart<'a>: AsUartMut<Target<'a> = DynBoard<'a>>
where
    Self: 'a,
{
}

struct DynBoard<'a> {
    pins: DynPins<'a>,
    // uart: &'a mut dyn ConcreteUart<'a>,
}

struct DynPin<'a> {
    inner: &'a mut dyn ComboPin<'a>,
}

impl<'a> AsInput for DynPin<'a> {
    type Target = Input<'a>;

    fn as_input(self) -> Self::Target {
        self.inner.as_input()
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

    fn as_pins_mut(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>> {
        todo!()
    }
}
impl Dependency for DynBoard<'_> {}

impl_board!(BoardA, PIN_A, PIN_B, UART0);
impl_board!(BoardB, PIN_B, PIN_C, UART1);
impl_board!(BoardC, PIN_C, PIN_D, UART2);

#[derive(Debug)]
enum Boards {
    A,
    B,
    C,
}

enum AnyBoard<'a> {
    A(BoardA<'a>),
    B(BoardB<'a>),
    C(BoardC<'a>),
}

impl<'a> AnyBoard<'a> {
    pub fn select(p: &'a mut Peripherals, board: Boards) -> Self {
        match board {
            Boards::A => AnyBoard::A(BoardA {
                pins: Pins {
                    rx: p.PIN_A.reborrow(),
                    tx: p.PIN_B.reborrow(),
                },
                uart: p.UART0.reborrow(),
            }),
            Boards::B => AnyBoard::B(BoardB {
                pins: Pins {
                    rx: p.PIN_B.reborrow(),
                    tx: p.PIN_C.reborrow(),
                },
                uart: p.UART1.reborrow(),
            }),
            Boards::C => AnyBoard::C(BoardC {
                pins: Pins {
                    rx: p.PIN_C.reborrow(),
                    tx: p.PIN_D.reborrow(),
                },
                uart: p.UART2.reborrow(),
            }),
        }
    }
}

impl<'b, 'a: 'b> From<&'a mut AnyBoard<'b>> for DynBoard<'a> {
    fn from(value: &'a mut AnyBoard<'b>) -> Self {
        match value {
            AnyBoard::A(board_a) => board_a.into(),
            AnyBoard::B(board_b) => board_b.into(),
            AnyBoard::C(board_c) => board_c.into(),
        }
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
            log::info!("{:?}", board);

            let board = AnyBoard::select(&mut p, board);
            let board: DynBoard<'_> = (&mut board).into();

            async fn run(board: impl Dependency) {
                embassy_futures::select::select(consumer::run(board), Timer::after_millis(100))
                    .await;
            }

            run(board).await
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
