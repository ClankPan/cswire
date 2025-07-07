use std::marker::PhantomData;

use crate::{Lc, Lin, Qua};

use ark_bn254::Fr;

macro_rules! vals {
    ($list:expr) => {
        $list.iter().map(|v| v).collect::<Vec<usize>>()
    };
}

fn lin(val: u64) -> Lin<'static, Fr> {
    Lin {
        value: Fr::from(val),
        lc: Lc::new(val as usize, true),
        _life: PhantomData,
    }
}
fn qua(val: u64) -> Qua<'static, Fr> {
    let one = Fr::from(val);
    Qua {
        value: one,
        qc: (one, Lc::default(), Lc::default()),
        lc: Lc::default(),
        _life: PhantomData,
    }
}

#[test]
fn lin_add_sub() {
    let a = lin(5);
    let b = lin(3);
    assert_eq!((&a + &b).value, Fr::from(8));
    assert_eq!((&a - &b).value, Fr::from(2));

    // let c = a + b;
    // assert!(vals!(c.lc.0) == vec![5, 3]);
}

#[test]
fn lin_mul_qua_symmetry() {
    let a = lin(7);
    let b = lin(2);
    let ab = a.clone() * b.clone();
    let ba = b * a;
    assert_eq!(ab.value, Fr::from(14));
    assert_eq!(ba.value, ab.value);
    // qc term should be symmetric
    assert_eq!(ab.qc.0, ba.qc.0);
    // assert!(vals!(ab.qc.1.list) == vec![7]);
    // assert!(vals!(ab.qc.2.list) == vec![2]);
    //
    // assert!(vals!(ba.qc.1.list) == vec![2]);
    // assert!(vals!(ba.qc.2.list) == vec![7]);
}

#[test]
fn scalar_ops() {
    let x = lin(11);
    assert_eq!((&x * 3u64).value, Fr::from(33));
    assert_eq!((3u64 * x).value, Fr::from(33));

    let mut q = qua(4);
    q = q * 5;
    assert_eq!(q.value, Fr::from(20));
}

#[test]
fn references_work() {
    let a = lin(9);
    let b = lin(4);
    assert_eq!((&a + &b).value, Fr::from(13u64));
    assert_eq!((&a * &b).value, Fr::from(36u64));

    let q = qua(6);
    let r = &q + &a;
    assert_eq!(r.value, Fr::from(15u64));
}
