use std::{
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub},
};

use ark_ff::Field;

use crate::{Coeff, Expr, V, VV, Wire};

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
                let life = self.life;
                let (lhs_val, lhs_exp) = self.parse();
                let (rhs_val, rhs_exp) = rhs.parse();
                Self::Output {
                    exp: Expr::$trait(Box::new(lhs_exp), Box::new(rhs_exp)),
                    val: lhs_val.$method(rhs_val),
                    life
                }
            }
        }
    };
    (@coeff $trait:ident, $method:ident, $lhs:ty, $rhs:ty, $output:ty) => {
        impl_op!(@coeff_self $trait, $method, $lhs, $rhs, $output);
        impl_op!(@coeff_self $trait, $method, &$lhs, $rhs, $output);
        impl_op!(@coeff_self $trait, $method, $lhs, &$rhs, $output);
        impl_op!(@coeff_self $trait, $method, &$lhs, &$rhs, $output);

        impl_op!(@coeff_rhs $trait, $method, $rhs, $lhs, $output);
        impl_op!(@coeff_rhs $trait, $method, &$rhs, $lhs, $output);
        impl_op!(@coeff_rhs $trait, $method, $rhs, &$lhs, $output);
        impl_op!(@coeff_rhs $trait, $method, &$rhs, &$lhs, $output);
    };
    (@coeff_self $trait:ident, $method:ident, $lhs:ty, $rhs:ty, $output:ty) => {
        impl<'a, F: Field> $trait<$rhs> for $lhs {
            type Output = $output;
            fn $method(self, rhs: $rhs) -> Self::Output {
                let life = self.life;
                let (lhs_val, lhs_exp) = self.parse();
                let (rhs_val, rhs_exp) = rhs.parse();
                Self::Output {
                    exp: Expr::$trait(Box::new(lhs_exp), Box::new(rhs_exp)),
                    val: lhs_val.$method(rhs_val),
                    life
                }
            }
        }
    };
    (@coeff_rhs $trait:ident, $method:ident, $lhs:ty, $rhs:ty, $output:ty) => {
        impl<'a, F: Field> $trait<$rhs> for $lhs {
            type Output = $output;
            fn $method(self, rhs: $rhs) -> Self::Output {
                let life = rhs.life;
                let (lhs_val, lhs_exp) = self.parse();
                let (rhs_val, rhs_exp) = rhs.parse();
                Self::Output {
                    exp: Expr::$trait(Box::new(lhs_exp), Box::new(rhs_exp)),
                    val: lhs_val.$method(rhs_val),
                    life
                }
            }
        }
    };
}

// 定数との加算は、oneのindexがswitchboardによって変わる可能性があるので、定義できない。
// impl_op!(Add, add, Wire<F>, bool, V<F>);
// impl_op!(Add, add, Wire<F>, u8, V<F>);
// impl_op!(Add, add, Wire<F>, u16, V<F>);
// impl_op!(Add, add, Wire<F>, u32, V<F>);
// impl_op!(Add, add, Wire<F>, u64, V<F>);
// impl_op!(Add, add, V<F>, bool, V<F>);
// impl_op!(Add, add, V<F>, u8, V<F>);
// impl_op!(Add, add, V<F>, u16, V<F>);
// impl_op!(Add, add, V<F>, u32, V<F>);
// impl_op!(Add, add, V<F>, u64, V<F>);
// impl_op!(Add, add, VV<F>, bool, V<F>);
// impl_op!(Add, add, VV<F>, u8, V<F>);
// impl_op!(Add, add, VV<F>, u16, V<F>);
// impl_op!(Add, add, VV<F>, u32, V<F>);
// impl_op!(Add, add, VV<F>, u64, V<F>);
// impl_op!(Sub,sub, Wire<F>, bool, V<F>);
// impl_op!(Sub,sub, Wire<F>, u8, V<F>);
// impl_op!(Sub,sub, Wire<F>, u16, V<F>);
// impl_op!(Sub,sub, Wire<F>, u32, V<F>);
// impl_op!(Sub,sub, Wire<F>, u64, V<F>);
// impl_op!(Sub,sub, V<F>, bool, V<F>);
// impl_op!(Sub,sub, V<F>, u8, V<F>);
// impl_op!(Sub,sub, V<F>, u16, V<F>);
// impl_op!(Sub,sub, V<F>, u32, V<F>);
// impl_op!(Sub,sub, V<F>, u64, V<F>);
// impl_op!(Sub,sub, VV<F>, bool, V<F>);
// impl_op!(Sub,sub, VV<F>, u8, V<F>);
// impl_op!(Sub,sub, VV<F>, u16, V<F>);
// impl_op!(Sub,sub, VV<F>, u32, V<F>);
// impl_op!(Sub,sub, VV<F>, u64, V<F>);

