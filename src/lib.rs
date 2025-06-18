pub mod expr;
pub mod switchboard;
pub mod wire;
pub mod ops;
pub mod ark_poseidon;
pub mod utils;
pub mod lifetime;

use std::{cell::RefCell, rc::Rc};

use ark_ff::Field;
// use expr::{Expr, R1CS, compile};
// use wire::{VV, Wire};
pub use expr::*;
pub use wire::*;

#[derive(Clone)]
pub(crate) struct ConstraintSystem<F: Field> {
    wires: Vec<Wire<F>>,
    exprs: Vec<Expr<F>>,
    switch: bool,
    one: Wire<F>,
}

impl<F: Field> ConstraintSystem<F> {
    pub fn new() -> Self {
        let one = Wire {
            val: F::ONE,
            exp: 0,
        };
        Self {
            wires: vec![one],
            exprs: vec![],
            switch: true,
            one,
        }
    }

    pub fn set_one(&mut self, one: Wire<F>) {
        self.one = one
    }
    pub fn one(&self) -> Wire<F> {
        self.one
    }
    pub fn io(&mut self, _wire: Wire<F>) -> Wire<F> {
        todo!()
    }
    pub fn wire(&mut self, variable: VV<F>) -> Wire<F> {
        self.exprs.push(Expr::Sub(
            Box::new(variable.exp),
            Box::new(Expr::Con(variable.val)),
        ));
        self.alloc(variable.val)
    }

    pub fn link<T>(&mut self, vv: VV<F>, constant: T)
    where
        F: From<T>,
    {
        self.exprs.push(Expr::Sub(
            Box::new(vv.exp),
            Box::new(Expr::Con(constant.into())),
        ));
    }

    pub fn alloc<T>(&mut self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        let val = match self.switch {
            true => val.into(),
            false => F::ZERO,
        };

        let wire = Wire {
            val,
            exp: self.wires.len(),
        };
        self.wires.push(wire);
        wire
    }

    pub fn witnesses(&self) -> Vec<F> {
        todo!()
    }

    pub fn set_switch(&mut self, switch: bool) {
        self.switch = switch;
    }

    pub fn switch(&self) -> bool {
        self.switch
    }

    pub fn compile(&self) -> R1CS<F> {
        let constraints = self.exprs.iter().map(|exp| compile(exp)).collect();
        R1CS(constraints)
    }
}

#[derive(Clone)]
pub struct ConstraintSystemRef<F: Field>(Rc<RefCell<ConstraintSystem<F>>>);
impl<F: Field> ConstraintSystemRef<F> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ConstraintSystem::new())))
    }

    pub fn set_one(&self, one: Wire<F>) {
        self.0.borrow_mut().set_one(one);
    }

    pub fn one(&self) -> Wire<F> {
        self.0.borrow().one()
    }
    pub fn io(&self, wire: Wire<F>) -> Wire<F> {
        self.0.borrow_mut().io(wire)
    }
    pub fn wire(&self, var: VV<F>) -> Wire<F> {
        self.0.borrow_mut().wire(var)
    }

    pub fn link<T>(&self, vv: VV<F>, constant: T)
    where
        F: From<T>,
    {
        self.0.borrow_mut().link(vv, constant)
    }

    pub fn alloc<T>(&self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        self.0.borrow_mut().alloc(val)
    }

    pub fn witnesses(&self) -> Vec<F> {
        self.0.borrow().witnesses()
    }

    pub fn compile(&self) -> R1CS<F> {
        self.0.borrow().compile()
    }

    pub fn set_switch(&self, switch: bool) {
        self.0.borrow_mut().set_switch(switch);
    }

    pub fn switch(&self) -> bool {
        self.0.borrow().switch()
    }
}
