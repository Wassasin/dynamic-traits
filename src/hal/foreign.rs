use embassy_hal_internal::Peripheral;

use crate::traits::{AsOutput, OwnedRef};

use super::gpio::{self, Output};

impl<T: Peripheral<P = impl gpio::Instance>> AsOutput for T {
    type Target<'a>
        = Output<'a>
    where
        T: 'a;

    fn as_output<'a>(&'a mut self) -> crate::traits::OwnedRef<'a, Self::Target<'a>> {
        OwnedRef::new(Output::new(self))
    }
}
