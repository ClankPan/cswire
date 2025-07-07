use std::{collections::HashMap, marker::PhantomData};

use ark_ff::Field;

use super::{Lc, Lin, Qua};

// To Qua
impl<'a, F: Field> From<Lin<'a, F>> for Qua<'a, F> {
    fn from(lin: Lin<'a, F>) -> Self {
        Qua {
            value: lin.value,
            qc: (F::ZERO, Lc(Some(HashMap::new())), Lc(Some(HashMap::new()))),
            lc: lin.lc,
            _life: PhantomData,
        }
    }
}

impl<'a, F: Field> From<&Lin<'a, F>> for Qua<'a, F> {
    fn from(lin: &Lin<'a, F>) -> Self {
        Qua {
            value: lin.value,
            qc: (F::ZERO, Lc(Some(HashMap::new())), Lc(Some(HashMap::new()))),
            lc: lin.lc.clone(),
            _life: PhantomData,
        }
    }
}

impl<'a, F: Field> From<&Qua<'a, F>> for Qua<'a, F> {
    fn from(qua: &Qua<'a, F>) -> Self {
        qua.clone()
    }
}



// To Lin
impl<'a, F: Field> From<&Lin<'a, F>> for Lin<'a, F> {
    fn from(lin: &Lin<'a, F>) -> Self {
        lin.clone()
    }
}

