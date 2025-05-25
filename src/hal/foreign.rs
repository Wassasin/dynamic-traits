use embassy_hal_internal::Peri;

use crate::hal::gpio::{Input, Instance, Output};
use crate::traits::{AsInput, AsOutput};

impl<'b, T: Instance> AsOutput for Peri<'b, T> {
    type Target = Output<'b>;

    fn as_output(self) -> Self::Target {
        Output::new(self)
    }
}

impl<'b, T: Instance> AsInput for Peri<'b, T> {
    type Target = Input<'b>;

    fn as_input(self) -> Self::Target {
        Input::new(self)
    }
}
