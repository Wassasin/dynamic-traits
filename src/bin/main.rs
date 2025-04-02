
use dynamic_traits::{
    consumer::{self, AsPins, Dependency, Pins},
    hal::{
        Peri, Peripherals,
        gpio::{Input, Output},
        uart::Uart,
    },
    traits::AsIoReadWriteDevice,
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

        impl<'a> AsPins for $board<'a> {
            type RX = Peri<'a, dynamic_traits::hal::peripherals::$pin_rx>;
            type TX = Peri<'a, dynamic_traits::hal::peripherals::$pin_tx>;

            fn as_pins(&mut self) -> &mut consumer::Pins<Self::RX, Self::TX> {
                &mut self.pins
            }
        }

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
