use std::{
    cell::RefCell,
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub},
    rc::Rc,
};

use ark_ff::Field;

#[derive(Clone, Copy, Debug)]
pub struct Idx(usize);

#[derive(Clone, Copy, Debug)]
pub struct Wire<F: Field> {
    exp: Option<Idx>,
    val: F,
}

#[derive(Clone, Debug)]
pub enum Exp<F> {
    Idx(Idx),
    Coe(F),                        // 係数
    Add(Box<Exp<F>>, Box<Exp<F>>), // 加算
    Sub(Box<Exp<F>>, Box<Exp<F>>), // 減算
    Mul(Box<Exp<F>>, Box<Exp<F>>), // 乗算
}

impl<F: Field> From<Idx> for Exp<F> {
    fn from(idx: Idx) -> Self {
        Exp::Idx(idx)
    }
}

#[derive(Clone, Debug)]
pub struct V<F: Field> {
    val: F,
    exp: Option<Exp<F>>,
}
#[derive(Clone)]
pub struct VV<F: Field> {
    val: F,
    exp: Option<Exp<F>>,
}

impl<F: Field> Wire<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}
impl<F: Field> V<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}

impl<F: Field> VV<F> {
    pub fn raw(&self) -> F {
        self.val
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Compile,
    Run,
}

pub struct R1CS;

#[derive(Clone, Debug)]
pub struct ConstraintSystem<F: Field> {
    wires: Vec<F>,
    exprs: Vec<(Option<Idx>, Exp<F>)>,
    mode: Mode,
}

impl<F: Field> ConstraintSystem<F> {
    pub fn new(mode: Mode) -> Self {
        Self {
            wires: vec![F::ONE],
            exprs: vec![],
            mode,
        }
    }
    pub fn new_ref(mode: Mode) -> ConstraintSystemRef<F> {
        ConstraintSystemRef::<F>::new(mode)
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }
    pub fn compile(&self) -> R1CS {
        match self.mode {
            Mode::Compile => todo!(),
            Mode::Run => todo!(),
        }
    }
    pub fn witnesses(&self) -> Vec<F> {
        self.wires.clone()
    }
    pub fn wire<W: Wirable<F>>(&mut self, w: W) -> Wire<F> {
        let vv = w.into_vv();
        let mut wire = self.alloc(vv.val);
        match (vv.exp, self.mode) {
            (None, Mode::Run) => wire.exp = None,
            (Some(_), Mode::Run) => wire.exp = None,
            (None, Mode::Compile) => panic!("Wire has to be set exp"),
            (Some(exp), Mode::Compile) => {
                self.exprs.push((Some(wire.exp.unwrap()), exp));
            }
        }
        wire
    }
    pub fn alloc<T>(&mut self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        let val = F::from(val);
        self.wires.push(val);
        Wire {
            exp: Some(Idx(self.wires.len())),
            val,
        }
    }
    pub fn one(&self) -> Wire<F> {
        Wire {
            val: self.wires[0],
            exp: Some(Idx(0)),
        }
    }
    pub fn anchor<W: Wirable<F>>(&mut self, w: W) {
        let vv = w.into_vv();
        match (vv.exp, self.mode) {
            (None, Mode::Run) => {}
            (Some(_), Mode::Run) => {}
            (None, Mode::Compile) => panic!("Wire has to be set exp"),
            (Some(exp), Mode::Compile) => {
                self.exprs.push((None, exp));
            }
        }
    }
}

#[derive(Clone)]
pub struct ConstraintSystemRef<F: Field>(Rc<RefCell<ConstraintSystem<F>>>);
impl<F: Field> ConstraintSystemRef<F> {
    pub fn new(mode: Mode) -> Self {
        Self(Rc::new(RefCell::new(ConstraintSystem::new(mode))))
    }
    pub fn set_mode(&self, mode: Mode) {
        self.0.borrow_mut().mode = mode
    }
    pub fn compile(&self) -> R1CS {
        self.0.borrow().compile()
    }
    pub fn witnesses(&self) -> Vec<F> {
        self.0.borrow().witnesses()
    }
    pub fn alloc<T>(&self, val: T) -> Wire<F>
    where
        F: From<T>,
    {
        self.0.borrow_mut().alloc(val)
    }

