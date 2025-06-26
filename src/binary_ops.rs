use std::{marker::PhantomData, ops::{Add, Mul, Sub}};

use ark_ff::Field;

use crate::{extract::{ToExpr, ToRaw}, Expr, Lin, Qua};

macro_rules! impl_op {
    ($trait:ident, $method:ident, $lhs:ty, $output:ty) => {
        impl_op!(@inner $trait, $method, $lhs, $lhs, $output);
        impl_op!(@inner $trait, $method, &$lhs, $lhs, $output);
        impl_op!(@inner $trait, $method, $lhs, &$lhs, $output);
        impl_op!(@inner $trait, $method, &$lhs, &$lhs, $output);
    };

    ($trait:ident, $method:ident, $lhs:ty, $rhs:ty, $output:ty) => {
        impl_op!(@inner $trait, $method, $lhs, $rhs, $output);
        impl_op!(@inner $trait, $method, &$lhs, $rhs, $output);
        impl_op!(@inner $trait, $method, $lhs, &$rhs, $output);
        impl_op!(@inner $trait, $method, &$lhs, &$rhs, $output);

        impl_op!(@inner $trait, $method, $rhs, $lhs, $output);
        impl_op!(@inner $trait, $method, &$rhs, $lhs, $output);
        impl_op!(@inner $trait, $method, $rhs, &$lhs, $output);
        impl_op!(@inner $trait, $method, &$rhs, &$lhs, $output);
    };
    (@inner $trait:ident, $method:ident, $lhs:ty, $rhs:ty, $output:ty) => {
        impl<'a, F: Field> $trait<$rhs> for $lhs {
            type Output = $output;

            fn $method(self, rhs: $rhs) -> Self::Output {
                Self::Output {
                    value: ToRaw::<F>::raw(&self).$method(ToRaw::<F>::raw(&rhs)),
                    expr: ToExpr::<F>::expr(&self).$method(ToExpr::<F>::expr(&rhs)),
                    _life: PhantomData,
                }
            }
        }
    };
}

impl_op!(Add, add, Lin<'a, F>, Lin<'a, F>);
impl_op!(Sub, sub, Lin<'a, F>, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, Qua<'a, F>);
impl_op!(Add, add, Lin<'a, F>, Qua<'a, F>, Qua<'a, F>);
impl_op!(Sub, sub, Lin<'a, F>, Qua<'a, F>, Qua<'a, F>);

// 係数
impl_op!(Mul, mul, Lin<'a, F>, bool, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, u8, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, u16, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, u32, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, u64, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, u128, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, i8, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, i16, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, i32, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, i64, Lin<'a, F>);
impl_op!(Mul, mul, Lin<'a, F>, i128, Lin<'a, F>);

// Fの係数
pub struct FF<F: Field>(pub F);
impl<'a, F: Field> Mul<FF<F>> for Lin<'a, F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: FF<F>) -> Self::Output {
        Self::Output {
            value: self.raw().mul(rhs.0),
            expr: self.expr().mul(Expr::coefficient(rhs.0)),
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<Lin<'a, F>> for FF<F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: Lin<'a, F>) -> Self::Output {
        Self::Output {
            value: self.0.mul(rhs.raw()),
            expr: Expr::coefficient(self.0).mul(rhs.expr()),
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<FF<F>> for &Lin<'a, F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: FF<F>) -> Self::Output {
        Self::Output {
            // value: Quadratic::<F>::raw(&self).mul(rhs.0),
            value: self.raw().mul(rhs.0),
            expr: self.expr().mul(Expr::coefficient(rhs.0)),
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<&Lin<'a, F>> for FF<F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: &Lin<'a, F>) -> Self::Output {
        Self::Output {
            value: self.0.mul(rhs.raw()),
            expr: Expr::coefficient(self.0).mul(rhs.expr()),
            _life: PhantomData,
        }
    }
}


#[cfg(test)]
mod tests {
    use ark_bn254::Fr;

    use crate::{extract::ToRaw, CSWire};

    #[test]
    pub fn test_binary_add() {
        let cs = CSWire::<Fr>::default();
        let a = Fr::from(10);
        let b = Fr::from(11);
        let c = cs.alloc(a) + cs.alloc(b);
        assert!(a + b == c.raw())
    }
}
