use ark_ff::Field;

use crate::{CSWire, Lin};

pub fn pow<'a, F: Field>(cs: &'a CSWire<F>, mut base: Lin<'a, F>, mut exp: u64) -> Lin<'a, F> {
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

pub fn enforce_bits<'a, F: Field>(cs: &'a CSWire<F>, bits: &[Lin<'a, F>]) {
    bits.iter().for_each(|b| cs.equal((cs.one() - b) * b, 0));
}
