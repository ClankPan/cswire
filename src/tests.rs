use ark_bn254::Fr;

use crate::CSWire;

#[test]
pub fn test_fibonacci() {
    let cs = CSWire::<Fr>::default();
    
    let mut a = cs.alloc(1);
    for _ in 0..3 {
        a = &a + &a;
    }

    cs.finish();
}
