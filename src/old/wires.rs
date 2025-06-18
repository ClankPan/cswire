use ark_ff::Field;
use crate::expr::Idx;


#[derive(Clone, Copy, Debug)]
pub struct Wire<F: Field> {
    pub(crate) exp: Option<Idx>,
    pub(crate) val: F,
}
