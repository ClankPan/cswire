#[derive(Clone, Copy, Debug)]
pub struct Idx(pub(crate) usize);

#[derive(Clone, Debug)]
pub enum Exp<F> {
    Idx(Idx),
    Con(F),                        // 係数
    Add(Box<Exp<F>>, Box<Exp<F>>), // 加算
    Sub(Box<Exp<F>>, Box<Exp<F>>), // 減算
    Mul(Box<Exp<F>>, Box<Exp<F>>), // 乗算
}
