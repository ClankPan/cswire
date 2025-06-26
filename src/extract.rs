use ark_ff::Field;

use crate::{Expr, Lin, Qua};


pub trait ToRaw<F: Field> {
    fn raw(&self) -> F;
}

pub trait ToExpr<F: Field> {
    fn expr(&self) -> Expr<F>;
}



impl<F: Field> ToRaw<F> for Lin<'_, F> {
    fn raw(&self) -> F {
        self.value
    }
}

impl<F: Field> ToRaw<F> for &Lin<'_, F> {
    fn raw(&self) -> F {
        self.value
    }
}

impl<F: Field> ToExpr<F> for Lin<'_, F> {
    fn expr(&self) -> Expr<F> {
        self.expr.clone()
    }
}

impl<F: Field> ToExpr<F> for &Lin<'_, F> {
    fn expr(&self) -> Expr<F> {
        self.expr.clone()
    }
}

impl<F: Field> ToRaw<F> for Qua<'_, F> {
    fn raw(&self) -> F {
        self.value
    }
}

impl<F: Field> ToRaw<F> for &Qua<'_, F> {
    fn raw(&self) -> F {
        self.value
    }
}

impl<F: Field> ToExpr<F> for Qua<'_, F> {
    fn expr(&self) -> Expr<F> {
        self.expr.clone()
    }
}

impl<F: Field> ToExpr<F> for &Qua<'_, F> {
    fn expr(&self) -> Expr<F> {
        self.expr.clone()
    }
}




macro_rules! impl_linear {
    ($primitive:ident) => {
        impl<F: Field> ToRaw<F> for $primitive {
            fn raw(&self) -> F {
                F::from(*self)
            }
        }

        impl<F: Field> ToExpr<F> for $primitive {
            fn expr(&self) -> Expr<F> {
                Expr::coefficient(*self)
            }
        }
    };
    (&$primitive:ident) => {
        impl<F: Field> ToRaw<F> for &$primitive {
            fn raw(&self) -> F {
                F::from(**self)
            }
        }
        impl<F: Field> ToExpr<F> for &$primitive {

            fn expr(&self) -> Expr<F> {
                Expr::coefficient(**self)
            }
        }
    };
}

impl_linear!(bool);
impl_linear!(u8);
impl_linear!(u16);
impl_linear!(u32);
impl_linear!(u64);
impl_linear!(u128);
impl_linear!(i8);
impl_linear!(i16);
impl_linear!(i32);
impl_linear!(i64);
impl_linear!(i128);

impl_linear!(&bool);
impl_linear!(&u8);
impl_linear!(&u16);
impl_linear!(&u32);
impl_linear!(&u64);
impl_linear!(&u128);
impl_linear!(&i8);
impl_linear!(&i16);
impl_linear!(&i32);
impl_linear!(&i64);
impl_linear!(&i128);
