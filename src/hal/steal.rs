use embassy_hal_internal::{Peri, PeripheralType};

pub trait Stealable: PeripheralType {
    unsafe fn steal<'a>() -> Peri<'a, Self>;
}
