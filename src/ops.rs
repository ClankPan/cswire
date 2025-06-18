use std::{iter::Sum, ops::{Add, AddAssign, Mul, Sub}};

use ark_ff::Field;

use crate::{Coeff, Expr, Wire, V, VV};

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
        impl<F: Field> $trait<$rhs> for $lhs {
            type Output = $output;
            fn $method(self, rhs: $rhs) -> Self::Output {
                let (lhs_val, lhs_exp) = self.parse();
                let (rhs_val, rhs_exp) = rhs.parse();
                Self::Output {
                    exp: Expr::$trait(Box::new(lhs_exp), Box::new(rhs_exp)),
                    val: lhs_val.$method(rhs_val),
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

impl_op!(Add, add, Wire<F>, V<F>);
impl_op!(Add, add, V<F>, V<F>);
impl_op!(Add, add, Wire<F>, V<F>, V<F>);
impl_op!(Add, add, Wire<F>, VV<F>, VV<F>);
impl_op!(Add, add, V<F>, VV<F>, VV<F>);

impl_op!(Sub, sub, Wire<F>, V<F>);
impl_op!(Sub, sub, V<F>, V<F>);
impl_op!(Sub, sub, Wire<F>, V<F>, V<F>);
impl_op!(Sub, sub, Wire<F>, VV<F>, VV<F>);
impl_op!(Sub, sub, V<F>, VV<F>, VV<F>);

impl_op!(Mul, mul, Wire<F>, VV<F>);
impl_op!(Mul, mul, V<F>, VV<F>);
impl_op!(Mul, mul, Wire<F>, V<F>, VV<F>);

impl_op!(Mul, mul, Wire<F>, bool, V<F>);
impl_op!(Mul, mul, Wire<F>, u8, V<F>);
impl_op!(Mul, mul, Wire<F>, u16, V<F>);
impl_op!(Mul, mul, Wire<F>, u32, V<F>);
impl_op!(Mul, mul, Wire<F>, u64, V<F>);
impl_op!(Mul, mul, V<F>, bool, V<F>);
impl_op!(Mul, mul, V<F>, u8, V<F>);
impl_op!(Mul, mul, V<F>, u16, V<F>);
impl_op!(Mul, mul, V<F>, u32, V<F>);
impl_op!(Mul, mul, V<F>, u64, V<F>);
impl_op!(Mul, mul, VV<F>, bool, V<F>);
impl_op!(Mul, mul, VV<F>, u8, V<F>);
impl_op!(Mul, mul, VV<F>, u16, V<F>);
impl_op!(Mul, mul, VV<F>, u32, V<F>);
impl_op!(Mul, mul, VV<F>, u64, V<F>);

impl_op!(Mul, mul, Wire<F>, i8, V<F>);
impl_op!(Mul, mul, Wire<F>, i16, V<F>);
impl_op!(Mul, mul, Wire<F>, i32, V<F>);
impl_op!(Mul, mul, Wire<F>, i64, V<F>);
impl_op!(Mul, mul, V<F>, i8, V<F>);
impl_op!(Mul, mul, V<F>, i16, V<F>);
impl_op!(Mul, mul, V<F>, i32, V<F>);
impl_op!(Mul, mul, V<F>, i64, V<F>);
impl_op!(Mul, mul, VV<F>, i8, V<F>);
impl_op!(Mul, mul, VV<F>, i16, V<F>);
impl_op!(Mul, mul, VV<F>, i32, V<F>);
impl_op!(Mul, mul, VV<F>, i64, V<F>);

impl_op!(Mul, mul, Wire<F>, Coeff<F>, V<F>);
impl_op!(Mul, mul, V<F>, Coeff<F>, V<F>);
impl_op!(Mul, mul, VV<F>, Coeff<F>, VV<F>);




impl<F: Field> AddAssign<Wire<F>> for V<F> {
    fn add_assign(&mut self, rhs: Wire<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<V<F>> for V<F> {
    fn add_assign(&mut self, rhs: V<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<&Wire<F>> for V<F> {
    fn add_assign(&mut self, rhs: &Wire<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<&V<F>> for V<F> {
    fn add_assign(&mut self, rhs: &V<F>) {
        *self = &*self + rhs;
    }
}

impl<F: Field> Sum<Wire<F>> for V<F> {
    fn sum<I: Iterator<Item = Wire<F>>>(iter: I) -> Self {
        iter.map(|i| i * 1)
            .reduce(|acc, x| acc + x)
            .expect("length is zero")
    }
}

impl<F: Field> Sum<V<F>> for V<F> {
    fn sum<I: Iterator<Item = V<F>>>(iter: I) -> Self {
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

impl<F: Field> Parse<F> for Wire<F> {
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


impl<F: Field> Parse<F> for V<F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = self.exp.clone();
        let val = self.val;
        (val, exp)
    }
}

impl<F: Field> Parse<F> for VV<F> {
    fn parse(&self) -> (F, Expr<F>) {
        let exp = self.exp.clone();
        let val = self.val;
        (val, exp)
    }
}

#[cfg(test)]
mod tests {
    use ark_bn254::Fr;

    use crate::{Coeff, ConstraintSystemRef};


    #[test]
    fn test_coeff() {
        let a = Fr::from(123);
        let cs = ConstraintSystemRef::<Fr>::new();
        let wire = cs.alloc(123);
        let _b = Coeff(a) * wire;
    }
}
