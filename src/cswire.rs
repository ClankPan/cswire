use std::{cell::RefCell, marker::PhantomData};

use ark_ff::Field;

use crate::{
    Expr,
    binary_ops::FF,
    extract::{ToExpr, ToRaw},
    linear::Linear,
    quadratic::Quadratic,
};
pub type V<'a, F> = Lin<'a, F>;
pub type VV<'a, F> = Qua<'a, F>;

#[derive(Clone)]
pub struct Lin<'a, F: Field> {
    pub(crate) value: F,
    pub(crate) expr: Expr<F>,
    pub(crate) _life: PhantomData<&'a ()>,
}

#[derive(Clone)]
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

            cs.equal(&a + &b, 0);
            cs.equal(0, &a + &b);
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