    pub fn wire<W: Wirable<F>>(&self, w: W) -> Wire<F> {
        self.0.borrow_mut().wire(w)
    }

    pub fn one(&self) -> Wire<F> {
        self.0.borrow().one()
    }

    pub fn anchor<W: Wirable<F>>(&self, w: W) {
        self.0.borrow_mut().anchor(w)
    }

    pub fn mode(&self) -> Mode {
        self.0.borrow().mode
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

/* From */

impl<F: Field> From<Wire<F>> for V<F> {
    fn from(Wire { exp, val }: Wire<F>) -> Self {
        let exp = exp.map(Exp::Idx);
        Self { exp, val }
    }
}

impl<F: Field> From<&Wire<F>> for V<F> {
    fn from(Wire { exp, val }: &Wire<F>) -> Self {
        let exp = exp.map(Exp::Idx);
        Self { exp, val: *val }
    }
}

impl<F: Field> From<V<F>> for VV<F> {
    fn from(v: V<F>) -> Self {
        VV {
            val: v.val,
            exp: v.exp,
        }
    }
}
impl<F: Field> From<&V<F>> for VV<F> {
    fn from(v: &V<F>) -> Self {
        VV {
            val: v.val,
            exp: v.exp.clone(),
        }
    }
}

impl<F: Field> From<F> for V<F> {
    fn from(val: F) -> Self {
        V {
            val,
            exp: Some(Exp::Coe(val)),
        }
    }
}

impl<F: Field> From<&F> for V<F> {
    fn from(val: &F) -> Self {
        V {
            val: *val,
            exp: Some(Exp::Coe(*val)),
        }
    }
}

macro_rules! impl_op {

    // lhs op rhs
    ($trait:ident, $method:ident, $lhs:ident, $rhs:ident, $output:ident) => {
        impl<F: Field> $trait<$rhs<F>> for $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: $rhs<F>) -> Self::Output {
                impl_op!(@inner $trait, self, rhs, $method, $output)
            }
        }
        impl_op!(@ref $trait, $method, $lhs, &$rhs, $output);
        impl_op!(@ref $trait, $method, &$lhs, $rhs, $output);
        impl_op!(@ref $trait, $method, &$lhs, &$rhs, $output);
    };

