use crate::traits::AsOutput;

use super::gpio::{self, Output};

impl<'a, T: gpio::Instance> AsOutput<'a> for T {
    type Target = Output<'a>;

    fn as_output(&'a mut self) -> crate::traits::OwnedRef<'a, Self::Target> {
        todo!()
    }
}
