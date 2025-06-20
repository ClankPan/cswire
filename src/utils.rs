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

pub fn augmented_circuit<'a, F: Field>(
    cs: &'a ConstraintSystemRef<'a, F>,
    s_i: F,
    z_0: &[F],
    z_i: &[F],
    u_r: &[F],
    u_i: &[F],
) -> (Wire<'a, F>, Vec<Wire<'a, F>>, Vec<Wire<'a, F>>) {
    let s_i = cs.alloc(s_i);
    let z_0: Vec<_> = z_0.iter().map(|v| cs.alloc(*v)).collect();
    let z_i: Vec<_> = z_i.iter().map(|v| cs.alloc(*v)).collect();
    let u_r: Vec<_> = u_r.iter().map(|v| cs.alloc(*v)).collect();
    let u_i: Vec<_> = u_i.iter().map(|v| cs.alloc(*v)).collect();
    
    (s_i, z_i,u_r)
}
