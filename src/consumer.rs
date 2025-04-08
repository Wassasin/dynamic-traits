//! A driver library that does not know what hardware it is running on.
use embassy_time::Timer;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{Read, Write};

use crate::traits::{AsInput, AsIoReadWriteDevice, AsOutput};

/// Specific arrangement of pins as expected by this crate.
pub struct Pins<RX, TX> {
    pub rx: RX,
    pub tx: TX,
}

pub trait AsPinsMut {
    type RX<'a>: AsOutput + AsInput + 'a
    where
        Self: 'a;
    type TX<'a>: AsOutput + AsInput + 'a
    where
        Self: 'a;

    fn as_pins_mut(&mut self) -> Pins<Self::RX<'_>, Self::TX<'_>>;
}

/// BSP crates should implement this trait if they want to use this library.
pub trait Dependency: AsPinsMut + AsIoReadWriteDevice {}

enum FeatureState {
    PowerOn,
    FullBus,
    BitBanging,
    LowPower,
}

fn parse(_buf: &[u8]) -> Result<(), ()> {
    Ok(())
}

async fn precise_wait_us(time_us: u64) {
    Timer::after_micros(time_us).await
}

async fn wait_for_something() {
    embassy_futures::yield_now().await
}

/// Core logic implemented by this crate.
pub async fn run(mut dependencies: impl Dependency) -> ! {
    const MAGIC_SEQUENCE_TO_STARTUP: [u8; 4] = [0x01, 0x02, 0x03, 0xff];

    let mut state = FeatureState::PowerOn;

    loop {
        match state {
            FeatureState::PowerOn => {
                let mut pins = dependencies.as_pins_mut();

                // Weird chip on the other side needs the bus "de-gaussed"
                let mut rx_pin = pins.rx.as_output();
                let mut tx_pin = pins.tx.as_output();

                rx_pin.set_high().unwrap();
                tx_pin.set_high().unwrap();

                state = FeatureState::FullBus;
            }
            FeatureState::FullBus => {
                let mut uart_bus = dependencies.as_io_read_write();

                uart_bus.write(&MAGIC_SEQUENCE_TO_STARTUP).await.unwrap();

                let mut some_buffer_that_exists = [0u8; 4];
                // This is contrived but shows the states we might expect to enter + exit frequently
                uart_bus.read(&mut some_buffer_that_exists).await.unwrap();

                drop(uart_bus);

                if parse(&some_buffer_that_exists).is_err() {
                    state = FeatureState::BitBanging;
                } else {
                    state = FeatureState::LowPower;
                }
            }
            FeatureState::BitBanging => {
                let mut pins = dependencies.as_pins_mut();

                let mut rx_pin = pins.rx.as_input();
                let mut tx_pin = pins.tx.as_output();

                // probably would have some termination condition
                while rx_pin.is_low().unwrap() {
                    tx_pin.set_high().unwrap();
                    precise_wait_us(150).await;
                    tx_pin.set_low().unwrap();
                    precise_wait_us(273).await;
                }

                state = FeatureState::FullBus;
            }
            FeatureState::LowPower => {
                // Pins are always in low power unless in some mode.
                // tx_pin.enter_lowpower();
                // rx_pin.enter_lowpower();

                wait_for_something().await;

                // order might actually matter in some scenarios
                // tx_pin.exit_lowpower();
                // rx_pin.exit_lowpower();

                state = FeatureState::FullBus;
            }
        }
    }
}
