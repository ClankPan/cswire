use ark_ff::UniformRand;
use ark_std::test_rng;

use crate::variables::*;

use super::*;

#[test]
fn test_add_wire_wire() {
    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);
    let mut rng = test_rng();
    let a = Fr::rand(&mut rng);
    let b = Fr::rand(&mut rng);
    let c = a + b;

    let a = cs.alloc(a);
    let b = cs.alloc(b);

    let x = a + b;
    assert!(c==x.raw());

    let x = &a + b;
    assert!(c==x.raw());
    
    let x = a + &b;
    assert!(c==x.raw());

    let x = &a + &b;
    assert!(c==x.raw());
}

#[test]
fn test_sub_wire_wire() {
    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);
    let mut rng = test_rng();
    let a = Fr::rand(&mut rng);
    let b = Fr::rand(&mut rng);
    let c = a - b;

    let a = cs.alloc(a);
    let b = cs.alloc(b);

    let x = a - b;
    assert!(c==x.raw());

    let x = &a - b;
    assert!(c==x.raw());
    
    let x = a - &b;
    assert!(c==x.raw());

    let x = &a - &b;
    assert!(c==x.raw());
}

#[test]
fn test_mul_wire_wire() {
    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);
    let mut rng = test_rng();
    let a = Fr::rand(&mut rng);
    let b = Fr::rand(&mut rng);
    let c = a * b;

    let a = cs.alloc(a);
    let b = cs.alloc(b);

    let x = a * b;
    assert!(c==x.raw());

    let x = &a * b;
    assert!(c==x.raw());
    
    let x = a * &b;
    assert!(c==x.raw());

    let x = &a * &b;
    assert!(c==x.raw());
}

#[test]
fn test_mul_wire_v() {
    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);
    let mut rng = test_rng();
    let a = Fr::rand(&mut rng);
    let b = Fr::rand(&mut rng);
    let c = a * (b+b);

    let a = cs.alloc(a);
    let b = cs.alloc(b);
    let b = b+b;

    let x = a * b.clone();
    assert!(c==x.raw());

    let x = &a * b.clone();
    assert!(c==x.raw());
    
    let x = a * &b;
    assert!(c==x.raw());

    let x = &a * &b;
    assert!(c==x.raw());
}

#[test]
pub fn test_sum_wire() {

    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);

    let sum: V<Fr> = (0..10).map(|i| cs.alloc(i)).sum();
    let aaa: Fr = (0..10).map(|i| {
        Fr::from(i)
    }).sum();
    assert!(sum.raw() == aaa)
}
