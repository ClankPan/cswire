use ark_bn254::Fr;

use crate::CSWire;

#[test]
pub fn test_fibonacci() {
    let cs = CSWire::<Fr>::default();

    let i = cs.alloc(1);
    let mut a = i.clone();
    for _ in 0..2 {
        a = cs.wire(&a + &a);
    }

    let (_witness, asts) = cs.finish(&[a]);
    let r1cs = asts.compile();

    println!("{}", r1cs);
}
