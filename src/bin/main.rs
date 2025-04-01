use std::convert::Infallible;

use dynamic_traits::{
    consumer::{self, AsPins, Dependency, Pins},
    hal::{
        Peripherals,
        gpio::{Input, Output},
        uart::Uart,
    },
    traits::{AsFlex, AsInput, AsIoReadWriteDevice, AsOutput},
};
use embassy_executor::Executor;
use embassy_time::Timer;
use embedded_hal::digital::{InputPin, OutputPin};
use static_cell::StaticCell;

macro_rules! impl_board {
    ($board:ident, $pin_rx:ident, $pin_tx:ident, $uart:ident) => {
        struct $board<'a> {
            pins: consumer::Pins<
                &'a mut dynamic_traits::hal::peripherals::$pin_rx,
                &'a mut dynamic_traits::hal::peripherals::$pin_tx,
            >,
            uart: &'a mut dynamic_traits::hal::peripherals::$uart,
        }

        impl<'c> AsPins for $board<'c> {
            type RX<'b>
                = &'b mut dynamic_traits::hal::peripherals::$pin_rx
            where
                Self: 'b;
            type TX<'b>
                = &'b mut dynamic_traits::hal::peripherals::$pin_tx
            where
                Self: 'b;

            fn as_pins<'a, 'b: 'a>(
                &'a mut self,
            ) -> &'a mut consumer::Pins<Self::RX<'b>, Self::TX<'b>> {
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

// struct DynBoard<'a> {
//     // pins: Pins<&'a dyn Test<'a>, &'a dyn Test<'a>>,
//     rc: dyn Dependency,
// }

impl AsIoReadWriteDevice for AnyBoard<'_> {
    type Target<'a>
        = Uart<'a>
    where
        Self: 'a;

    fn as_io_read_write(&mut self) -> Self::Target<'_> {
        match self {
            AnyBoard::A(board_a) => board_a.as_io_read_write(),
            AnyBoard::B(board_b) => board_b.as_io_read_write(),
            AnyBoard::C(board_c) => board_c.as_io_read_write(),
        }
    }
}

trait AsPinConcr<'a>: AsInput<'a, Target = Input<'a>> + AsOutput<'a, Target = Input<'a>>
where
    Self: 'a,
{
}

type Test = dyn for<'a> AsPinConcr<'a>;

impl AsPins for AnyBoard<'_> {
    type RX<'a>
        = Test
    where
        Self: 'a;
    type TX<'a>
        = Test
    where
        Self: 'a;

    fn as_pins(&mut self) -> &mut Pins<Self::RX<'_>, Self::TX<'_>> {
        todo!()
    }
}

impl<'a> AnyBoard<'a> {
    pub fn select(p: &'a mut Peripherals, board: Boards) -> Self {
        match board {
            Boards::A => AnyBoard::A(BoardA {
                pins: Pins {
                    rx: &mut p.PIN_A,
                    tx: &mut p.PIN_B,
                },
                uart: &mut p.UART0,
            }),
            Boards::B => AnyBoard::B(BoardB {
                pins: Pins {
                    rx: &mut p.PIN_B,
                    tx: &mut p.PIN_C,
                },
                uart: &mut p.UART1,
            }),
            Boards::C => AnyBoard::C(BoardC {
                pins: Pins {
                    rx: &mut p.PIN_C,
                    tx: &mut p.PIN_D,
                },
                uart: &mut p.UART2,
            }),
        }
    }
}

#[embassy_executor::task]
async fn run() {
    let mut p = unsafe { Peripherals::steal() };
    {
        let mut output = Output::new(&mut p.PIN_A);
        output.set_high().unwrap();

        let _input = Input::new(&mut p.PIN_A);
    }

    loop {
        for board in [Boards::A, Boards::B, Boards::C] {
            log::info!("{:?}", board);

            let board = AnyBoard::select(&mut p, board);

            async fn run(board: impl Dependency) {
                embassy_futures::select::select(consumer::run(board), Timer::after_millis(100))
                    .await;
            }

            match board {
                AnyBoard::A(board) => run(board).await,
                AnyBoard::B(board) => run(board).await,
                AnyBoard::C(board) => run(board).await,
            }
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
