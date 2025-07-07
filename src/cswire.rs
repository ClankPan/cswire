use ark_ff::{BigInt, Field, PrimeField};
use itertools::Itertools;
use core::fmt;
use std::{cell::RefCell, cmp::Ordering, marker::PhantomData};

use crate::{Lc, Lin, Qua, binary_ops::FF};

macro_rules! iter {
    ($list:expr) => {
        $list.0.unwrap().into_iter()
    };
}

#[derive(Debug, Clone)]
pub(crate) struct InnerConstraint<F: Field>(pub F, pub Lc<F>, pub Lc<F>, pub Lc<F>);

#[derive(Debug, Clone)]
pub struct Constraint<F: Field> {
    pub a: Vec<(usize, F)>,
    pub b: Vec<(usize, F)>,
    pub c: Vec<(usize, F)>,
}

#[derive(Debug, Clone)]
pub struct R1CS<F: Field>(pub Vec<Constraint<F>>);

// 一貫性を保つためにCloneは実装しない。やるならRcとともに。
pub struct CSWire<F: Field> {
    one: RefCell<Lin<'static, F>>,
    witness: RefCell<Vec<F>>,
    exprs: RefCell<Vec<InnerConstraint<F>>>,
    do_compile: bool,
}

impl<F: Field> Default for CSWire<F> {
    fn default() -> Self {
        let one = Lin {
            value: F::ONE,
            lc: Lc::new(1, true),
            _life: PhantomData,
        };
        // IOの分を確保しておく。
        Self {
            one: RefCell::new(one),
            witness: RefCell::new(vec![]),
            exprs: RefCell::new(vec![]),
            do_compile: true,
        }
    }
}

impl<F: Field> CSWire<F> {
    pub fn new(do_compile: bool) -> Self {
        Self {
            do_compile,
            ..Default::default()
        }
    }

    fn alloc_inner<T>(&self, value: T) -> (usize, Lin<'_, F>)
    where
        F: From<T>,
    {
        let value = value.into();
        let mut witness = self.witness.borrow_mut();
        let index = witness.len();
        witness.push(value);
        let lin = Lin {
            value,
            lc: Lc::new(index, self.do_compile),
            _life: PhantomData,
        };
        (index, lin)
    }

    fn wire_inner<'a, Q>(&'a self, var: Q) -> (usize, Lin<'a, F>)
    where
        Q: Into<Qua<'a, F>>,
    {
        let v: Qua<'a, F> = var.into();

        // todo: もしwitnessそのものなら、そのまま返す。

        let (i, w) = self.alloc_inner(v.value);
        let c = v - &w; // 制約式はイコール・ゼロになる形で保管
        if self.do_compile {
            self.exprs
                .borrow_mut()
                .push(InnerConstraint(c.qc.0, c.qc.1, c.qc.2, c.lc));
        }
        (i, w)
    }

    pub fn alloc<T>(&self, value: T) -> Lin<'_, F>
    where
        F: From<T>,
    {
        let (_, lin) = self.alloc_inner(value);
        lin
    }

    pub fn wire<'a, Q>(&'a self, var: Q) -> Lin<'a, F>
    where
        Q: Into<Qua<'a, F>>,
    {
        let (_, lin) = self.wire_inner(var);
        lin
    }

    pub fn equal<'a, Q, L>(&'a self, qua: Q, lin: L)
    where
        Q: Into<Qua<'a, F>>,
        L: Into<Lin<'a, F>>,
    {
        let q: Qua<'a, F> = qua.into();
        let l: Lin<'a, F> = lin.into();

        // println!("q: {:?}, l: {:?}", q,l);
        let c = q - l; // 制約式はイコール・ゼロになる形で保管
        self.exprs
            .borrow_mut()
            .push(InnerConstraint(c.qc.0, c.qc.1, c.qc.2, c.lc));
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
            value: new.value, // accessor で値を取得
            lc: new.lc,
            _life: PhantomData, // 'static
        };
        self.one.replace(static_lin)
    }

    pub fn finish<'a, Q>(&'a self, io: &[Q]) -> (Vec<F>, Vec<F>, Option<R1CS<F>>)
    where
        Q: Into<Qua<'a, F>> + Clone,
    {
        // todo: Linがwireしたばかりのものだと、非効率な制約になってしまう
        let io: Vec<_> = io
            .iter()
            .cloned()
            .map(|v| self.wire_inner(v).0) // witnessのindexだけを取り出す
            .chain(std::iter::once(0)) // 定数
            .unique() // 重複を取り除く
            .sorted_unstable() // 昇順ソート
            .collect();

        let exprs = self.exprs.borrow().clone();
        let mut witness = self.witness.borrow().clone();
        let mut permu: Vec<usize> = (0..witness.len()).collect();
        for (i, j) in io.iter().enumerate() {
            permu.swap(i, *j);
            witness.swap(i, *j);
        }
        
        println!("do_compile: {}", self.do_compile);
        let constraint = if self.do_compile {
            let mut count = 0;
            let exprs_len = exprs.len();
            let constraints: Vec<_> = exprs
                .into_iter()
                .map(|expr| {
                    println!("{count}/{exprs_len}");
                    count += 1;
                    
                    let coeff = expr.0;
                    // println!("expr: {:?}", expr);
                    let a: Vec<_> = iter!(expr.1).map(|(i, f)| (i, coeff * f)).collect();
                    let b: Vec<_> = iter!(expr.2).collect();
                    let c: Vec<_> = iter!(expr.3).collect();
                    Constraint { a, b, c }
                })
                .collect();
            Some(R1CS(constraints))
        } else {
            None
        };

        let io = witness.drain(..io.len()).collect();

        (io, witness, constraint)
    }
}

impl<F: Field> R1CS<F> {
    pub fn is_satisfied(&self, x: &[F], w: &[F]) -> bool {
        let w = [x, w].concat();
        for constraint in &self.0 {
            let a: F = constraint.a.iter().map(|(i, f)| *f * w[*i]).sum();
            let b: F = constraint.a.iter().map(|(i, f)| *f * w[*i]).sum();
            let c: F = constraint.c.iter().map(|(i, f)| *f * w[*i]).sum();
            if a * b + c == F::ZERO {
                return false
            }
        }
        true
    }

    pub fn num_of_constraints(&self) -> usize {
        self.0.len()
    }
}



// #[cfg(test)]
// mod tests {
//     use crate::extract::ToRaw;
//
//     use super::CSWire;
//     use ark_bn254::Fr;
//
//     #[test]
//     pub fn test_alloc_ref() {
//         let mut var;
//         {
//             let cs = CSWire::<Fr>::default();
//             let a = cs.alloc(0);
//             let b = cs.alloc(0);
//             let c = cs.alloc(0);
//             // let one = cs.set_one(a);
//             // let one = cs.set_one(one);
//             var = cs.one();
//
//             cs.equal(&a + &b, cs.constant(0));
//             cs.equal(cs.constant(0), &a + &b);
//             cs.equal(&a + &b, &a + &b);
//             cs.equal(&c * (&a + &b), &a + &b);
//
//             let _e = cs.wire(&c * (&a + &b));
//             let e = cs.alloc(c.raw() * (a.raw() + b.raw()));
//             cs.equal(&c * (&a + &b), e);
//
//             // cs.equal(c, 0);
//             // let old = cs.set_one(var);
//             // var = cs.set_one(old);
//             // let b = var;
//             // cs.one = var;
//         }
//         // println!("{}", var.raw());
//     }
// }
