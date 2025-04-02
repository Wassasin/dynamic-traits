use embassy_hal_internal::Peri;

use crate::hal::gpio::{Input, Instance, Output};
use crate::traits::{AsInput, AsOutput};

impl<'a, T: Instance> AsOutput for Peri<'a, T> {
    type Target = Output<'a>;

    fn as_output(self) -> Self::Target {
        Output::new(self)
    }
}

impl<'a, T: Instance> AsOutput for &'a mut Peri<'a, T> {
    type Target = Output<'a>;

    fn as_output(self) -> Self::Target {
        Output::new(self.reborrow())
    }
}

impl<'a, T: Instance> AsInput for Peri<'a, T> {
    type Target = Input<'a>;

    fn as_input(self) -> Self::Target {
        Input::new(self)
    }
}

impl<'a, T: Instance> AsInput for &'a mut Peri<'a, T> {
    type Target = Input<'a>;

    fn as_input(self) -> Self::Target {
        Input::new(self.reborrow())
    }
}
