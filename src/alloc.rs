use ark_ff::Field;

use crate::{CSWire, Lin};

pub trait Alloc<'a, F: Field> {
    type Output;
    fn alloc(&self, cs: &'a CSWire<F>) -> Self::Output;
}

impl<'a, F: Field> Alloc<'a, F> for Vec<F> {
    type Output = Vec<Lin<'a, F>>;

    fn alloc(&self, cs: &'a CSWire<F>) -> Self::Output {
        self.iter().map(|v| cs.alloc(*v)).collect()
    }
}

impl<'a, F: Field> Alloc<'a, F> for F {
    type Output = Lin<'a, F>;

    fn alloc(&self, cs: &'a CSWire<F>) -> Self::Output {
        cs.alloc(*self)
    }
}

impl<'a, F: Field> Alloc<'a, F> for (F, F) {
    type Output = (Lin<'a, F>, Lin<'a, F>);

    fn alloc(&self, cs: &'a CSWire<F>) -> Self::Output {
        (cs.alloc(self.0), cs.alloc(self.1))
    }
}


impl<'a, F: Field> Alloc<'a, F> for [F;2] {
    type Output = [Lin<'a, F>;2];

    fn alloc(&self, cs: &'a CSWire<F>) -> Self::Output {
        [self[0].alloc(cs), self[1].alloc(cs)]
    }
}
