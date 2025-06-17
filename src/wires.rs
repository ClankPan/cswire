use std::ops::{Add, AddAssign, Mul, Sub};

use ark_ff::Field;

#[derive(Clone, Copy, Debug)]
pub struct Idx(usize);

#[derive(Clone, Copy, Debug)]
pub struct Wire<F: Field> {
    exp: Option<Idx>,
    val: F,
}

#[derive(Clone, Debug)]
pub struct Linear<F: Field>(Vec<(bool, F, Idx)>);
#[derive(Clone, Debug)]
pub struct Quadratic<F: Field>((Linear<F>, Linear<F>), Linear<F>);
#[derive(Clone, Debug)]
pub struct V<F: Field> {
    val: F,
    exp: Option<Linear<F>>,
}
impl<F: Field> Linear<F> {
    #[allow(non_snake_case)]
    pub fn add_V_V(self, y: Self) -> Self {
        Self([self.0, y.0].concat())
    }
    #[allow(non_snake_case)]
    pub fn sub_V_V(self, y: Self) -> Self {
        Self(
            [
                self.0,
                y.0.into_iter().map(|(s, v, i)| (!s, v, i)).collect(),
            ]
            .concat(),
        )
    }
    #[allow(non_snake_case)]
    pub fn mul_V_V(self, y: Self) -> Quadratic<F> {
        Quadratic((self, y), Self(vec![]))
    }

    #[allow(non_snake_case)]
    pub fn add_V_VV(self, y: Quadratic<F>) -> Quadratic<F> {
        Quadratic(y.0, Linear::add_V_V(y.1, self))
    }
    #[allow(non_snake_case)]
    pub fn sub_V_VV(self, y: Quadratic<F>) -> Quadratic<F> {
        let y = y.negate();
        Quadratic(y.0, Linear::add_V_V(y.1, self))
    }

    pub fn negate(self) -> Self {
        Self(self.0.into_iter().map(|(s, v, i)| (!s, v, i)).collect())
    }
}

impl<F: Field> Quadratic<F> {
    #[allow(non_snake_case)]
    pub fn add_VV_V(self, y: Linear<F>) -> Self {
        Self(self.0, Linear::add_V_V(self.1, y))
    }
    #[allow(non_snake_case)]
    pub fn sub_VV_V(self, y: Linear<F>) -> Self {
        Self(self.0, Linear::sub_V_V(self.1, y))
    }
    pub fn negate(self) -> Self {
        let a = self.0.0.negate();
        let b = self.0.1;
        let c = self.1.negate();

        Self((a, b), c)
    }
}
#[derive(Clone)]
pub struct VV<F: Field> {
    val: F,
    exp: Option<Quadratic<F>>,
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

impl<F: Field> Add<V<F>> for V<F> {
    type Output = V<F>;

    fn add(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(x.add_V_V(y)),
            _ => None,
        };
        V {
            exp,
            val: self.val + rhs.val,
        }
    }
}
impl<F: Field> Sub<V<F>> for V<F> {
    type Output = V<F>;

    fn sub(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(x.sub_V_V(y)),
            _ => None,
        };
        V {
            exp,
            val: self.val - rhs.val,
        }
    }
}

impl<F: Field> Mul<V<F>> for V<F> {
    type Output = VV<F>;

    fn mul(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(x.mul_V_V(y)),
            _ => None,
        };
        VV {
            exp,
            val: self.val * rhs.val,
        }
    }
}

impl<F: Field> Add<V<F>> for VV<F> {
    type Output = VV<F>;

    fn add(self, rhs: V<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(x.sub_VV_V(y)),
            _ => None,
        };
        VV {
            exp,
            val: self.val + rhs.val,
        }
    }
}

impl<F: Field> Add<VV<F>> for V<F> {
    type Output = VV<F>;

    fn add(self, rhs: VV<F>) -> Self::Output {
        let exp = match (self.exp, rhs.exp) {
            (Some(x), Some(y)) => Some(x.sub_V_VV(y)),
            _ => None,
        };
        VV {
            exp,
            val: self.val + rhs.val,
        }
    }
}
