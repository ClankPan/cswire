use std::{
    iter::Sum,
    marker::PhantomData,
    ops::{Add, Mul, Sub},
};

use ark_ff::Field;

#[derive(Clone, Copy)]
pub struct Wire<F: Field> {
    idx: Option<usize>,
    val: F,
}

#[derive(Clone, Debug)]
enum Exp<F> {
    Idx(usize),
    Coe(F),                        // 係数
    Add(Box<Exp<F>>, Box<Exp<F>>), // 加算
    Sub(Box<Exp<F>>, Box<Exp<F>>), // 減算
    Mul(Box<Exp<F>>, Box<Exp<F>>), // 乗算
}

pub struct V<F: Field> {
    val: F,
    exp: Option<Exp<F>>,
}
pub struct VV<F: Field> {
    val: F,
    exp: Option<Exp<F>>,
}

impl<F: Field> VV<F> {
    pub fn eval(&self, values: &[u64]) -> u64 {
        todo!();
    }
}

pub struct ConstraintSystem<F: Field> {
    wires: Vec<F>,
}

impl<F: Field> ConstraintSystem<F> {
    pub fn new() -> Self {
        Self {
            wires: vec![F::ONE],
        }
    }
    pub fn wire<W: Wirable<F>>(&mut self, w: W) -> Wire<F> {
        let vv = w.into_vv();
        todo!();
    }
    pub fn alloc<T>(&mut self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        let val = F::from(val);
        self.wires.push(val);
        Wire {
            idx: Some(self.wires.len()),
            val,
        }
    }
    pub fn one(&self) -> Wire<F> {
        todo!()
    }
    pub fn anchor<W: Wirable<F>>(&mut self, w: W) {
        todo!()
    }
}

// cs.wireがVとVVを同時に扱えるようにする。
pub trait Wirable<F: Field> {
    fn into_vv(self) -> VV<F>;
}

impl<F: Field> Wirable<F> for VV<F> {
    fn into_vv(self) -> VV<F> {
        self
    }
}

impl<F: Field> Wirable<F> for V<F> {
    fn into_vv(self) -> VV<F> {
        VV::from(self)
    }
}

/* 演算定義 */

// Wire + Wire = V
impl<F: Field> Add for Wire<F> {
    type Output = V<F>;

    fn add(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.idx, rhs.idx) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(Exp::Idx(x)), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        V {
            val: self.val + rhs.val,
            exp,
        }
    }
}

// V + Wire = V
impl<F: Field> Add<Wire<F>> for V<F> {
    type Output = V<F>;
    fn add(self, rhs: Wire<F>) -> Self::Output {
        V {
            val: self.val + rhs.1,
            exp: Exp::Add(Box::new(self.exp), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// Wire + V = V
impl<F: Field> Add<V<F>> for Wire<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        V {
            val: self.1 + rhs.val,
            exp: Exp::Add(Box::new(Exp::Val(self.0)), Box::new(rhs.exp)),
        }
    }
}

// V + V = V
impl<F: Field> Add for V<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        V {
            val: self.val + rhs.val,
            exp: Exp::Add(Box::new(self.exp), Box::new(rhs.exp)),
        }
    }
}

// Wire * Wire = VV
impl<F: Field> Mul for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        VV {
            val: self.1 * rhs.1,
            exp: Exp::Mul(Box::new(Exp::Val(self.0)), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// V * Wire = VV
impl<F: Field> Mul<Wire<F>> for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        VV {
            val: self.val * rhs.1,
            exp: Exp::Mul(Box::new(self.exp), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// Wire * V = VV
impl<F: Field> Mul<V<F>> for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        VV {
            val: self.1 * rhs.val,
            exp: Exp::Mul(Box::new(Exp::Val(self.0)), Box::new(rhs.exp)),
        }
    }
}

// V * V = VV
impl<F: Field> Mul for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        VV {
            val: self.val * rhs.val,
            exp: Exp::Mul(Box::new(self.exp), Box::new(rhs.exp)),
        }
    }
}

// Wire + VV = VV
impl<F: Field> Add<VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        VV {
            val: self.1 + rhs.val,
            exp: Exp::Add(Box::new(Exp::Val(self.0)), Box::new(rhs.exp)),
        }
    }
}

// VV + Wire = VV
impl<F: Field> Add<Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: Wire<F>) -> Self::Output {
        VV {
            val: self.val + rhs.1,
            exp: Exp::Add(Box::new(self.exp), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// VV + VV = VV
impl<F: Field> Add for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        VV {
            val: self.val + rhs.val,
            exp: Exp::Add(Box::new(self.exp), Box::new(rhs.exp)),
        }
    }
}

/* Sub */

// Wire - Wire = V
impl<F: Field> Sub for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        V {
            val: self.1 - rhs.1,
            exp: Exp::Sub(Box::new(Exp::Val(self.0)), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// V - Wire = V
impl<F: Field> Sub<Wire<F>> for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        V {
            val: self.val - rhs.1,
            exp: Exp::Sub(Box::new(self.exp), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// Wire - V = V
impl<F: Field> Sub<V<F>> for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        V {
            val: self.1 - rhs.val,
            exp: Exp::Sub(Box::new(Exp::Val(self.0)), Box::new(rhs.exp)),
        }
    }
}

// V - V = V
impl<F: Field> Sub for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        V {
            val: self.val - rhs.val,
            exp: Exp::Sub(Box::new(self.exp), Box::new(rhs.exp)),
        }
    }
}

// Wire - VV = VV
impl<F: Field> Sub<VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        VV {
            val: self.1 - rhs.val,
            exp: Exp::Sub(Box::new(Exp::Val(self.0)), Box::new(rhs.exp)),
        }
    }
}

// VV - Wire = VV
impl<F: Field> Sub<Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        VV {
            val: self.val - rhs.1,
            exp: Exp::Sub(Box::new(self.exp), Box::new(Exp::Val(rhs.0))),
        }
    }
}

// VV + VV = VV
impl<F: Field> Sub for VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        VV {
            val: self.val - rhs.val,
            exp: Exp::Sub(Box::new(self.exp), Box::new(rhs.exp)),
        }
    }
}

/* Sum */

impl<F: Field> Sum for V<F> {
    fn sum<I: Iterator<Item = Self>>(_iter: I) -> Self {
        todo!()
    }
}

impl<F: Field> Sum for VV<F> {
    fn sum<I: Iterator<Item = Self>>(_iter: I) -> Self {
        todo!()
    }
}

impl<F: Field> Sum<Wire<F>> for V<F> {
    fn sum<I: Iterator<Item = Wire<F>>>(_iter: I) -> Self {
        todo!()
    }
}

/* 定数倍 */

// Wire * usize = V
impl<F: Field> Mul<usize> for Wire<F> {
    type Output = V<F>;
    fn mul(self, rhs: usize) -> Self::Output {
        let coef = F::from(rhs as u64);
        V {
            val: coef * self.1,
            exp: Exp::Mul(Box::new(Exp::Coe(coef)), Box::new(Exp::Val(self.0))),
        }
    }
}

// usize * Wire = V
impl<F: Field> Mul<Wire<F>> for usize {
    type Output = V<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        let coef = F::from(self as u64);
        V {
            val: coef * rhs.1,
            exp: Exp::Mul(Box::new(Exp::Coe(coef)), Box::new(Exp::Val(rhs.0))),
        }
    }
}

/* From */

impl<F: Field> From<V<F>> for VV<F> {
    fn from(_v: V<F>) -> Self {
        todo!()
    }
}
