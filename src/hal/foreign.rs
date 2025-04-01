use embassy_hal_internal::Peripheral;

use crate::traits::{AsInput, AsOutput};

use super::gpio::{self, Input, Output};

impl<'a, T: Peripheral<P = impl gpio::Instance>> AsOutput<'a> for T {
    type Target = Output<'a>;
    fn as_output(&'a mut self) -> Self::Target {
        Output::new(self)
    }
}

impl<'a, T: Peripheral<P = impl gpio::Instance>> AsInput<'a> for T {
    type Target = Input<'a>;
    fn as_input(&'a mut self) -> Self::Target {
        Input::new(self)
    }
}
