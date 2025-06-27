use ark_crypto_primitives::sponge::constraints;
use ark_ff::Field;
use core::fmt;
use std::{
    collections::HashMap, fmt::{Display, Formatter}, ops::{Add, Mul, Sub}, rc::Rc
};

#[derive(Clone)]
pub enum Expr<F> {
    Coeff(F),
    Idx(usize),
    Add(Rc<Expr<F>>, Rc<Expr<F>>),
    Sub(Rc<Expr<F>>, Rc<Expr<F>>),
    Mul(Rc<Expr<F>>, Rc<Expr<F>>),
}

// indexからexprを作る。
impl<F: Field> From<usize> for Expr<F> {
    fn from(idx: usize) -> Self {
        Expr::Idx(idx)
    }
}

impl<F: Field> Expr<F> {
    pub fn coefficient<T>(value: T) -> Self
    where
        F: From<T>,
    {
        Expr::Coeff(F::from(value))
    }
}

impl<F: Field> Add for Expr<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Expr::Add(Rc::new(self), Rc::new(rhs))
    }
}

/// 引き算と掛け算も同様
impl<F: Field> Sub for Expr<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Expr::Sub(Rc::new(self), Rc::new(rhs))
    }
}

impl<F: Field> Mul for Expr<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Expr::Mul(Rc::new(self), Rc::new(rhs))
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Term {
    Coeff,
    Linear(usize),
    Quadratic(usize, usize),
}

pub fn parse<F: Field>(expr: &Expr<F>) -> HashMap<Term, F> {
    match expr {
        Expr::Coeff(coeff) => HashMap::from([(Term::Coeff, *coeff)]),
        Expr::Idx(i) => HashMap::from([(Term::Linear(*i), F::ONE)]),
        Expr::Add(left, right) => {
            let (mut left, right) = (parse(left), parse(right));
            for (term, coeff) in right.into_iter() {
                left.entry(term)
                    .and_modify(|existing_coeff| {
                        *existing_coeff += coeff;
                    })
                    .or_insert(coeff);
            }
            left
        }
        Expr::Sub(left, right) => {
            let (mut left, right) = (parse(left), parse(right));
            for (term, coeff) in right.into_iter() {
                left.entry(term)
                    .and_modify(|existing_coeff| {
                        *existing_coeff -= coeff;
                    })
                    .or_insert(coeff);
            }
            left
        }
        Expr::Mul(left, right) => {
            let (left, right) = (parse(left), parse(right));
            let mut map = HashMap::new();

            for (term1, coeff1) in left.iter() {
                for (term2, coeff2) in right.iter() {
                    let term = match (term1, term2) {
                        (Term::Coeff, Term::Coeff) => Term::Coeff,
                        (Term::Coeff, Term::Linear(j)) => Term::Linear(*j),
                        (Term::Linear(i), Term::Coeff) => Term::Linear(*i),
                        (Term::Linear(i), Term::Linear(j)) => Term::Quadratic(*i, *j),
                        (Term::Coeff, Term::Quadratic(i, j)) => Term::Quadratic(*i, *j),
                        (Term::Quadratic(i, j), Term::Coeff) => Term::Quadratic(*i, *j),
                        (Term::Linear(_), Term::Quadratic(_, _)) => todo!(),
                        (Term::Quadratic(_, _), Term::Linear(_)) => todo!(),
                        (Term::Quadratic(_, _), Term::Quadratic(_, _)) => todo!(),
                    };

                    *map.entry(term).or_insert(F::ZERO) += *coeff1 * *coeff2;
                }
            }
            map
        }
    }
}

#[derive(Debug, Clone)]
pub struct Constraint<F>(
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
);

#[derive(Debug, Clone)]
pub struct R1CS<F>(pub Vec<Constraint<F>>);

pub struct ASTs<F: Field> {
    pub(crate) permu: Vec<usize>,
    pub(crate) exprs: Vec<Expr<F>>,
}

impl<F: Field> R1CS<F> {

    pub fn optimize(&mut self) {
        todo!();
    }
}

impl<F: Field> ASTs<F> {
    pub fn compile(self) -> R1CS<F> {
        let constraints: Vec<_> = self
            .exprs
            .iter()
            .map(|expr| {
                let mut constraint = ASTs::convert(expr);
                self.permutate(&mut constraint.0);
                self.permutate(&mut constraint.1);
                self.permutate(&mut constraint.2);

                constraint
            })
            .collect();

        R1CS(constraints)
    }

    fn permutate(&self, a: &mut [(F, usize)]) {
        // ① インデックスを書き換え
        a.iter_mut().for_each(|(_, idx)| *idx = self.permu[*idx]);

        // ② idx でソート
        a.sort_by_key(|&(_, idx)| idx);
    }

    fn convert(exp: &Expr<F>) -> Constraint<F> {
        let map = parse(exp);

        let mut a = Vec::new();
        let mut b = Vec::new();
        let mut c = Vec::new();

        for (term, coeff) in map.into_iter() {
            match term {
                Term::Coeff => todo!(),
                Term::Linear(i) => c.push((-coeff, i)),
                Term::Quadratic(i, j) => {
                    a.push((coeff, i));
                    b.push((F::ONE, j));
                }
            }
        }
        // 左辺に項が無い場合は定数1を補完
        if a.is_empty() {
            a.push((F::ONE, 0));
        }
        if b.is_empty() {
            b.push((F::ONE, 0));
        }

        Constraint(a, b, c)
    }
}


/* ---------- Display 実装 ---------- */

impl<F: Field> Display for Constraint<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "A: {:?}", self.0)?;
        writeln!(f, "B: {:?}", self.1)?;
        writeln!(f, "C: {:?}", self.2)
    }
}

impl<F: Field> Display for R1CS<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, c) in self.0.iter().enumerate() {
            writeln!(f, "Constraint #{i}")?;
            writeln!(f, "{c}")?;
        }
        Ok(())
    }
}
