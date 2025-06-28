use ark_ff::{BigInteger, Field, PrimeField};
use core::fmt;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::{Add, Mul, Sub},
    rc::Rc,
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

#[derive(Debug)]
pub enum Term<F: Field> {
    Coeff(F),
    Linear(HashMap<usize, F>),
    Quadratic(Vec<(usize, F)>, Vec<(usize, F)>, F, HashMap<usize, F>),
}

pub fn parse<F: Field>(expr: &Expr<F>) -> Term<F> {
    match expr {
        Expr::Coeff(coeff) => Term::Coeff(*coeff),
        Expr::Idx(i) => Term::Linear(HashMap::from([(*i, F::ONE)])),
        Expr::Mul(left, right) => {
            println!("mul");
            match (parse(left), parse(right)) {
                (Term::Coeff(coeff1), Term::Coeff(coeff2)) => Term::Coeff(coeff1 * coeff2),
                (Term::Coeff(coeff), Term::Linear(mut map))
                | (Term::Linear(mut map), Term::Coeff(coeff)) => {
                    map.iter_mut().for_each(|(_, f)| *f *= coeff);
                    Term::Linear(map)
                }
                (Term::Coeff(coeff), Term::Quadratic(a, b, f, mut lc))
                | (Term::Quadratic(a, b, f, mut lc), Term::Coeff(coeff)) => {
                    lc.iter_mut().for_each(|(_, f)| *f *= coeff);
                    Term::Quadratic(a, b, f * coeff, lc)
                }
                (Term::Linear(a), Term::Linear(b)) => Term::Quadratic(
                    a.into_iter().collect(),
                    b.into_iter().collect(),
                    F::ONE,
                    HashMap::new(),
                ),
                (Term::Linear(_), Term::Quadratic(_, _, _, _))
                | (Term::Quadratic(_, _, _, _), Term::Linear(_)) => panic!(),
                (Term::Quadratic(_, _, _, _), Term::Quadratic(_, _, _, _)) => panic!(),
            }
        }

        Expr::Add(left, right) => {
            println!("add");
            match (parse(left), parse(right)) {
                (Term::Coeff(_), Term::Coeff(_)) => panic!(),
                (Term::Coeff(_), Term::Linear(_)) => panic!(),
                (Term::Coeff(_), Term::Quadratic(_, _, _, _)) => panic!(),
                (Term::Linear(_), Term::Coeff(_)) => panic!(),
                (Term::Linear(lc1), Term::Linear(lc2)) => Term::Linear(add_two_lc(lc1, lc2)),
                (Term::Linear(lc1), Term::Quadratic(a, b, f, lc2))
                | (Term::Quadratic(a, b, f, lc1), Term::Linear(lc2)) => {
                    Term::Quadratic(a, b, f, add_two_lc(lc1, lc2))
                }
                (Term::Quadratic(_, _, _, _), Term::Coeff(_)) => panic!(),
                (Term::Quadratic(_, _, _, _), Term::Quadratic(_, _, _, _)) => panic!(),
            }
        }

        Expr::Sub(left, right) => {
            println!("sub");
            match (parse(left), parse(right)) {
                (Term::Coeff(_), Term::Coeff(_)) => panic!(),
                (Term::Coeff(_), Term::Linear(_)) => panic!(),
                (Term::Coeff(_), Term::Quadratic(_, _, _, _)) => panic!(),
                (Term::Linear(_), Term::Coeff(_)) => panic!(),
                (Term::Linear(lc1), Term::Linear(lc2)) => Term::Linear(sub_two_lc(lc1, lc2)),
                (Term::Linear(lc1), Term::Quadratic(a, b, f, lc2))
                | (Term::Quadratic(a, b, f, lc1), Term::Linear(lc2)) => {
                    Term::Quadratic(a, b, f, sub_two_lc(lc1, lc2))
                }
                (Term::Quadratic(_, _, _, _), Term::Coeff(_)) => panic!(),
                (Term::Quadratic(_, _, _, _), Term::Quadratic(_, _, _, _)) => panic!(),
            }
        }
    }
}

fn add_two_lc<F: Field>(mut lc1: HashMap<usize, F>, lc2: HashMap<usize, F>) -> HashMap<usize, F> {
    for (i, coeff) in lc2.into_iter() {
        lc1.entry(i)
            .and_modify(|existing_coeff| {
                *existing_coeff += coeff;
            })
            .or_insert(coeff);
    }
    lc1
}

fn sub_two_lc<F: Field>(mut lc1: HashMap<usize, F>, lc2: HashMap<usize, F>) -> HashMap<usize, F> {
    for (i, coeff) in lc2.into_iter() {
        lc1.entry(i)
            .and_modify(|existing_coeff| {
                *existing_coeff -= coeff;
            })
            .or_insert(-coeff);
    }
    lc1
}

#[derive(Debug, Clone)]
pub struct Constraint<F>(
    pub Vec<(usize, F)>,
    pub Vec<(usize, F)>,
    pub Vec<(usize, F)>,
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

    fn permutate(&self, a: &mut [(usize, F)]) {
        // ① インデックスを書き換え
        a.iter_mut().for_each(|(idx, _)| *idx = self.permu[*idx]);

        // ② idx でソート
        a.sort_by_key(|&(_, idx)| idx);
    }

    fn convert(exp: &Expr<F>) -> Constraint<F> {
        let term = parse(exp);

        match term {
            Term::Coeff(_) => todo!(),
            Term::Linear(c) => Constraint(vec![], vec![], c.into_iter().collect()),
            Term::Quadratic(a, b, coeff, c) => {
                let a = a.into_iter().map(|(i, f)| (i, f * coeff)).collect();
                let b = b.into_iter().collect();
                let c = c.into_iter().collect();
                Constraint(a, b, c)
            }
        }
    }
}

/* ---------- Display 実装 ---------- */

impl<F: PrimeField> Display for Constraint<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let show = |vec: &[(usize, F)]| {
            vec.iter()
                .map(|(i, c)| format!("(w:{}, {})", i, I32Coeff(*c)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        writeln!(f, "A: [{}]", show(&self.0))?;
        writeln!(f, "B: [{}]", show(&self.1))?;
        writeln!(f, "C: [{}]", show(&self.2))
    }
}

impl<F: PrimeField> Display for R1CS<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, c) in self.0.iter().enumerate() {
            writeln!(f, "Constraint #{i}")?;
            writeln!(f, "{c}")?;
        }
        Ok(())
    }
}

/// ── 係数ラッパ ──
struct I32Coeff<F: PrimeField>(F);

impl<F: PrimeField> Display for I32Coeff<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 素体 p とその半分 (p/2)
        let p = F::MODULUS;
        let mut half = p;
        half.div2();

        // 係数を BigInt に
        let x = self.0.into_bigint();

        // x > p/2 なら負側へ射影：  x' = x - p  (結果は  -(p - x) )
        let signed = if x > half {
            // p - x は必ず正
            let mut tmp = p;
            let _borrow = tmp.sub_with_borrow(&x);
            // "-" を付けた文字列
            format!("-{}", tmp)
        } else {
            format!("{}", x)
        };

        // i32 範囲チェック
        match signed.parse::<i128>() {
            Ok(n) if n.abs() <= i32::MAX as i128 => write!(f, "{n}"),
            _ => write!(f, "<overflow>"), // 収まらないときは警告表示
        }
    }
}
