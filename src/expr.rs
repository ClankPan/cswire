use std::{
    collections::HashMap,
    ops::{Add, Mul, Sub},
    rc::Rc,
};

use ark_ff::Field;

use crate::Linear;

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
pub fn parse<'a, F: Field>(expr: &Expr<F>) -> HashMap<Term, F> {
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

            for (term1,  coeff1) in left.iter() {
                for (term2, coeff2) in right.iter() {
                
                   let term =  match (term1, term2) {
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
        },
    }
}

pub struct Constraint<F>(
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
);
pub struct R1CS<F>(pub Vec<Constraint<F>>);

pub fn compile<F: Field>(exp: &Expr<F>) -> Constraint<F> {
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
            },
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

// #[derive(Clone, Debug)]
// pub enum AST<F> {
//     Idx(usize),
//     Con(F),     // 係数
//     Add(usize), // 加算
//     Sub(usize), // 減算
//     Mul(usize), // 乗算
// }
// #[derive(Clone, Debug)]
// pub struct Expr<F: Field>(pub(crate) Vec<AST<F>>);
//
// // indexからexprを作る。
// impl<F: Field> From<usize> for Expr<F> {
//     fn from(idx: usize) -> Self {
//         Expr(vec![AST::Idx(idx)])
//     }
// }
//
// impl<F: Field> Expr<F> {
//     pub fn constant<T>(value: T) -> Self
//     where
//         F: From<T>,
//     {
//         Self(vec![AST::Con(F::from(value))])
//     }
// }
//
// impl<F: Field> Add for Expr<F> {
//     type Output = Self;
//     fn add(self, rhs: Self) -> Self::Output {
//         let nodes: Vec<AST<F>> = iter::once(AST::Add(self.0.len())) // 操作
//             .chain(self.0) // 左ノード
//             .chain(rhs.0) // 右ノード
//             .collect();
//         Expr(nodes)
//     }
// }
//
// impl<F: Field> Sub for Expr<F> {
//     type Output = Self;
//     fn sub(self, rhs: Self) -> Self::Output {
//         let nodes: Vec<AST<F>> = iter::once(AST::Sub(self.0.len())) // 操作
//             .chain(self.0) // 左ノード
//             .chain(rhs.0) // 右ノード
//             .collect();
//         Expr(nodes)
//     }
// }
//
// impl<F: Field> Mul for Expr<F> {
//     type Output = Self;
//     fn mul(self, rhs: Self) -> Self::Output {
//         let nodes: Vec<AST<F>> = iter::once(AST::Mul(self.0.len())) // 操作
//             .chain(self.0) // 左ノード
//             .chain(rhs.0) // 右ノード
//             .collect();
//         Expr(nodes)
//     }
// }
//
// pub struct Constraint<F>(
//     pub Vec<(F, usize)>,
//     pub Vec<(F, usize)>,
//     pub Vec<(F, usize)>,
// );
// pub struct R1CS<F>(pub Vec<Constraint<F>>);
//
// fn parse<F: Field>(exp: &Expr<F>) -> HashMap<(usize, usize), F> {
//     todo!()
//     // match exp {
//     //
//     //     // wip: 定数の位置はswitchboardによって変わるので、これに対応しないといけない。
//     //     //
//     //     Expr::Con(coeff) => HashMap::from([((0, 0), *coeff)]),
//     //     Expr::Idx(idx) => HashMap::from([((*idx, 0), F::ONE)]),
//     //     Expr::Add(lhs, rhs) => {
//     //         let mut lhs = parse(lhs);
//     //         let rhs = parse(rhs);
//     //
//     //         for (idx, coeff) in rhs.into_iter() {
//     //             lhs.entry(idx)
//     //                 .and_modify(|existing_coeff| {
//     //                     *existing_coeff += coeff;
//     //                 })
//     //                 .or_insert(coeff);
//     //         }
//     //         lhs
//     //     }
//     //     Expr::Sub(lhs, rhs) => {
//     //         let mut lhs = parse(lhs);
//     //         let rhs = parse(rhs);
//     //
//     //         for (idx, coeff) in rhs.into_iter() {
//     //             lhs.entry(idx)
//     //                 .and_modify(|existing_coeff| {
//     //                     *existing_coeff -= coeff;
//     //                 })
//     //                 .or_insert(-coeff);
//     //         }
//     //         lhs
//     //     }
//     //     Expr::Mul(lhs, rhs) => {
//     //         let lhs = parse(lhs);
//     //         let rhs = parse(rhs);
//     //         let mut map = HashMap::new();
//     //
//     //         for ((i1, j1), coeff1) in lhs.iter() {
//     //             assert!(*j1 == 0, "lhs has quadratic term");
//     //             for ((i2, j2), coeff2) in rhs.iter() {
//     //                 assert!(*j2 == 0, "rhs has quadratic term");
//     //
//     //                 let idx = if *i1 > *i2 { (*i1, *i2) } else { (*i2, *i1) };
//     //
//     //                 *map.entry(idx).or_insert(F::ZERO) += *coeff1 * *coeff2;
//     //             }
//     //         }
//     //         map
//     //     }
//     // }
// }
//
// pub fn compile<F: Field>(exp: &Expr<F>) -> Constraint<F> {
//     let map = parse(exp);
//
//     let mut a = Vec::new();
//     let mut b = Vec::new();
//     let mut c = Vec::new();
//
//     for ((x, y), coeff) in map.into_iter() {
//         match (x, y) {
//             (0, 0) => {
//                 // 定数項は右辺に移動（符号反転）
//                 c.push((-coeff, 0));
//             }
//             (x, 0) => {
//                 // 単変数項は右辺に移動（符号反転）
//                 c.push((-coeff, x));
//             }
//             (0, _y) => {
//                 panic!("Invalid term (0, y) should not exist.");
//             }
//             (x, y) => {
//                 // 二変数項は左辺に置く。通常は片方をleft_a、もう片方をleft_bに
//                 // 係数はどちらか片方にだけつける (ここではleft_aにつける)
//                 a.push((coeff, x));
//                 b.push((F::ONE, y));
//             }
//         }
//     }
//
//     // 左辺に項が無い場合は定数1を補完
//     if a.is_empty() {
//         a.push((F::ONE, 0));
//     }
//     if b.is_empty() {
//         b.push((F::ONE, 0));
//     }
//
//     Constraint(a, b, c)
// }
