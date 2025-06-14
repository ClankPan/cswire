#[cfg(test)]
mod tests;
mod poseidon;
mod variables;

use ark_bn254::Fr;
use ark_ff::{BigInteger, PrimeField};
use variables::{ConstraintSystem, ConstraintSystemRef, Mode, V, Wire};


pub fn example(cs: ConstraintSystemRef<Fr>, a: Wire<Fr>, b: Wire<Fr>, c: Wire<Fr>) -> Wire<Fr> {
    let z = cs.wire(a * b - c);
    let mut pow = cs.one();
    let mut pows = vec![];
    for _ in 0..10 {
        pows.push(pow);
        pow = cs.wire(z * pow);
    }
    let poly_sum = pows
        .into_iter()
        .enumerate()
        .map(|(c, p)| (c + 1) * p)
        .sum::<V<Fr>>();
    cs.wire(poly_sum)
}

// ランダムな値をWireに入れて同じ多項式になるかを確認することで、ユーザに非決定的な制約かを伝える。
// 回路は動的に変えてもいいが、それがWireの値によるものはだめ。
// cs.wireは回路を生成しつつも、Witnessも計算する？
// cs.wireすると、csに値が記録される。
// Wire変数は確実に値が記録されていることを保証したいから、係数付きは別の型にしたほうがいいかも。
pub fn bit_decompose(cs: &mut ConstraintSystem<Fr>, v: Wire<Fr>) -> Vec<Wire<Fr>> {
    let bits = v.raw().into_bigint().to_bits_le();
    let bits: Vec<Wire<Fr>> = bits.into_iter().map(|b| cs.alloc(b)).collect();
    bits.iter().for_each(|b| cs.anchor((cs.one() - b) * b));
    cs.anchor(
        bits.iter()
            .enumerate()
            .map(|(i, b)| (1 << i) * *b)
            .sum::<V<Fr>>()
            - v,
    );
    bits
}
pub type CS = ConstraintSystem<Fr>;
pub type CSRef = ConstraintSystemRef<Fr>;

pub fn sample() -> Vec<Fr> {
    let cs = CS::new_ref(Mode::Compile);
    let (a, b, c) = (cs.alloc(11), cs.alloc(22), cs.alloc(33));
    let _ = ranged_linear_combination(cs.clone(), a, b, c);
    cs.compile();
    cs.witnesses()
}

pub fn ranged_linear_combination(cs: CSRef, a: Wire<Fr>, b: Wire<Fr>, c: Wire<Fr>) -> Wire<Fr> {
    let d = cs.wire(a + b * c);
    range_check(cs, &d, 32);
    d
}

pub fn range_check(cs: CSRef, v: &Wire<Fr>, bit_range: usize) {
    let one = cs.one();
    let bits = v.raw().into_bigint().to_bits_le();
    // Allocate bits from non-deterministic
    let bits: Vec<_> = bits.iter().map(|b| cs.alloc(*b)).collect();
    // Enforce b is 0 or 1
    bits.iter().for_each(|b| cs.anchor((one - b) * b));
    // Reconstruct value from bits
    let sum = (0..bit_range)
        .map(|i| 1 << i) // Power of 2
        .zip(bits)
        .map(|(coeff, b)| coeff * b) // 2^i * b
        .sum::<V<Fr>>();
    // Enforce v is sum
    cs.anchor(v - sum);
}

pub fn fold(cs: &mut ConstraintSystem<Fr>, u_r: U, u_i: U, r: Wire<Fr>) -> U {
    let xcc = cs.wire((cs.one() - r) * u_r.xcc + r * u_i.xcc);
    let xpc = cs.wire((cs.one() - r) * u_r.xpc + r * u_i.xpc);
    let aa = xcc + xpc;
    U { xcc, xpc }
}

pub fn aaa(cs: &mut ConstraintSystem<Fr>, a: Wire<Fr>, b: &Wire<Fr>, c: &Wire<Fr>) {
    let x = a - b;
}

pub struct U {
    xcc: Wire<Fr>,
    xpc: Wire<Fr>,
}
