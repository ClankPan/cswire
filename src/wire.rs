use ark_ff::Field;

use crate::expr::Expr;

#[derive(Clone, Copy)]
pub struct Wire<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: usize,
    pub(crate) life: &'a (),
}

#[derive(Clone)]
pub struct V<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
    pub(crate) life: &'a (),
}

#[derive(Clone)]
pub struct VV<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
    pub(crate) life: &'a (),
}
#[derive(Clone)]
pub struct Coeff<F: Field>(pub F);

impl<F: Field> Coeff<F> {
    pub fn new(value: F) -> Self {
        Self(value)
    }
}
impl<F: Field> From<F> for Coeff<F> {
    fn from(value: F) -> Self {
        Self(value)
    }
}

impl<'a, F: Field> Wire<'a, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<'a, F: Field> V<'a, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<'a, F: Field> VV<'a, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
