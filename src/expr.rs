use ark_ff::{BigInteger, Field, PrimeField};
use core::fmt;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    ops::{Add, Mul, Sub},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub enum Expr<F: Send + Sync> {
    Coeff(F),
    Idx(usize),
    Add(Arc<Expr<F>>, Arc<Expr<F>>),
    Sub(Arc<Expr<F>>, Arc<Expr<F>>),
    Mul(Arc<Expr<F>>, Arc<Expr<F>>),
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
        Expr::Add(Arc::new(self), Arc::new(rhs))
    }
}

/// 引き算と掛け算も同様
impl<F: Field> Sub for Expr<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Expr::Sub(Arc::new(self), Arc::new(rhs))
    }
}

impl<F: Field> Mul for Expr<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Expr::Mul(Arc::new(self), Arc::new(rhs))
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
            // println!("mul");
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
            // println!("add");
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
            // println!("sub");
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
    pub fn is_satisfied(&self, w: &[F]) -> bool {
        for constraint in &self.0 {
            let a: F = constraint.0.iter().map(|(i, coef)| w[*i] * coef).sum();
            let b: F = constraint.1.iter().map(|(i, coef)| w[*i] * coef).sum();
            let c: F = constraint.2.iter().map(|(i, coef)| w[*i] * coef).sum();
            if a * b - c != F::ZERO {
                return false;
            }
        }
        true
    }

    pub fn pad_to_next_power_of_2(&mut self) {
        let len = self.0.len();
        if len.is_power_of_two() {
            return; // すでに 2 の冪なら何もしない
        }

        let empty_constraint = Constraint::<F>(vec![], vec![], vec![]);
        let next_pow = len.next_power_of_two(); // ① 次の 2 の冪
        let pad = next_pow - len; // 欠けている個数
        self.0.extend(std::iter::repeat(empty_constraint).take(pad)); // ② 末尾に追加
    }

    pub fn num_of_constraints(&self) -> usize {
        self.0.len()
    }
}

