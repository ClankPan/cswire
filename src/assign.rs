use std::ops::AddAssign;

use ark_ff::Field;

use crate::{Lin, Qua};

impl<'a, F: Field> AddAssign<Lin<'a, F>> for Lin<'a, F> {
    fn add_assign(&mut self, rhs: Self) {
        *self = &*self + rhs;
    }
}

impl<'a, F: Field> AddAssign<&Lin<'a, F>> for Lin<'a, F> {
    fn add_assign(&mut self, rhs: &Self) {
        *self = &*self + rhs;
    }
}

impl<'a, F: Field> AddAssign<Lin<'a, F>> for Qua<'a, F> {
    fn add_assign(&mut self, rhs: Lin<'a, F>) {
        *self = &*self + rhs;
    }
}

impl<'a, F: Field> AddAssign<&Lin<'a, F>> for Qua<'a, F> {
    fn add_assign(&mut self, rhs: &Lin<'a, F>) {
        *self = &*self + rhs;
    }
}
