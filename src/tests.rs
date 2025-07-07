use ark_bn254::Fr;

use crate::CSWire;

#[test]
pub fn test_fibonacci() {
    let cs = CSWire::<Fr>::default();

    let i = cs.alloc(1);
    // let mut a = i.clone();
    let mut b = i.clone();
    // let mut b = i.clone() + cs.one();
    b = cs.wire((&b * 3 * &b - &b) * 5);
    // let a = cs.wire(&a - &a);

    // for _ in 0..1 {
    //     // a = cs.wire(&a + &a);
    //     b = cs.wire(&b * &b - &b);
    // }

    // let (witness, asts) = cs.finish(&[a]);
    let (x, w, r1cs) = cs.finish(&[b]);
    let r1cs = r1cs.unwrap();

    println!("{}", r1cs);
    println!("{:?}", w);
    println!("is_satisfied {}", r1cs.is_satisfied(&x, &w));
}
