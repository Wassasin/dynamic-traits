//! A mockup library mimmicking a HAL, i.e. embassy-imrtx

pub mod foreign;
pub mod gpio;
pub mod i2c;
pub mod uart;

embassy_hal_internal::peripherals!(PIN_A, PIN_B, I2C0, UART0);
