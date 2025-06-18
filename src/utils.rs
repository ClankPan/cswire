use ark_ff::Field;

use crate::{ ConstraintSystemRef, Wire, V};

pub fn pow<'a, F: Field>(cs: ConstraintSystemRef<'a, F>, mut base: V<'a, F>, mut exp: u64) -> V<'a, F> {
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