impl_op!(Add, add, Wire<'a, F>, V<'a, F>);
impl_op!(Add, add, V<'a, F>, V<'a, F>);
impl_op!(Add, add, Wire<'a, F>, V<'a, F>, V<'a, F>);
impl_op!(Add, add, Wire<'a, F>, VV<'a, F>, VV<'a, F>);
impl_op!(Add, add, V<'a, F>, VV<'a, F>, VV<'a, F>);
//
impl_op!(Sub, sub, Wire<'a, F>, V<'a, F>);
impl_op!(Sub, sub, V<'a, F>, V<'a, F>);
impl_op!(Sub, sub, Wire<'a, F>, V<'a, F>, V<'a, F>);
impl_op!(Sub, sub, Wire<'a, F>, VV<'a, F>, VV<'a, F>);
impl_op!(Sub, sub, V<'a, F>, VV<'a, F>, VV<'a, F>);
//
impl_op!(Mul, mul, Wire<'a, F>, VV<'a, F>);
impl_op!(Mul, mul, V<'a, F>, VV<'a, F>);
impl_op!(Mul, mul, Wire<'a, F>, V<'a, F>, VV<'a, F>);
//
impl_op!(@coeff Mul, mul, Wire<'a, F>, bool, V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, u8, V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, u16, V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, u32, V<'a,F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, u64, V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, bool, V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, u8, V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, u16, V<'a,F>);
impl_op!(@coeff Mul, mul, V<'a, F>, u32, V<'a,F>);
impl_op!(@coeff Mul, mul, V<'a, F>, u64, V<'a,F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, bool, V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, u8, V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, u16, V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, u32, V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, u64, V<'a, F>);

impl_op!(@coeff Mul, mul, Wire<'a, F>, i8,  V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, i16, V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, i32, V<'a, F>);
impl_op!(@coeff Mul, mul, Wire<'a, F>, i64, V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, i8,     V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, i16,    V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, i32,    V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, i64,    V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, i8,    V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, i16,   V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, i32,   V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, i64,   V<'a, F>);

impl_op!(@coeff Mul, mul, Wire<'a, F>, Coeff<F>, V<'a, F>);
impl_op!(@coeff Mul, mul, V<'a, F>, Coeff<F>, V<'a, F>);
impl_op!(@coeff Mul, mul, VV<'a, F>, Coeff< F>, VV<'a, F>);
//

impl<'a, F: Field> AddAssign<Wire<'a, F>> for V<'a, F> {
    fn add_assign(&mut self, rhs: Wire<'a, F>) {
        *self = &*self + rhs;
    }
}
impl<'a, F: Field> AddAssign<V<'a, F>> for V<'a, F> {
    fn add_assign(&mut self, rhs: V<'a, F>) {
        *self = &*self + rhs;
    }
}
impl<'a, F: Field> AddAssign<&Wire<'a, F>> for V<'a, F> {
    fn add_assign(&mut self, rhs: &Wire<'a, F>) {
        *self = &*self + rhs;
    }
}
impl<'a, F: Field> AddAssign<&V<'a, F>> for V<'a, F> {
    fn add_assign(&mut self, rhs: &V<'a, F>) {
        *self = &*self + rhs;
    }
}

impl<'a, F: Field> Sum<Wire<'a, F>> for V<'a, F> {
    fn sum<I: Iterator<Item = Wire<'a, F>>>(iter: I) -> Self {
        iter.map(|i| i * 1)
            .reduce(|acc, x| acc + x)
            .expect("length is zero")
    }
}

impl<'a, F: Field> Sum<V<'a, F>> for V<'a, F> {
    fn sum<I: Iterator<Item = V<'a, F>>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).expect("length is zero")
    }
}

trait Parse<F: Field> {
    fn parse(&self) -> (F, Expr<F>);
}

impl<F: Field> Parse<F> for Coeff<F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con(self.0);
        let val = self.0;
        (val, exp)
    }
}

impl<F: Field> Parse<F> for bool {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for u8 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for u16 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for u32 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for u64 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<'a, F: Field> Parse<F> for Wire<'a, F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = self.exp.into();
        let val = self.val;
        (val, exp)
    }
}

impl<F: Field> Parse<F> for i8 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for i16 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for i32 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<F: Field> Parse<F> for i64 {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = Expr::Con((*self).into());
        let val = F::from(*self);
        (val, exp)
    }
}

impl<'a, F: Field> Parse<F> for V<'a, F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = self.exp.clone();
        let val = self.val;
        (val, exp)
    }
}

impl<'a, F: Field> Parse<F> for VV<'a, F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = self.exp.clone();
        let val = self.val;
        (val, exp)
    }
}

#[cfg(test)]
mod tests {
    use ark_bn254::Fr;

    use crate::{Coeff, ConstraintSystem};

    #[test]
    fn test_coeff() {
        // let a = Fr::from(123);
        // let cs = ConstraintSystemRef::<Fr>::new();
        // let wire = cs.alloc(123);
        // let _b = Coeff(a) * wire;
    }
}
