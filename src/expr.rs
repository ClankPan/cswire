use std::collections::HashMap;

use ark_ff::Field;


#[derive(Clone, Debug)]
pub enum Expr<F> {
    Idx(usize),
    Con(F),                        // 係数
    Add(Box<Expr<F>>, Box<Expr<F>>), // 加算
    Sub(Box<Expr<F>>, Box<Expr<F>>), // 減算
    Mul(Box<Expr<F>>, Box<Expr<F>>), // 乗算
}

impl<F: Field> From<usize> for Expr<F> {
    fn from(idx: usize) -> Self {
        Expr::Idx(idx)
    }
}


pub struct Constraint<F>(
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
    pub Vec<(F, usize)>,
);
pub struct R1CS<F>(pub Vec<Constraint<F>>);

fn parse<F: Field>(exp: &Expr<F>) -> HashMap<(usize, usize), F> {
    match exp {

        // wip: 定数の位置はswitchboardによって変わるので、これに対応しないといけない。
        //
        Expr::Con(coeff) => HashMap::from([((0, 0), *coeff)]),
        Expr::Idx(idx) => HashMap::from([((*idx, 0), F::ONE)]),
        Expr::Add(lhs, rhs) => {
            let mut lhs = parse(lhs);
            let rhs = parse(rhs);

            for (idx, coeff) in rhs.into_iter() {
                lhs.entry(idx)
                    .and_modify(|existing_coeff| {
                        *existing_coeff += coeff;
                    })
                    .or_insert(coeff);
            }
            lhs
        }
        Expr::Sub(lhs, rhs) => {
            let mut lhs = parse(lhs);
            let rhs = parse(rhs);

            for (idx, coeff) in rhs.into_iter() {
                lhs.entry(idx)
                    .and_modify(|existing_coeff| {
                        *existing_coeff -= coeff;
                    })
                    .or_insert(-coeff);
            }
            lhs
        }
        Expr::Mul(lhs, rhs) => {
            let lhs = parse(lhs);
            let rhs = parse(rhs);
            let mut map = HashMap::new();

            for ((i1, j1), coeff1) in lhs.iter() {
                assert!(*j1 == 0, "lhs has quadratic term");
                for ((i2, j2), coeff2) in rhs.iter() {
                    assert!(*j2 == 0, "rhs has quadratic term");

                    let idx = if *i1 > *i2 { (*i1, *i2) } else { (*i2, *i1) };

                    *map.entry(idx).or_insert(F::ZERO) += *coeff1 * *coeff2;
                }
            }
            map
        }
    }
}

pub fn compile<F: Field>(exp: &Expr<F>) -> Constraint<F> {
    let map = parse(exp);

    let mut a = Vec::new();
    let mut b = Vec::new();
    let mut c = Vec::new();

    for ((x, y), coeff) in map.into_iter() {
        match (x, y) {
            (0, 0) => {
                // 定数項は右辺に移動（符号反転）
                c.push((-coeff, 0));
            }
            (x, 0) => {
                // 単変数項は右辺に移動（符号反転）
                c.push((-coeff, x));
            }
            (0, _y) => {
                panic!("Invalid term (0, y) should not exist.");
            }
            (x, y) => {
                // 二変数項は左辺に置く。通常は片方をleft_a、もう片方をleft_bに
                // 係数はどちらか片方にだけつける (ここではleft_aにつける)
                a.push((coeff, x));
                b.push((F::ONE, y));
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
