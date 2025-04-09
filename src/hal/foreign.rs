use embassy_hal_internal::Peri;

use crate::dynamic::Owned;
use crate::hal::gpio::{Input, Instance, Output};
use crate::traits::{AsInput, AsOutput};

impl<'b, T: Instance> AsOutput<'b> for Peri<'b, T> {
    type Target = Output<'b>;

    fn as_output(value: Owned<'b, Self>) -> Self::Target {
        Output::new(Into::into(value))
    }
}

impl<'b, T: Instance> AsInput<'b> for Peri<'b, T> {
    type Target = Input<'b>;

    fn as_input(value: Owned<'b, Self>) -> Self::Target {
        Input::new(Into::into(value))
    }
}
