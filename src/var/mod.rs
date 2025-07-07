pub mod assign_ops;
pub mod binary_ops;
pub mod from;

#[cfg(test)]
mod tests;
// pub use binary_ops::*;

use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Add, Mul, Sub},
};

use ark_ff::Field;

pub type V<'a, F> = Lin<'a, F>;
pub type VV<'a, F> = Qua<'a, F>;

#[derive(Clone, Debug, Default)]
pub struct Lc<F: Field>(pub(crate) Option<HashMap<usize, F>>);

impl<F: Field> Lc<F> {
    pub fn new(idx: usize, do_compile: bool) -> Self {
        if do_compile {
            Self(Some(HashMap::from([(idx, F::ONE)])))
        } else {
            Self(None)
        }
    }
}

impl<F: Field> Add<Lc<F>> for Lc<F> {
    type Output = Lc<F>;

    fn add(self, rhs: Lc<F>) -> Self::Output {
        let lc = match (self.0, rhs.0) {
            (Some(mut lc), Some(v)) => {
                for (idx, v_rhs) in v {
                    lc.entry(idx)
                        .and_modify(|v_lhs| *v_lhs += v_rhs) // 既存なら加算
                        .or_insert(v_rhs); // なければ挿入
                }
                Some(lc)
            }
            _ => None,
            // _ => panic!("debug: add"),
        };
        Self(lc)
    }
}

impl<F: Field> Sub<Lc<F>> for Lc<F> {
    type Output = Lc<F>;

    fn sub(self, rhs: Lc<F>) -> Self::Output {
        let lc = match (self.0, rhs.0) {
            (Some(mut lc), Some(v)) => {
                for (idx, v_rhs) in v {
                    lc.entry(idx)
                        .and_modify(|v_lhs| *v_lhs -= v_rhs) // 既存なら加算
                        .or_insert(v_rhs); // なければ挿入
                }
                Some(lc)
            }
            _ => None,
            // _ => panic!("debug: sub"),
        };
        Self(lc)
    }
}

impl<F: Field> Mul<F> for Lc<F> {
    type Output = Self;

    fn mul(mut self, coeff: F) -> Self::Output {
        if let Some(ref mut map) = self.0 {
            if coeff.is_zero() {
                // 係数が 0 なら空にする
                self.0 = Some(HashMap::new());
            } else {
                map.values_mut().for_each(|v| *v *= coeff);
            }
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct Lin<'a, F: Field> {
    pub(crate) value: F,
    pub(crate) lc: Lc<F>,
    pub(crate) _life: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct Qua<'a, F: Field> {
    pub(crate) value: F,
    pub(crate) qc: (F, Lc<F>, Lc<F>),
    pub(crate) lc: Lc<F>,
    pub(crate) _life: PhantomData<&'a ()>,
}



impl<F: Field> Lin<'_, F> {
    pub fn raw(&self) -> F {
        self.value
    }
    pub(crate) fn len(&self) -> usize {
        self.lc.0.as_ref().map_or(0, |m| m.len())
    }
}

impl<F: Field> Qua<'_, F> {
    pub fn raw(&self) -> F {
        self.value
    }
}
