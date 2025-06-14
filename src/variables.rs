use std::{
    cell::RefCell,
    iter::Sum,
    ops::{Add, Mul, Sub},
    rc::Rc,
};

use ark_ff::Field;

#[derive(Clone, Copy)]
pub struct Wire<F: Field> {
    exp: Option<usize>,
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

#[derive(Clone)]
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

#[derive(Clone, Copy)]
pub enum Mode {
    Compile,
    Run,
}
#[derive(Clone)]
pub struct ConstraintSystem<F: Field> {
    wires: Vec<F>,
    exprs: Vec<(Option<usize>, Exp<F>)>,
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
    pub fn compile(&self) {
        match self.mode {
            Mode::Compile => todo!(),
            Mode::Run => todo!(),
        }
    }
    pub fn witnesses(&self) -> Vec<F> {
        todo!()
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
            exp: Some(self.wires.len()),
            val,
        }
    }
    pub fn one(&self) -> Wire<F> {
        Wire {
            val: self.wires[0],
            exp: Some(0),
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
    pub fn compile(&self) {
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

/* 演算定義 */

// Wire + Wire = V
impl<F: Field> Add for Wire<F> {
    type Output = V<F>;

    fn add(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(Exp::Idx(x)), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        V {
            val: self.val + rhs.val,
            exp,
        }
    }
}

// &Wire + Wire = V
impl<F: Field> Add<Wire<F>> for &Wire<F> {
    type Output = V<F>;

    fn add(self, rhs: Wire<F>) -> Self::Output {
        *self + rhs
    }
}
// Wire + &Wire = V
impl<F: Field> Add<&Wire<F>> for Wire<F> {
    type Output = V<F>;

    fn add(self, rhs: &Wire<F>) -> Self::Output {
        self + *rhs
    }
}
// &Wire + &Wire = V
impl<F: Field> Add<&Wire<F>> for &Wire<F> {
    type Output = V<F>;

    fn add(self, rhs: &Wire<F>) -> Self::Output {
        *self + *rhs
    }
}

// V + Wire = V
impl<F: Field> Add<Wire<F>> for V<F> {
    type Output = V<F>;
    fn add(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(x), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        V {
            val: self.val + rhs.val,
            exp,
        }
    }
}

// &V + Wire = V
impl<F: Field> Add<Wire<F>> for &V<F> {
    type Output = V<F>;

    fn add(self, rhs: Wire<F>) -> Self::Output {
        let cloned: V<F> = self.clone();
        cloned + rhs
    }
}

// &V + Wire = V
impl<F: Field> Add<&Wire<F>> for V<F> {
    type Output = V<F>;

    fn add(self, rhs: &Wire<F>) -> Self::Output {
        self + *rhs
    }
}

// &V + &Wire = V
impl<F: Field> Add<&Wire<F>> for &V<F> {
    type Output = V<F>;

    fn add(self, rhs: &Wire<F>) -> Self::Output {
        self + *rhs
    }
}

// Wire + V = V
impl<F: Field> Add<V<F>> for Wire<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(Exp::Idx(x)), Box::new(y))),
            _ => None,
        };
        V {
            val: self.val + rhs.val,
            exp,
        }
    }
}

// &Wire + V = V
impl<F: Field> Add<V<F>> for &Wire<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        *self + rhs
    }
}

// Wire + &V = V
impl<F: Field> Add<&V<F>> for Wire<F> {
    type Output = V<F>;
    fn add(self, rhs: &V<F>) -> Self::Output {
        let cloned: V<F> = rhs.clone();
        self + cloned
    }
}

// &Wire + &V = V
impl<F: Field> Add<&V<F>> for &Wire<F> {
    type Output = V<F>;
    fn add(self, rhs: &V<F>) -> Self::Output {
        *self + rhs
    }
}

// V + V = V
impl<F: Field> Add for V<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(x), Box::new(y))),
            _ => None,
        };
        V {
            val: self.val + rhs.val,
            exp,
        }
    }
}

