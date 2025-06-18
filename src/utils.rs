use ark_ff::Field;

use crate::{ConstraintSystemRef, V, Wire};

pub fn pow<F: Field>(cs: ConstraintSystemRef<F>, mut base: V<F>, mut exp: u64) -> V<F> {
    let mut pow = cs.one();
    while exp > 0 {
        if exp % 2 == 1 {
            pow = cs.wire(pow * &base);
        }
        base = cs.wire(&base * &base) * 1;
        exp /= 2;
    }

    pow * 1
}

// Wireだけじゃなくて、Vも入れるようにする。
pub fn enforce_bits<F: Field>(cs: ConstraintSystemRef<F>, bits: &[Wire<F>]) {
    bits.iter().for_each(|b| cs.link((cs.one() - b) * b, 0));
}
