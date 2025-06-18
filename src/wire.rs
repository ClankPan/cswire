use ark_ff::Field;

use crate::expr::Expr;


#[derive(Clone, Copy, Default)]
pub struct Wire<F: Field> {
    pub(crate) val: F,
    pub(crate) exp: usize,
}


#[derive(Clone)]
pub struct V<F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
}

#[derive(Clone)]
pub struct VV<F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
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

impl<F: Field> Wire<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<F: Field> V<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<F: Field> VV<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