impl<F: Field + Send + Sync> ASTs<F> {
    // pub fn compile(self) -> R1CS<F> {
    //     let constraints: Vec<_> = self
    //         .exprs
    //         .par_iter()
    //         .map(|expr| {
    //             let mut constraint = ASTs::convert(expr);
    //             self.permutate(&mut constraint.0);
    //             self.permutate(&mut constraint.1);
    //             self.permutate(&mut constraint.2);
    //
    //             constraint
    //         })
    //         .collect();
    //
    //     R1CS(constraints)
    // }

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
            Term::Linear(c) => Constraint(
                vec![],
                vec![],
                c.into_iter().map(|(i, f)| (i, -f)).collect(),
            ),
            Term::Quadratic(a, b, coeff, c) => {
                let a = a.into_iter().map(|(i, f)| (i, f * coeff)).collect();
                let b = b.into_iter().collect();
                let c = c.into_iter().map(|(i, f)| (i, -f)).collect();
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

// Refactored R1CS compiler – copy‑free, cache‑friendly, and ready for Rayon
// --------------------------------------------------------------------------
// Highlights
// ----------
// 1.  Eliminates the Term/HashMap detour → no quadratic copying.
// 2.  Uses `SmallVec` for most linear combinations (1‑8 terms stay on the stack).
// 3.  Performs permutation & dedup *while building* – no post‑sort per wire list.
// 4.  Keeps API surface identical: `Expr`, `Constraint`, `R1CS`, `ASTs`.
// 5.  Single‑pass DFS; trivially paralellisable via `.par_iter()`.

use smallvec::SmallVec;

/* ---------------- Core IR ---------------- */

// #[derive(Debug, Clone)]
// pub enum Expr<F: Field + Send + Sync> {
//     Coeff(F),
//     Idx(usize),
//     Add(Arc<Expr<F>>, Arc<Expr<F>>),
//     Sub(Arc<Expr<F>>, Arc<Expr<F>>),
//     Mul(Arc<Expr<F>>, Arc<Expr<F>>),
// }

// #[derive(Debug, Clone)]
// pub struct Constraint<F: Field>(
//     pub Vec<(usize, F)>, // A‑vector
//     pub Vec<(usize, F)>, // B‑vector
//     pub Vec<(usize, F)>, // C‑vector
// );

// #[derive(Debug, Clone)]
// pub struct R1CS<F: Field>(pub Vec<Constraint<F>>);
//
// pub struct ASTs<F: Field + Send + Sync> {
//     pub permu: Vec<usize>,
//     pub exprs: Vec<Expr<F>>, // one constraint per expression
// }

/* ---------------- Internal helpers ---------------- */

// type LC<F> = SmallVec<[(usize, F); 32]>; // local linear comb
type LC<F> = Vec<(usize, F)>; // local linear comb

#[derive(Default)]
struct Parts<F: Field> {
    a: LC<F>,
    b: LC<F>,
    c: LC<F>,
    scale: F, // accumulated scalar factor
}

// merge (push & combine duplicates) – assumes input already permuted
fn push<F: Field>(dst: &mut LC<F>, (idx, coef): (usize, F)) {
    if let Some(pos) = dst.iter_mut().find(|(i, _)| *i == idx) {
        pos.1 += coef;
    } else {
        dst.push((idx, coef));
    }
}

/* ---------------- DFS builder ---------------- */

fn dfs<F: Field>(e: &Expr<F>, permu: &[usize], out: &mut Parts<F>) {
    match e {
        // Expr::Coeff(c) => out.scale *= c,
        // Expr::Idx(i) => push(&mut out.c, (permu[*i], -F::ONE)), // move to RHS
        Expr::Coeff(c) => {},
        Expr::Idx(i) => {}, // move to RHS
        Expr::Add(l, r) => {
            dfs(l, permu, out);
            dfs(r, permu, out);
        }
        Expr::Sub(l, r) => {
            dfs(l, permu, out);
            // let prev = out.scale;
            // out.scale = -F::ONE; // negate right branch
            dfs(r, permu, out);
            // out.scale = prev;
        }
        Expr::Mul(l, r) => {
            // collect lhs
            let mut left = Parts::<F> {
                scale: F::ONE,
                ..Default::default()
            };
            dfs(l, permu, &mut left);
            // // collect rhs
            // let mut right = Parts::<F> {
            //     scale: F::ONE,
            //     ..Default::default()
            // };
            // dfs(r, permu, &mut right);
            //
            // // Outer product – push into A & B (one term per side)
            // for &(i, coef_l) in &left.c {
            //     push(&mut out.a, (i, coef_l * right.scale));
            // }
            // for &(j, coef_r) in &right.c {
            //     push(&mut out.b, (j, coef_r * left.scale));
            // }
            //
            // // carry over accumulated scalar
            // out.scale *= left.scale * right.scale;
        }
    }
}

/* ---------------- Public API ---------------- */

impl<F: Field + Send + Sync> ASTs<F> {
    pub fn compile(self) -> R1CS<F> {
        let permu = &self.permu;
        let exprs_len = self.exprs.len();
        println!("expr.len: {}", exprs_len);
        let mut constraints = Vec::with_capacity(self.exprs.len());
        for (count, expr) in self.exprs.into_iter().enumerate() {
                
                let mut parts = Parts::<F> {
                    scale: F::ONE,
                    ..Default::default()
                };
                dfs(&expr, permu, &mut parts);

                // scale A & B by accumulated scalar
                // let a = parts
                //     .a
                //     .into_iter()
                //     .map(|(i, c)| (i, c * parts.scale))
                //     .collect::<Vec<_>>();
                // let b: Vec<_> = parts.b.into_iter().collect();
                // let mut c = parts.c;
                // if parts.scale != F::ONE {
                //     // constant term becomes scale * 1 on C side
                //     push(&mut c, (0, parts.scale));
                // }
                // println!("count: {count}/{exprs_len}, len of a:{},b:{},c:{}", a.len(),b.len(),c.len());
                // constraints.push(Constraint(a, b, c.into_iter().collect()));


                println!("count: {count}/{exprs_len}");
                constraints.push(Constraint(vec![],vec![],vec![]));
                
        }
        // let constraints: Vec<_> = self
        //     .exprs
        //     // .par_iter()
        //     .into_iter()
        //     .enumerate()
        //     .map(|(count, expr)| {
        //         let mut parts = Parts::<F> {
        //             scale: F::ONE,
        //             ..Default::default()
        //         };
        //         dfs(&expr, permu, &mut parts);
        //
        //         // scale A & B by accumulated scalar
        //         let a = parts
        //             .a
        //             .into_iter()
        //             .map(|(i, c)| (i, c * parts.scale))
        //             .collect::<Vec<_>>();
        //         let b: Vec<_> = parts.b.into_iter().collect();
        //         let mut c = parts.c;
        //         if parts.scale != F::ONE {
        //             // constant term becomes scale * 1 on C side
        //             push(&mut c, (0, parts.scale));
        //         }
        //         println!("count: {count}/{exprs_len}, len of a:{},b:{},c:{}", a.len(),b.len(),c.len());
        //         Constraint(a, b, c.into_iter().collect())
        //     })
        //     .collect();

        R1CS(constraints)
    }
}