    // &lhs op rhs
    (@ref $trait:ident, $method:ident, &$lhs:ident, $rhs:ident, $output:ident) => {
        impl<'a, F: Field> $trait<$rhs<F>> for &'a $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: $rhs<F>) -> Self::Output {
                impl_op!(@inner $trait, self.clone(), rhs, $method, $output)
            }
        }
    };

    // lhs op &rhs
    (@ref $trait:ident, $method:ident, $lhs:ident, &$rhs:ident, $output:ident) => {
        impl<'a, F: Field> $trait<&'a $rhs<F>> for $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: &'a $rhs<F>) -> Self::Output {
                impl_op!(@inner $trait, self, rhs.clone(), $method, $output)
            }
        }
    };

    // &lhs op &rhs
    (@ref $trait:ident, $method:ident, &$lhs:ident, &$rhs:ident, $output:ident) => {
        impl<'a, 'b, F: Field> $trait<&'b $rhs<F>> for &'a $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: &'b $rhs<F>) -> Self::Output {
                impl_op!(@inner $trait, self.clone(), rhs.clone(), $method, $output)
            }
        }
    };

    // ty op rhs
    ($trait:ident, $method:ident, $lhs:ident, #$rhs:ident, $output:ident) => {
        impl<F: Field> $trait<$rhs> for $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: $rhs) -> Self::Output {
                let val: F = rhs.into();
                impl_op!(@inner $trait, self, V { val, exp: Some(Exp::Coe(val)) }, $method, $output)
            }
        }
        impl_op!(@ref $trait, $method, $lhs, &#$rhs, $output);
        impl_op!(@ref $trait, $method, &$lhs, #$rhs, $output);
        impl_op!(@ref $trait, $method, &$lhs, &#$rhs, $output);
    };

    // &ty op rhs
    (@ref $trait:ident, $method:ident, $lhs:ident, &#$rhs:ident, $output:ident) => {
        impl<F: Field> $trait<&$rhs> for $lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: &$rhs) -> Self::Output {
                let val: F = rhs.clone().into();
                impl_op!(@inner $trait, self, V { val, exp: Some(Exp::Coe(val)) }, $method, $output)
            }
        }
    };

    // ty op &rhs
    (@ref $trait:ident, $method:ident, &$lhs:ident, #$rhs:ident, $output:ident) => {
        impl<F: Field> $trait<$rhs> for &$lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: $rhs) -> Self::Output {
                let val: F = rhs.into();
                impl_op!(@inner $trait, self.clone(), V { val, exp: Some(Exp::Coe(val)) }, $method, $output)
            }
        }
    };

    // &ty op &rhs
    (@ref $trait:ident, $method:ident, &$lhs:ident, &#$rhs:ident, $output:ident) => {
        impl<F: Field> $trait<&$rhs> for &$lhs<F> {
            type Output = $output<F>;
            fn $method(self, rhs: &$rhs) -> Self::Output {
                let val: F = rhs.clone().into();
                impl_op!(@inner $trait, self.clone(), V { val, exp: Some(Exp::Coe(val)) }, $method, $output)
            }
        }
    };

    // lhs op ty
    (@ref $trait:ident, $method:ident, #$lhs:ident, $rhs:ident, $output:ident) => {
        impl<F: Field> $trait<$rhs<F>> for $lhs {
            type Output = $output<F>;
            fn $method(self, rhs: $rhs<F>) -> Self::Output {
                let val: F = self.into();
                impl_op!(@inner $trait, V { val, exp: Some(Exp::Coe(val)) }, rhs, $method, $output)
            }
        }
    };

    // 共通の内部処理
    (@inner $trait:ident, $lhs:expr, $rhs:expr, $method:ident, $output:ident) => {{
        let lhs = $lhs;
        let rhs = $rhs;
        let exp = match (lhs.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::$trait(Box::new(x.into()), Box::new(y.into()))),
            _ => None,
        };
        $output {
            val: lhs.val.$method(rhs.val),
            exp,
        }
    }};
}