// &V + &V = V
impl<F: Field> Add for &V<F> {
    type Output = V<F>;
    fn add(self, rhs: &V<F>) -> Self::Output {
        let cloned_self: V<F> = self.clone();
        let cloned_rhs: V<F> = rhs.clone();
        cloned_self + cloned_rhs
    }
}
// &V + V = V
impl<F: Field> Add<V<F>> for &V<F> {
    type Output = V<F>;
    fn add(self, rhs: V<F>) -> Self::Output {
        self + &rhs
    }
}
// V + &V = V
impl<F: Field> Add<&V<F>> for V<F> {
    type Output = V<F>;
    fn add(self, rhs: &V<F>) -> Self::Output {
        &self + rhs
    }
}

// Wire * Wire = VV
impl<F: Field> Mul for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Mul(Box::new(Exp::Idx(x)), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        VV {
            val: self.val * rhs.val,
            exp,
        }
    }
}

// &Wire * &Wire = VV
impl<F: Field> Mul for &Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &Wire<F>) -> Self::Output {
        *self * *rhs
    }
}
// &Wire * Wire = VV
impl<F: Field> Mul<Wire<F>> for &Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        *self * rhs
    }
}
// Wire * &Wire = VV
impl<F: Field> Mul<&Wire<F>> for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &Wire<F>) -> Self::Output {
        self * *rhs
    }
}

// V * Wire = VV
impl<F: Field> Mul<Wire<F>> for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Mul(Box::new(x), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        VV {
            val: self.val * rhs.val,
            exp,
        }
    }
}
// &V * Wire = VV
impl<F: Field> Mul<Wire<F>> for &V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        let cloned: V<F> = self.clone();
        cloned * rhs
    }
}
// V * &Wire = VV
impl<F: Field> Mul<&Wire<F>> for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &Wire<F>) -> Self::Output {
        self * *rhs
    }
}

// &V * &Wire = VV
impl<F: Field> Mul<&Wire<F>> for &V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &Wire<F>) -> Self::Output {
        self * *rhs
    }
}

// Wire * V = VV
impl<F: Field> Mul<V<F>> for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Mul(Box::new(Exp::Idx(x)), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val * rhs.val,
            exp,
        }
    }
}
// &Wire * V = VV
impl<F: Field> Mul<V<F>> for &Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        *self * rhs
    }
}
// Wire * &V = VV
impl<F: Field> Mul<&V<F>> for Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &V<F>) -> Self::Output {
        let cloned: V<F> = rhs.clone();
        self * cloned
    }
}
// &Wire * &V = VV
impl<F: Field> Mul<&V<F>> for &Wire<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &V<F>) -> Self::Output {
        *self * rhs
    }
}

// V * V = VV
impl<F: Field> Mul for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Mul(Box::new(x), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val * rhs.val,
            exp,
        }
    }
}

// &V * &V = VV
impl<F: Field> Mul for &V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &V<F>) -> Self::Output {
        let cloned_self: V<F> = self.clone();
        let cloned_rhs: V<F> = rhs.clone();
        cloned_self * cloned_rhs
    }
}
// &V * V = VV
impl<F: Field> Mul<&V<F>> for V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: &V<F>) -> Self::Output {
        &self * rhs
    }
}
// V * &V = VV
impl<F: Field> Mul<V<F>> for &V<F> {
    type Output = VV<F>;
    fn mul(self, rhs: V<F>) -> Self::Output {
        self * &rhs
    }
}

// Wire + VV = VV
impl<F: Field> Add<VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(Exp::Idx(x)), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val + rhs.val,
            exp,
        }
    }
}
// &Wire + VV = VV
impl<F: Field> Add<VV<F>> for &Wire<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        *self + rhs
    }
}

// Wire + &VV = VV
impl<F: Field> Add<&VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn add(self, rhs: &VV<F>) -> Self::Output {
        let cloned: VV<F> = rhs.clone();
        self + cloned
    }
}

// &Wire + &VV = VV
impl<F: Field> Add<&VV<F>> for &Wire<F> {
    type Output = VV<F>;
    fn add(self, rhs: &VV<F>) -> Self::Output {
        *self + rhs
    }
}

