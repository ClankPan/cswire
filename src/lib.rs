pub mod ark_poseidon;
pub mod expr;
pub mod lifetime;
pub mod ops;
pub mod switchboard;
pub mod utils;
pub mod wire;

use std::{cell::RefCell, marker::PhantomData, rc::Rc};

use ark_ff::Field;
// use expr::{Expr, R1CS, compile};
// use wire::{VV, Wire};
pub use expr::*;
pub use wire::*;

#[derive(Clone)]
pub(crate) struct ConstraintSystem<'a, F: Field> {
    wires: Vec<F>,
    exprs: Vec<Expr<F>>,
    switch: bool,
    one_idx: usize,             // oneを指し示すindex
    _life: PhantomData<&'a ()>, // 型データとライフタイムだけを保持することで、クロージャーにcsを渡した時に、コンパイラがcloneを要求しなくなる。
}

impl<'a, F: Field> ConstraintSystem<'a, F> {
    pub fn new(_: &'a ()) -> Self {
        let one_idx = 0;
        Self {
            wires: vec![F::ONE],
            exprs: vec![],
            switch: true,
            one_idx,
            _life: PhantomData,
        }
    }

    pub fn set_one(&mut self, one: Wire<F>) {
        self.one_idx = one.exp;
    }

    pub fn one(&self) -> Wire<'_, F> {
        Wire {
            exp: self.one_idx,
            val: self.wires[self.one_idx],
            _life: PhantomData,
            // life: self.life,
        }
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
            // life: self.life,
            _life: PhantomData,
        };
        self.wires.push(val);
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
pub struct ConstraintSystemRef<'a, F: Field>(Rc<RefCell<ConstraintSystem<'a, F>>>);

impl<'a, F: Field> ConstraintSystemRef<'a, F> {
    pub fn new(life: &'a ()) -> Self {
        Self(Rc::new(RefCell::new(ConstraintSystem::new(life))))
    }

    pub fn set_one(&self, one: Wire<F>) {
        self.0.borrow_mut().set_one(one);
    }

    pub fn one(&self) -> Wire<'a, F> {
        let (exp, val) = {
            let cs = self.0.borrow();
            let wire = cs.one();
            (wire.exp, wire.val)
        };
        Wire {
            exp,
            val,
            _life: PhantomData,
        }
    }

    pub fn io(&self, _wire: Wire<F>) {
        todo!()
    }
    pub fn wire(&self, var: VV<F>) -> Wire<'a, F> {
        // ───── ① RefMut のスコープをこのブロック内に閉じ込める
        let (exp, val) = {
            let mut cs = self.0.borrow_mut(); // Ref 開始
            let wire = cs.wire(var);
            (wire.exp, wire.val)
        }; // Ref 終了

        Wire {
            exp,
            val,
            _life: PhantomData,
        } // 借用はもう存在しないので返せる
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
        let (exp, val) = {
            let mut cs = self.0.borrow_mut(); // Ref 開始
            let wire = cs.alloc(val);
            (wire.exp, wire.val)
        }; // Ref 終了

        Wire {
            exp,
            val,
            _life: PhantomData,
        } // 借用はもう存在しないので返せる
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
