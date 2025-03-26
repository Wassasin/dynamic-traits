use embassy_hal_internal::Peripheral;

use crate::traits::{AsInput, AsOutput};

use super::gpio::{self, Input, Output};

impl<T: Peripheral<P = impl gpio::Instance>> AsOutput for T {
    type Target<'a>
        = Output<'a>
    where
        T: 'a;

    fn as_output<'a>(&'a mut self) -> Self::Target<'a> {
        Output::new(self)
    }
}

impl<T: Peripheral<P = impl gpio::Instance>> AsInput for T {
    type Target<'a>
        = Input<'a>
    where
        T: 'a;

    fn as_input<'a>(&'a mut self) -> Self::Target<'a> {
        Input::new(self)
    }
}
