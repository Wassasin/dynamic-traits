//! A mockup library mimmicking a HAL, i.e. embassy-imrtx

pub mod foreign;
pub mod gpio;
pub mod i2c;
pub mod uart;

pub use embassy_hal_internal::Peri;

embassy_hal_internal::peripherals!(PIN_A, PIN_B, PIN_C, PIN_D, I2C0, UART0, UART1, UART2);
