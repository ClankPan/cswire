use ark_ff::Field;

use crate::Lin;

// 線形のものだけを通すトレイト
pub trait Linear<F: Field> {}

impl<F: Field> Linear<F> for Lin<'_, F> {}
impl<F: Field> Linear<F> for &Lin<'_, F> {}

macro_rules! impl_linear {
    ($primitive:ident) => {
        impl<F: Field> Linear<F> for $primitive {}
    };
    (&$primitive:ident) => {
        impl<F: Field> Linear<F> for &$primitive {}
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
