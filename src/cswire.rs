use std::{cell::RefCell, marker::PhantomData};

use ark_ff::Field;
use itertools::Itertools;

use crate::{
    ASTs, Expr,
    binary_ops::FF,
    extract::{ToExpr, ToRaw},
    linear::Linear,
    quadratic::Quadratic,
};
pub type V<'a, F> = Lin<'a, F>;
pub type VV<'a, F> = Qua<'a, F>;

#[derive(Debug, Clone)]
pub struct Lin<'a, F: Field> {
    pub(crate) value: F,
    pub(crate) expr: Expr<F>,
    pub(crate) _life: PhantomData<&'a ()>,
}

#[derive(Debug, Clone)]
pub struct Qua<'a, F: Field> {
    pub(crate) value: F,
    pub(crate) expr: Expr<F>,
    pub(crate) _life: PhantomData<&'a ()>,
}

// 一貫性を保つためにCloneは実装しない。やるならRcとともに。
pub struct CSWire<F: Field> {
    one: RefCell<Lin<'static, F>>,
    witness: RefCell<Vec<F>>,
    exprs: RefCell<Vec<Expr<F>>>,
}

impl<F: Field> Default for CSWire<F> {
    fn default() -> Self {
        let one = Lin {
            value: F::ONE,
            expr: 0.into(),
            _life: PhantomData,
        };
        Self {
            one: RefCell::new(one),
            witness: RefCell::new(vec![F::ONE]),
            exprs: RefCell::new(vec![]),
        }
    }
}

impl<F: Field> CSWire<F> {
    pub fn alloc<T>(&self, value: T) -> Lin<'_, F>
    where
        F: From<T>,
    {
        let value = value.into();
        let mut witness = self.witness.borrow_mut();
        let index = witness.len();
        witness.push(value);
        Lin {
            value,
            expr: index.into(),
            _life: PhantomData,
        }
    }

    pub fn wire<Q: Quadratic<F> + ToRaw<F> + ToExpr<F>>(&self, var: Q) -> Lin<'_, F> {
        if let Expr::Idx(i) = var.expr() {
            // もし、exprが何の線型結合でなければ(Witnessそのもの)
            // そのままを返す
            return Lin {
                value: var.raw(),
                expr: Expr::Idx(i),
                _life: PhantomData,
            };
        }

        let new_var = self.alloc(var.raw());
        self.exprs
            .borrow_mut()
            .push(var.expr() - new_var.expr.clone()); // 制約式はイコール・ゼロになる形で保管。
        new_var
    }

    pub fn equal<Q, L>(&self, lhs: Q, rhs: L)
    where
        Q: Quadratic<F> + ToRaw<F> + ToExpr<F>,
        L: Linear<F> + ToRaw<F> + ToExpr<F>,
    {
        self.exprs.borrow_mut().push(lhs.expr() - rhs.expr()); // 制約式はイコール・ゼロになる形で保管。
    }

    pub fn one(&self) -> Lin<'_, F> {
        self.one.borrow().clone()
    }
    pub fn zero(&self) -> Lin<'_, F> {
        self.constant(0)
    }

    pub fn constant<T>(&self, value: T) -> Lin<'_, F>
    where
        F: From<T>,
    {
        self.one() * FF(value.into())
    }

    pub fn set_one(&self, new: Lin<'_, F>) -> Lin<'static, F> {
        // コピーして 'static にする
        let static_lin = Lin {
            value: new.value,   // accessor で値を取得
            expr: new.expr,     // `expr` が Copy なら直接
            _life: PhantomData, // 'static
        };
        self.one.replace(static_lin)
    }

    pub fn finish<'a>(
        &'a self,
        io: &[Lin<'a,F>],
    ) -> (Vec<F>, ASTs<F>) {
        let io: Vec<_> = io
            .iter()
            .cloned()
            .map(|i| match self.wire(i).expr() {
                Expr::Idx(idx) => idx,
                _ => unreachable!("wire() should return Expr::Idx as its expr"),
            })
            .chain(std::iter::once(0)) // ONE を指すwitnessはからならず、先頭にくるように追加
            .unique() // 重複を取り除く
            .sorted_unstable() // 昇順ソート
            .collect();
        let exprs = self.exprs.borrow().clone();
        let mut witness = self.witness.borrow().clone();
        let mut permu: Vec<usize> = (0..witness.len()).collect();
        for (i, j) in io.into_iter().enumerate() {
            permu.swap(i, j);
            witness.swap(i, j);
        }

        (witness, ASTs { permu, exprs })
    }

    // pub fn finalize(mut self, inputs: &[Wire<'a, F>]) -> (Vec<F>, Vec<Expr<F>>) {
    //     // ユーザが指定したinputにするWireを先頭の方に持ってくる。
    //     // 今ココで返すWitnessの順番と、Exprが指定するWitnessの場所が合うようにする。
    //     let inputs: Vec<usize> = inputs
    //         .iter()
    //         .map(|w| w.exp) // 既存インデックス
    //         .chain(std::iter::once(0)) // ONE (=0) を必ず追加
    //         .unique() // 重複を取り除く
    //         .sorted_unstable() // 昇順ソート
    //         .collect();
    //     let mut permu: Vec<usize> = (0..self.wires.len()).collect();
    //     for (i, j) in inputs.into_iter().enumerate() {
    //         permu.swap(i, j);
    //         self.wires.swap(i, j);
    //     }
    //     let exprs = self
    //         .exprs
    //         .into_iter()
    //         .map(|expr| {
    //             let nodes: Vec<AST<F>> = expr
    //                 .0
    //                 .into_iter()
    //                 .map(|node| match node {
    //                     AST::Idx(n) => AST::Idx(permu[n]),
    //                     _ => node,
    //                 })
    //                 .collect();
    //             Expr(nodes)
    //         })
    //         .collect();
    //     (self.wires, exprs)
    // }
}

#[cfg(test)]
mod tests {
    use crate::extract::ToRaw;

    use super::CSWire;
    use ark_bn254::Fr;

    #[test]
    pub fn test_alloc_ref() {
        let mut var;
        {
            let cs = CSWire::<Fr>::default();
            let a = cs.alloc(0);
            let b = cs.alloc(0);
            let c = cs.alloc(0);
            // let one = cs.set_one(a);
            // let one = cs.set_one(one);
            var = cs.one();

            cs.equal(&a + &b, cs.constant(0));
            cs.equal(cs.constant(0), &a + &b);
            cs.equal(&a + &b, &a + &b);
            cs.equal(&c * (&a + &b), &a + &b);

            let _e = cs.wire(&c * (&a + &b));
            let e = cs.alloc(c.raw() * (a.raw() + b.raw()));
            cs.equal(&c * (&a + &b), e);

            // cs.equal(c, 0);
            // let old = cs.set_one(var);
            // var = cs.set_one(old);
            // let b = var;
            // cs.one = var;
        }
        // println!("{}", var.raw());
    }
}