impl_op!(Add, add, Wire, #bool, V);
impl_op!(Add, add, Wire, #u8, V);
impl_op!(Add, add, Wire, #u16, V);
impl_op!(Add, add, Wire, #u32, V);
impl_op!(Add, add, Wire, #u64, V);
impl_op!(Add, add, Wire, #u128, V);

impl_op!(Add, add, V, #bool, V);
impl_op!(Add, add, V, #u8, V);
impl_op!(Add, add, V, #u16, V);
impl_op!(Add, add, V, #u32, V);
impl_op!(Add, add, V, #u64, V);
impl_op!(Add, add, V, #u128, V);

impl_op!(Add, add, VV, #bool, VV);
impl_op!(Add, add, VV, #u8, VV);
impl_op!(Add, add, VV, #u16, VV);
impl_op!(Add, add, VV, #u32, VV);
impl_op!(Add, add, VV, #u64, VV);
impl_op!(Add, add, VV, #u128, VV);

impl_op!(Sub, sub, Wire, #bool, V);
impl_op!(Sub, sub, Wire, #u8, V);
impl_op!(Sub, sub, Wire, #u16, V);
impl_op!(Sub, sub, Wire, #u32, V);
impl_op!(Sub, sub, Wire, #u64, V);
impl_op!(Sub, sub, Wire, #u128, V);

impl_op!(Sub, sub, V, #bool, V);
impl_op!(Sub, sub, V, #u8, V);
impl_op!(Sub, sub, V, #u16, V);
impl_op!(Sub, sub, V, #u32, V);
impl_op!(Sub, sub, V, #u64, V);
impl_op!(Sub, sub, V, #u128, V);

impl_op!(Sub, sub, VV, #bool, VV);
impl_op!(Sub, sub, VV, #u8, VV);
impl_op!(Sub, sub, VV, #u16, VV);
impl_op!(Sub, sub, VV, #u32, VV);
impl_op!(Sub, sub, VV, #u64, VV);
impl_op!(Sub, sub, VV, #u128, VV);

impl_op!(Mul, mul, Wire, #bool, V);
impl_op!(Mul, mul, Wire, #u8, V);
impl_op!(Mul, mul, Wire, #u16, V);
impl_op!(Mul, mul, Wire, #u32, V);
impl_op!(Mul, mul, Wire, #u64, V);
impl_op!(Mul, mul, Wire, #u128, V);

impl_op!(Mul, mul, V, #bool, V);
impl_op!(Mul, mul, V, #u8, V);
impl_op!(Mul, mul, V, #u16, V);
impl_op!(Mul, mul, V, #u32, V);
impl_op!(Mul, mul, V, #u64, V);
impl_op!(Mul, mul, V, #u128, V);

impl_op!(Mul, mul, VV, #bool, VV);
impl_op!(Mul, mul, VV, #u8, VV);
impl_op!(Mul, mul, VV, #u16, VV);
impl_op!(Mul, mul, VV, #u32, VV);
impl_op!(Mul, mul, VV, #u64, VV);
impl_op!(Mul, mul, VV, #u128, VV);

impl_op!(Add, add, Wire, Wire, V);
impl_op!(Add, add, Wire, V, V);
impl_op!(Add, add, Wire, VV, VV);
impl_op!(Add, add, V, Wire, V);
impl_op!(Add, add, V, V, V);
impl_op!(Add, add, V, VV, VV);
impl_op!(Add, add, VV, Wire, VV);
impl_op!(Add, add, VV, V, VV);
impl_op!(Add, add, VV, VV, VV);

impl_op!(Sub, sub, Wire, Wire, V);
impl_op!(Sub, sub, Wire, V, V);
impl_op!(Sub, sub, Wire, VV, VV);
impl_op!(Sub, sub, V, Wire, V);
impl_op!(Sub, sub, V, V, V);
impl_op!(Sub, sub, V, VV, VV);
impl_op!(Sub, sub, VV, Wire, VV);
impl_op!(Sub, sub, VV, V, VV);
impl_op!(Sub, sub, VV, VV, VV);

impl_op!(Mul, mul, Wire, Wire, VV);
impl_op!(Mul, mul, Wire, V, VV);
impl_op!(Mul, mul, V, Wire, VV);
impl_op!(Mul, mul, V, V, VV);

impl<F: Field> Sum<Wire<F>> for V<F> {
    fn sum<I: Iterator<Item = Wire<F>>>(iter: I) -> Self {
        iter.map(|i| i.into())
            .reduce(|acc, x| acc + x)
            .expect("length is zero")
    }
}
impl<F: Field> Sum<V<F>> for V<F> {
    fn sum<I: Iterator<Item = V<F>>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).expect("length is zero")
    }
}
impl<F: Field> Sum<VV<F>> for VV<F> {
    fn sum<I: Iterator<Item = VV<F>>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x).expect("length is zero")
    }
}

impl<F: Field> AddAssign<Wire<F>> for V<F> {
    fn add_assign(&mut self, rhs: Wire<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<V<F>> for V<F> {
    fn add_assign(&mut self, rhs: V<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<&Wire<F>> for V<F> {
    fn add_assign(&mut self, rhs: &Wire<F>) {
        *self = &*self + rhs;
    }
}
impl<F: Field> AddAssign<&V<F>> for V<F> {
    fn add_assign(&mut self, rhs: &V<F>) {
        *self = &*self + rhs;
    }
}
