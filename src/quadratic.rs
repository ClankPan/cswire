use crate::{
    Expr,
    cswire::{Lin, Qua},
};
use ark_ff::Field;

// 変数を全て通すトレイト。
pub trait Quadratic<F: Field> {}

impl<F: Field> Quadratic<F> for Lin<'_, F> {}
impl<F: Field> Quadratic<F> for Qua<'_, F> {}
impl<F: Field> Quadratic<F> for &Lin<'_, F> {}
impl<F: Field> Quadratic<F> for &Qua<'_, F> {}

macro_rules! impl_quadratic {
    ($primitive:ident) => {
        impl<F: Field> Quadratic<F> for $primitive {}
    };
    (&$primitive:ident) => {
        impl<F: Field> Quadratic<F> for &$primitive {}
    };
}

impl_quadratic!(bool);
impl_quadratic!(u8);
impl_quadratic!(u16);
impl_quadratic!(u32);
impl_quadratic!(u64);
impl_quadratic!(u128);
impl_quadratic!(i8);
impl_quadratic!(i16);
impl_quadratic!(i32);
impl_quadratic!(i64);
impl_quadratic!(i128);

impl_quadratic!(&bool);
impl_quadratic!(&u8);
impl_quadratic!(&u16);
impl_quadratic!(&u32);
impl_quadratic!(&u64);
impl_quadratic!(&u128);
impl_quadratic!(&i8);
impl_quadratic!(&i16);
impl_quadratic!(&i32);
impl_quadratic!(&i64);
impl_quadratic!(&i128);
