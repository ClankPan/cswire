pub mod ark_poseidon;
pub mod expr;
pub mod lifetime;
pub mod ops;
pub mod switchboard;
pub mod utils;
pub mod wire;
pub mod cswire;
pub mod linear;
pub mod quadratic;

use std::{cell::RefCell, marker::PhantomData, ops::Sub, rc::Rc};

use ark_ff::Field;
// use expr::{Expr, R1CS, compile};
// use wire::{VV, Wire};
pub use expr::*;
use itertools::Itertools;
pub use wire::*;

#[derive(Clone)]
pub(crate) struct ConstraintSystem<'a, F: Field> {
    wires: Vec<F>,
    exprs: Vec<Expr<F>>,
    one_idx: usize,             // oneを指し示すindex
    _life: PhantomData<&'a ()>, // 型データとライフタイムだけを保持することで、クロージャーにcsを渡した時に、コンパイラがcloneを要求しなくなる。
}

impl<'a, F: Field> ConstraintSystem<'a, F> {
    pub fn new() -> Self {
        let one_idx = 0;
        Self {
            wires: vec![F::ONE],
            exprs: vec![],
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

    pub fn wire(&mut self, var: VV<F>) -> Wire<F> {
        let exp = var.exp.sub(Expr(vec![AST::Con(var.val)])); // exprはゼロになる式になるように。
        self.exprs.push(exp);
        self.alloc(var.val)
    }

    pub fn link<T>(&mut self, var: VV<F>, constant: T)
    where
        F: From<T>,
    {
        let exp = var.exp.sub(Expr(vec![AST::Con(constant.into())])); // exprはゼロになる式になるように。
        self.exprs.push(exp);
    }

    pub fn alloc<T>(&mut self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        let val = self.one().val * F::from(val); // oneが0の時はswitchがoffなので、witnessも全てゼロにする。
        let wire = Wire {
            val,
            exp: self.wires.len(),
            _life: PhantomData,
        };
        self.wires.push(val);
        wire
    }

    pub fn finalize(mut self, inputs: &[Wire<'a, F>]) -> (Vec<F>, Vec<Expr<F>>) {
        // ユーザが指定したinputにするWireを先頭の方に持ってくる。
        // 今ココで返すWitnessの順番と、Exprが指定するWitnessの場所が合うようにする。
        let inputs: Vec<usize> = inputs
            .iter()
            .map(|w| w.exp) // 既存インデックス
            .chain(std::iter::once(0)) // ONE (=0) を必ず追加
            .unique() // 重複を取り除く
            .sorted_unstable() // 昇順ソート
            .collect();
        let mut permu: Vec<usize> = (0..self.wires.len()).collect();
        for (i, j) in inputs.into_iter().enumerate() {
            permu.swap(i, j);
            self.wires.swap(i, j);
        }
        let exprs = self
            .exprs
            .into_iter()
            .map(|expr| {
                let nodes: Vec<AST<F>> = expr
                    .0
                    .into_iter()
                    .map(|node| match node {
                        AST::Idx(n) => AST::Idx(permu[n]),
                        _ => node,
                    })
                    .collect();
                Expr(nodes)
            })
            .collect();
        (self.wires, exprs)
    }
}

#[derive(Clone)]
pub struct ConstraintSystemRef<'a, F: Field>(Rc<RefCell<ConstraintSystem<'a, F>>>);

impl<'a, F: Field> ConstraintSystemRef<'a, F> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ConstraintSystem::new())))
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

    pub fn finalize(self, inputs: &[Wire<'a, F>]) -> (Vec<F>, Vec<Expr<F>>) {
        self.0.borrow_mut().clone().finalize(inputs)
    }
}


