use std::ops::Add;

use ark_ff::Field;

use crate::Expr;


#[derive(Clone, Copy)]
pub struct Wire<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: usize,
    life: &'a usize,
}
#[derive(Clone)]
pub struct V<'a, F: Field> {
    pub(crate) val: F,
    pub(crate) exp: Expr<F>,
    life: &'a (),
}

pub struct ConstraintSystem<F: Field> {
    wires: Vec<F>,
    life: usize,
}

impl<F: Field> ConstraintSystem< F> {
    pub fn alloc(&mut self, val: F) -> Wire<'_, F> {
        
        let wire = Wire {
            val,
            exp: self.wires.len(),
            life: &self.life, // self.lifeはもともと&'a ()なので安全に渡せる
        };

        self.wires.push(val);

        wire
    }
}

impl<'a, F: Field> Add for Wire<'a, F> {
    type Output = V<'a, F>;

    fn add(self, rhs: Self) -> Self::Output {
        todo!()
    }
}
