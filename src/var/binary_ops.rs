use ark_ff::Field;
use std::{
    collections::HashMap,
    marker::PhantomData,
    ops::{Add, Mul, Sub},
};

use super::{Lc, Lin, Qua};

macro_rules! impl_lin_op {

    () => {
        impl_lin_op!(@inner Add, add, Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@inner Add, add, &Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@inner Add, add, Lin<'a,F>, &Lin<'a,F>);
        impl_lin_op!(@inner Add, add, &Lin<'a,F>, &Lin<'a,F>);

        impl_lin_op!(@inner Sub, sub, Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@inner Sub, sub, &Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@inner Sub, sub, Lin<'a,F>, &Lin<'a,F>);
        impl_lin_op!(@inner Sub, sub, &Lin<'a,F>, &Lin<'a,F>);

        impl_lin_op!(@mul  Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@mul  &Lin<'a,F>, Lin<'a,F>);
        impl_lin_op!(@mul  Lin<'a,F>, &Lin<'a,F>);
        impl_lin_op!(@mul  &Lin<'a,F>, &Lin<'a,F>);
    };
    (@inner $trait:ident, $method:ident, $lhs:ty, $rhs:ty) => {
        impl<'a, F: Field> $trait<$rhs> for $lhs {
            type Output = Lin<'a,F>;

            fn $method(self, rhs: $rhs) -> Self::Output {
                Lin {
                    value: self.value.$method(rhs.value),
                    lc: self.lc.clone().$method(rhs.lc.clone()),
                    _life: PhantomData,
                }
            }
        }
    };

    (@mul $lhs:ty, $rhs:ty) => {
        impl<'a, F: Field> Mul<$rhs> for $lhs {
            type Output = Qua<'a,F>;

            fn mul(self, rhs: $rhs) -> Self::Output {
                Qua {
                    value: self.value * rhs.value,
                    qc: (F::ONE, self.lc.clone(), rhs.lc.clone()),
                    lc: Lc(Some(HashMap::new())),
                    _life: PhantomData,
                }
            }
        }
    };
}
impl_lin_op!();

macro_rules! impl_qua_op {

    () => {
        impl_qua_op!(@inner Add, add, Qua<'a,F>, Lin<'a,F>);
        impl_qua_op!(@inner Add, add, Qua<'a,F>, &Lin<'a,F>);
        impl_qua_op!(@inner Add, add, &Qua<'a,F>, Lin<'a,F>);
        impl_qua_op!(@inner Add, add, &Qua<'a,F>, &Lin<'a,F>);

        impl_qua_op!(@inner Sub, sub, Qua<'a,F>, Lin<'a,F>);
        impl_qua_op!(@inner Sub, sub, Qua<'a,F>, &Lin<'a,F>);
        impl_qua_op!(@inner Sub, sub, &Qua<'a,F>, Lin<'a,F>);
        impl_qua_op!(@inner Sub, sub, &Qua<'a,F>, &Lin<'a,F>);
    };
    (@inner $trait:ident, $method:ident, $a:ty, $b:ty) => {
        impl<'a, F: Field> $trait<$b> for $a {
            type Output = Qua<'a,F>;

            fn $method(self, rhs: $b) -> Self::Output {
                Qua {
                    qc: self.qc.clone(),
                    value: self.value.$method(rhs.value),
                    lc: self.lc.clone().$method( rhs.lc.clone()),
                    _life: PhantomData,
                }
            }
        }
        impl<'a, F: Field> $trait<$a> for $b {
            type Output = Qua<'a,F>;

            fn $method(self, rhs: $a) -> Self::Output {
                Qua {
                    qc: rhs.qc.clone(),
                    value: self.value.$method(rhs.value),
                    lc: self.lc.clone().$method(rhs.lc.clone()),
                    _life: PhantomData,
                }
            }
        }
    };
}
impl_qua_op!();

macro_rules! impl_coef_lin_op {

    () => {
        impl_coef_lin_op!(@inner bool);
        impl_coef_lin_op!(@inner u8);
        impl_coef_lin_op!(@inner u16);
        impl_coef_lin_op!(@inner u32);
        impl_coef_lin_op!(@inner u64);
        impl_coef_lin_op!(@inner u128);
        impl_coef_lin_op!(@inner i8);
        impl_coef_lin_op!(@inner i16);
        impl_coef_lin_op!(@inner i32);
        impl_coef_lin_op!(@inner i64);
        impl_coef_lin_op!(@inner i128);

        impl_coef_lin_op!(@inner &bool);
        impl_coef_lin_op!(@inner &u8);
        impl_coef_lin_op!(@inner &u16);
        impl_coef_lin_op!(@inner &u32);
        impl_coef_lin_op!(@inner &u64);
        impl_coef_lin_op!(@inner &u128);
        impl_coef_lin_op!(@inner &i8);
        impl_coef_lin_op!(@inner &i16);
        impl_coef_lin_op!(@inner &i32);
        impl_coef_lin_op!(@inner &i64);
        impl_coef_lin_op!(@inner &i128);

    };
    (@inner $p:ty) => {
        impl<'a, F: Field> Mul<$p> for Lin<'a, F> {
            type Output = Lin<'a, F>;

            fn mul(self, rhs: $p) -> Self::Output {
                let coeff = F::from(rhs.clone());
                Lin {
                    value: self.value * coeff,
                    lc: self.lc * coeff,
                    _life: PhantomData,
                }
            }
        }
        impl<'a, F: Field> Mul<Lin<'a,F>> for $p {
            type Output = Lin<'a, F>;

            fn mul(self, rhs: Lin<'a,F>) -> Self::Output {
                let coeff = F::from(self.clone());
                Lin {
                    value: rhs.value * coeff,
                    lc: rhs.lc * coeff,
                    _life: PhantomData,
                }
            }
        }

        impl<'a, F: Field> Mul<$p> for &Lin<'a, F> {
            type Output = Lin<'a, F>;

            fn mul(self, rhs: $p) -> Self::Output {
                let coeff = F::from(rhs.clone());
                Lin {
                    value: self.value * coeff,
                    lc: self.lc.clone() * coeff,
                    _life: PhantomData,
                }
            }
        }
        impl<'a, F: Field> Mul<&Lin<'a,F>> for $p {
            type Output = Lin<'a, F>;

            fn mul(self, rhs: &Lin<'a,F>) -> Self::Output {
                let coeff = F::from(self.clone());
                Lin {
                    value: rhs.value * coeff,
                    lc: rhs.lc.clone() * coeff,
                    _life: PhantomData,
                }
            }
        }
    };

}
impl_coef_lin_op!();