// VV + Wire = VV
impl<F: Field> Add<Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(x), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        VV {
            val: self.val + rhs.val,
            exp,
        }
    }
}
// &VV + Wire = VV
impl<F: Field> Add<Wire<F>> for &VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: Wire<F>) -> Self::Output {
        let cloned: VV<F> = self.clone();
        cloned + rhs
    }
}
// VV + &Wire = VV
impl<F: Field> Add<&Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: &Wire<F>) -> Self::Output {
        self + *rhs
    }
}
// &VV + &Wire = VV
impl<F: Field> Add<&Wire<F>> for &VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: &Wire<F>) -> Self::Output {
        self + *rhs
    }
}

// VV + VV = VV
impl<F: Field> Add for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Add(Box::new(x), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val + rhs.val,
            exp,
        }
    }
}
// &VV + &VV = VV
impl<F: Field> Add for &VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: &VV<F>) -> Self::Output {
        let cloned_self = self.clone();
        let cloned_rhs = rhs.clone();
        cloned_self + cloned_rhs
    }
}
// &VV + VV = VV
impl<F: Field> Add<VV<F>> for &VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: VV<F>) -> Self::Output {
        self + &rhs
    }
}

// VV + &VV = VV
impl<F: Field> Add<&VV<F>> for VV<F> {
    type Output = VV<F>;
    fn add(self, rhs: &VV<F>) -> Self::Output {
        &self + rhs
    }
}

/* Sub */

// Wire - Wire = V
impl<F: Field> Sub for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(Exp::Idx(x)), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        V {
            val: self.val - rhs.val,
            exp,
        }
    }
}

// &Wire - Wire = V
impl<F: Field> Sub for &Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        *self - *rhs
    }
}
// Wire - &Wire = V
impl<F: Field> Sub<&Wire<F>> for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        self - *rhs
    }
} // &Wire - Wire = V
impl<F: Field> Sub<Wire<F>> for &Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        *self - rhs
    }
}

// V - Wire = V
impl<F: Field> Sub<Wire<F>> for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(x), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        V {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &V - Wire = V
impl<F: Field> Sub<Wire<F>> for &V<F> {
    type Output = V<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        let cloned = self.clone();
        cloned - rhs
    }
}
// V - &Wire = V
impl<F: Field> Sub<&Wire<F>> for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        self - *rhs
    }
}
// &V - &Wire = V
impl<F: Field> Sub<&Wire<F>> for &V<F> {
    type Output = V<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        self - *rhs
    }
}

// Wire - V = V
impl<F: Field> Sub<V<F>> for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(Exp::Idx(x)), Box::new(y))),
            _ => None,
        };
        V {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &Wire - V = V
impl<F: Field> Sub<V<F>> for &Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        *self - rhs
    }
}
// Wire - &V = V
impl<F: Field> Sub<&V<F>> for Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: &V<F>) -> Self::Output {
        let cloned: V<F> = rhs.clone();
        self - cloned
    }
}
// &Wire - &V = V
impl<F: Field> Sub<&V<F>> for &Wire<F> {
    type Output = V<F>;
    fn sub(self, rhs: &V<F>) -> Self::Output {
        *self - rhs
    }
}

