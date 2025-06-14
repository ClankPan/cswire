use ark_ff::Field;

use crate::{variables::V, CSRef};


pub fn pow<F: Field>(cs: CSRef<F>, mut base: V<F>, mut exp: u64) -> V<F> {
    let mut pow = cs.one();
    while exp > 0 {
        if exp % 2 == 1 {
            pow = cs.wire(pow * &base);
        }
        base = cs.wire(&base * &base).into();
        exp /= 2;
    }

    pow.into()
}
