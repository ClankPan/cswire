use std::marker::PhantomData;

use ark_ff::Field;

use crate::expr::Expr;

#[derive(Clone, Copy)]
pub struct Wire<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: usize,
    // pub(crate) life: &'a (),
    pub(crate) _life: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct V<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
    // pub(crate) life: &'a (),
    pub(crate) _life: PhantomData<&'a ()>,
}

#[derive(Clone)]
pub struct VV<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
    // pub(crate) life: &'a (),
    pub(crate) _life: PhantomData<&'a ()>,
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

impl<F: Field> Wire<'_, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<F: Field> V<'_, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<F: Field> VV<'_, F> {
    pub fn raw(&self) -> F {
        self.val
    }
}



pub trait LinearVar<'a, F: Field> {

}

impl<'a, F: Field> LinearVar<'a, F>  for Wire<'a, F>{

}

impl<'a, F: Field> LinearVar<'a, F>  for V<'a, F>{

}