// V - V = V
impl<F: Field> Sub for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(x), Box::new(y))),
            _ => None,
        };
        V {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &V - &V = V
impl<F: Field> Sub for &V<F> {
    type Output = V<F>;
    fn sub(self, rhs: &V<F>) -> Self::Output {
        let cloned_self: V<F> = self.clone();
        let cloned_rhs: V<F> = rhs.clone();
        cloned_self - cloned_rhs
    }
}
// &V - V = V
impl<F: Field> Sub<V<F>> for &V<F> {
    type Output = V<F>;
    fn sub(self, rhs: V<F>) -> Self::Output {
        self - &rhs
    }
}
// V - &V = V
impl<F: Field> Sub<&V<F>> for V<F> {
    type Output = V<F>;
    fn sub(self, rhs: &V<F>) -> Self::Output {
        &self - rhs
    }
}

// Wire - VV = VV
impl<F: Field> Sub<VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(Exp::Idx(x)), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &Wire - VV = VV
impl<F: Field> Sub<VV<F>> for &Wire<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        *self - rhs
    }
}
// Wire - &VV = VV
impl<F: Field> Sub<&VV<F>> for Wire<F> {
    type Output = VV<F>;
    fn sub(self, rhs: &VV<F>) -> Self::Output {
        let cloned: VV<F> = rhs.clone();
        self - cloned
    }
}
// &Wire - &VV = VV
impl<F: Field> Sub<&VV<F>> for &Wire<F> {
    type Output = VV<F>;
    fn sub(self, rhs: &VV<F>) -> Self::Output {
        *self - rhs
    }
}

// VV - Wire = VV
impl<F: Field> Sub<Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(x), Box::new(Exp::Idx(y)))),
            _ => None,
        };
        VV {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &VV - Wire = VV
impl<F: Field> Sub<Wire<F>> for &VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: Wire<F>) -> Self::Output {
        let cloned: VV<F> = self.clone();
        cloned - rhs
    }
}

// VV - &Wire = VV
impl<F: Field> Sub<&Wire<F>> for VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        self - *rhs
    }
}
// &VV - &Wire = VV
impl<F: Field> Sub<&Wire<F>> for &VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: &Wire<F>) -> Self::Output {
        self - *rhs
    }
}

// VV - VV = VV
impl<F: Field> Sub for VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(Exp::Sub(Box::new(x), Box::new(y))),
            _ => None,
        };
        VV {
            val: self.val - rhs.val,
            exp,
        }
    }
}
// &VV - &VV = VV
impl<F: Field> Sub for &VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: &VV<F>) -> Self::Output {
        let cloned_self = self.clone();
        let cloned_rsh = rhs.clone();
        cloned_self - cloned_rsh
    }
}
// &VV - VV = VV
impl<F: Field> Sub<VV<F>> for &VV<F> {
    type Output = VV<F>;
    fn sub(self, rhs: VV<F>) -> Self::Output {
        self - &rhs
    }
}

/* Sum */

impl<F: Field> Sum for V<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x)
            .expect("Iterator must have at least one element")
    }
}

impl<F: Field> Sum for VV<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x)
            .expect("Iterator must have at least one element")
    }
}

impl<F: Field> Sum<Wire<F>> for V<F> {
    fn sum<I: Iterator<Item = Wire<F>>>(iter: I) -> Self {

        iter.map(V::from)
            .reduce(|acc, x| acc + x)
            .expect("Iterator must have at least one element")

    }
}

/* 定数倍 */

// Wire * usize = V
impl<F: Field> Mul<usize> for Wire<F> {
    type Output = V<F>;
    fn mul(self, rhs: usize) -> Self::Output {
        let coef = F::from(rhs as u64);
        let exp = self
            .exp
            .map(|x| Exp::Mul(Box::new(Exp::Coe(coef)), Box::new(Exp::Idx(x))));
        V {
            val: coef * self.val,
            exp,
        }
    }
}

// usize * Wire = V
impl<F: Field> Mul<Wire<F>> for usize {
    type Output = V<F>;
    fn mul(self, rhs: Wire<F>) -> Self::Output {
        let coef = F::from(self as u64);
        let exp = rhs
            .exp
            .map(|x| Exp::Mul(Box::new(Exp::Coe(coef)), Box::new(Exp::Idx(x))));
        V {
            val: coef * rhs.val,
            exp,
        }
    }
}

/* From */

impl<F: Field> From<Wire<F>> for V<F> {
    fn from(Wire { exp, val }: Wire<F>) -> Self {
        let exp = if let Some(idx) = exp {
            Some(Exp::Idx(idx))
        } else {
            None
        };
        Self { exp, val }
    }
}

impl<F: Field> From<V<F>> for VV<F> {
    fn from(_v: V<F>) -> Self {
        todo!()
    }
}
