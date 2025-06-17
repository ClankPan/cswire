// Switchboardをライブラリとして提供するのは、このcswireの思想からして難しい。
// cswireは、回路をRustの変数として扱う思想なので、csよりも大きいラッパーであるswitchboardを提供することはできなくて、
// switchboardを実装するのは、ユーザが書くRustプログラムのレイヤーになるので、
// 提供できるのは、switchboardパターンというプラクティスしかできない。
//
// CompileMode: ASTからR1CSを記録する。
// RunMode: Witnessだけ計算する。
// OffMode: allocateするwireのvalueを全て0する。
//
// let (x,y,z) = sb.input(x,y,z);
// add_op(sb.set(true), x,y,z);
// sub_op(sb.set(false), x,y,z);
// mul_op(sb.set(fasle), x,y,z);
// sb.finialize() // スイッチ制約を加える。
//
// sbは内部で、新しいcsを割り当てて、trueならば値つき、falseなら値なし実行する。
// すでにallocateされたwireの続きのindexから割り当てるけど、one()の扱いをどうしようか？
// swtichboardの時は、その専用に割り当てるように内部で覚えておく。
//
// switch変数の取り扱いをどうするかを決める。
// それぞれのbitに1,2,3,4,5,,,の係数を掛けて置いてその合計が元の値と同じかをみる。
// ビットが立っているかが1つだけか、全てビットか、はswitchboardが面倒をみる。
//
// let sw = (0..n).map(|i| sb.alloc(i==op.raw() as usize)).collect();
// sb.anchor(op - sw.iter().sum());
// add_op(sb.set(ture), x,y,z);
// sub_op(sb.set(false), x,y,z);
// let index = sb.index();
//
// let sum = sb.sw().enumlate().map(|(i,s)| (i+1)*s).sum();
// sb.anchor(op - sum);
//
// let sb = SB::new(cs, op);
// add_op(sb.idx(0), x,y,z);
// sub_op(sb.idx(1), x,y,z);
// mul_op(sb.idx(1), x,y,z);
//
// sb.idx(i)で、CSRefのラッパーをを返す。

use std::{cell::RefCell, rc::Rc};

use ark_ff::Field;

use crate::{
    CSRef, Mode, V, Wire,
    utils::enforce_bits,
    variables::{R1CS, VV, Wirable},
};

pub trait ConstraintSynthesizer<F: Field> {
    fn new(mode: Mode) -> Self;
    fn set_mode(&self, mode: Mode);
    fn compile(&self) -> R1CS<F>;
    fn witnesses(&self) -> Vec<F>;
    fn alloc<T>(&mut self, val: T) -> Wire<F>
    where
        F: From<T>;
    fn wire<W: Wirable<F>>(&self, w: W) -> Wire<F>;
    fn one(&self) -> Wire<F>;
    fn anchor<W: Wirable<F>>(&self, w: W);
    fn mode(&self) -> Mode;
}

#[derive(Clone)]
pub enum SwitchMode {
    OFF,
    ON,
    None,
}

#[derive(Clone)]
pub struct SwitchboardSystem<F: Field> {
    cs: CSRef<F>,
    turn_on: Wire<F>,
    switches: Vec<Wire<F>>,
    one: Wire<F>,
}

impl<F: Field> SwitchboardSystem<F> {
    pub fn new(cs: CSRef<F>, turn_on: Wire<F>) -> Self {
        Self {
            one: cs.one(), // 保存しておく。
            turn_on,
            switches: vec![],
            cs,
        }
    }

    pub fn idx(&mut self, index: u64) -> CSRef<F> {
        let sw = if self.turn_on.raw() == F::from(index) {
            self.cs.alloc(1)
        } else {
            // self.cs.mode = // ここで、Witnessがゼロモードにする。
            self.cs.alloc(0)
        };
        self.switches.push(sw);
        self.cs.set_one(sw); // oneとswは同じ。ONなら1だし、OFFなら0になる。
        self.cs.clone()
    }

    pub fn finialize(self) -> CSRef<F> {
        let mut cs = self.cs;
        cs.set_one(self.one); // csを元にもしておく。

        // 全てがbit
        enforce_bits(cs.clone(), &self.switches);
        // 合計で1
        let sum: V<F> = self.switches.clone().into_iter().sum();
        cs.anchor(sum - 1u64);
        // turn_onとswitchが同じか
        let sum: V<F> = self
            .switches
            .into_iter()
            .enumerate()
            .map(|(i, s)| s * i as u64)
            .sum();
        cs.anchor(sum - self.turn_on);

        cs
    }
}