macro_rules! impl_coef_qua_op {

    () => {
        impl_coef_qua_op!(@inner bool);
        impl_coef_qua_op!(@inner u8);
        impl_coef_qua_op!(@inner u16);
        impl_coef_qua_op!(@inner u32);
        impl_coef_qua_op!(@inner u64);
        impl_coef_qua_op!(@inner u128);
        impl_coef_qua_op!(@inner i8);
        impl_coef_qua_op!(@inner i16);
        impl_coef_qua_op!(@inner i32);
        impl_coef_qua_op!(@inner i64);
        impl_coef_qua_op!(@inner i128);

        impl_coef_qua_op!(@inner &bool);
        impl_coef_qua_op!(@inner &u8);
        impl_coef_qua_op!(@inner &u16);
        impl_coef_qua_op!(@inner &u32);
        impl_coef_qua_op!(@inner &u64);
        impl_coef_qua_op!(@inner &u128);
        impl_coef_qua_op!(@inner &i8);
        impl_coef_qua_op!(@inner &i16);
        impl_coef_qua_op!(@inner &i32);
        impl_coef_qua_op!(@inner &i64);
        impl_coef_qua_op!(@inner &i128);

    };
    (@inner $p:ty) => {

        impl<'a, F: Field> Mul<$p> for Qua<'a, F> {
            type Output = Qua<'a, F>;

            fn mul(mut self, rhs: $p) -> Self::Output {
                let coeff = F::from(rhs.clone());
                self.qc.0 *= coeff;
                Qua {
                    value: self.value * coeff,
                    qc: self.qc,
                    lc: self.lc * coeff,
                    _life: PhantomData,
                }
            }
        }

        impl<'a, F: Field> Mul<Qua<'a, F>> for $p {
            type Output = Qua<'a, F>;

            fn mul(self, mut rhs: Qua<'a, F>) -> Self::Output {
                let coeff = F::from(self.clone());
                rhs.qc.0 *= coeff;
                Qua {
                    value: rhs.value * coeff,
                    qc: rhs.qc,
                    lc: rhs.lc * coeff,
                    _life: PhantomData,
                }
            }
        }

        impl<'a, F: Field> Mul<$p> for &Qua<'a, F> {
            type Output = Qua<'a, F>;

            fn mul(self, rhs: $p) -> Self::Output {
                let mut lhs = self.clone();
                let coeff = F::from(rhs.clone());
                lhs.qc.0 *= coeff;
                Qua {
                    value: self.value * coeff,
                    qc: lhs.qc,
                    lc: lhs.lc * coeff,
                    _life: PhantomData,
                }
            }
        }

        impl<'a, F: Field> Mul<&Qua<'a, F>> for $p {
            type Output = Qua<'a, F>;

            fn mul(self, rhs: &Qua<'a, F>) -> Self::Output {
                let mut rhs = rhs.clone();
                let coeff = F::from(self.clone());
                rhs.qc.0 *= coeff;
                Qua {
                    value: rhs.value * coeff,
                    qc: rhs.qc,
                    lc: rhs.lc * coeff,
                    _life: PhantomData,
                }
            }
        }
    };

}
impl_coef_qua_op!();



// Fの係数
pub struct FF<F: Field>(pub F);
impl<'a, F: Field> Mul<FF<F>> for Lin<'a, F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: FF<F>) -> Self::Output {
        Self::Output {
            value: self.value.mul(rhs.0),
            lc: self.lc * rhs.0,
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<Lin<'a, F>> for FF<F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: Lin<'a, F>) -> Self::Output {
        Self::Output {
            value: self.0.mul(rhs.value),
            lc: rhs.lc * self.0,
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<FF<F>> for &Lin<'a, F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: FF<F>) -> Self::Output {
        Self::Output {
            value: self.value.mul(rhs.0),
            lc: self.lc.clone() * rhs.0,
            _life: PhantomData,
        }
    }
}
impl<'a, F: Field> Mul<&Lin<'a, F>> for FF<F> {
    type Output = Lin<'a, F>;

    fn mul(self, rhs: &Lin<'a, F>) -> Self::Output {
        Self::Output {
            value: self.0.mul(rhs.raw()),
            lc: rhs.lc.clone() * self.0,
            _life: PhantomData,
        }
    }
}
