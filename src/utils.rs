use ark_ff::Field;

use crate::{variables::V, CSRef, Wire};

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

// Wireだけじゃなくて、Vも入れるようにする。
pub fn enforce_bits<F: Field>(cs: CSRef<F>, bits: &[Wire<F>]) {
    bits.iter().for_each(|b|cs.anchor((cs.one() - b) * b));
    
}
