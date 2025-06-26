use ark_ff::Field;

use crate::{CSWire, Lin, extract::ToRaw};

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

pub fn is_zero<'a, F: Field>(cs: &'a CSWire<F>, v: &Lin<'a, F>) -> Lin<'a, F> {
    // v · z = 1 − q
    // v · q = 0
    let is_zero: bool = v.raw() == F::ZERO;
    let q = cs.alloc(is_zero);
    let z = if is_zero {
        cs.alloc(0)
    } else {
        let inv = v.raw().inverse().unwrap(); // vはzeroではないので逆元はある。
        cs.alloc(inv)
    };

    cs.equal(v * z, cs.one() - &q);
    cs.equal(v * &q, 0);
    q
}
