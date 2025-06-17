use ark_ff::Field;
use crate::{variables::{VV}, V};
use crate::expr::{Exp, Idx};
use crate::wires::Wire;

impl<F: Field> From<Wire<F>> for V<F> {
    fn from(Wire { exp, val }: Wire<F>) -> Self {
        let exp = exp.map(Exp::Idx);
        Self { exp, val }
    }
}

impl<F: Field> From<&Wire<F>> for V<F> {
    fn from(Wire { exp, val }: &Wire<F>) -> Self {
        let exp = exp.map(Exp::Idx);
        Self { exp, val: *val }
    }
}

impl<F: Field> From<V<F>> for VV<F> {
    fn from(v: V<F>) -> Self {
        VV {
            val: v.val,
            exp: v.exp,
        }
    }
}
impl<F: Field> From<&V<F>> for VV<F> {
    fn from(v: &V<F>) -> Self {
        VV {
            val: v.val,
            exp: v.exp.clone(),
        }
    }
}

impl<F: Field> From<F> for V<F> {
    fn from(val: F) -> Self {
        V {
            val,
            exp: Some(Exp::Con(val)),
        }
    }
}

impl<F: Field> From<&F> for V<F> {
    fn from(val: &F) -> Self {
        V {
            val: *val,
            exp: Some(Exp::Con(*val)),
        }
    }
}

impl<F: Field> From<Idx> for Exp<F> {
    fn from(idx: Idx) -> Self {
        Exp::Idx(idx)
    }
}
