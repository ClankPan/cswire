use ark_bn254::Fr;
use variables::{ConstraintSystem, V, Wire};

mod variables;

pub fn example(cs: &mut ConstraintSystem<Fr>, a: Wire<Fr>, b: Wire<Fr>, c: Wire<Fr>) -> Wire<Fr> {
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
pub fn decompose(cs: &mut ConstraintSystem<Fr>, v: Wire<Fr>) -> Vec<Wire<Fr>> {
    // let bits = v.raw().to_bits();
    // let bits = bits.into_iter().map(|b| cs.alloc(b)).collect();
    // bits.for_each(|b| cs.anchor((1-b)*b);
    // cs.anchor(bits.enumerate().map(|(i, b)| 1<<i * b)).sum() - v);
    // bits
    //
    // v.raw()の取り扱いをどうするか。
    //
    todo!()
}
pub fn fold(cs: &mut ConstraintSystem<Fr>, u_r: U, u_i: U, r: Wire<Fr>) -> U {
    let xcc = cs.wire((cs.one() - r) * u_r.xcc + r * u_i.xcc);
    let xpc = cs.wire((cs.one() - r) * u_r.xpc + r * u_i.xpc);
    U { xcc, xpc }
}

pub struct U {
    xcc: Wire<Fr>,
    xpc: Wire<Fr>,
}
