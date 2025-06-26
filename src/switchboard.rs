use ark_ff::Field;

use crate::{ConstraintSystemRef, V, utils::enforce_bits, wire::Wire};

pub struct SwitchBoard<'a, F: Field> {

    cs: &'a ConstraintSystemRef<'a, F>,
}

impl<'a, F: Field> SwitchBoard<'a, F> {
    pub fn set(index: usize, selector: Wire<'a, F>) {
        
    }
}

// #[derive(Clone)]
// pub struct SwitchboardSystem<'a, F: Field> {
//     cs: &'a ConstraintSystemRef<'a, F>,
//     turn_on: Wire<'a,F>,
//     switches: Vec<Wire<'a, F>>,
//     one: Wire<'a, F>,
//     mode: bool,
//     index: u64,
// }
//
// impl<'a, F: Field> SwitchboardSystem<'a, F> {
//     pub fn new(cs: &'a ConstraintSystemRef<'a, F>, turn_on: Wire<'a, F>) -> Self {
//         Self {
//             one: cs.one(), // 保存しておく。
//             mode: cs.switch(),
//             turn_on,
//             switches: vec![],
//             cs,
//             index: 0,
//         }
//     }
//
//     pub fn set(&mut self, index: u64) -> &'a ConstraintSystemRef<'a, F> {
//         // 順番にスイッチを割り当てているかを確認する。
//         assert!(index == self.index);
//         self.index += 1;
//
//         // この回路が有効かどうか
//         let sw = self.turn_on.raw() == F::from(index);
//         self.cs.set_switch(sw);
//         // スイッチのビットをを割り当てて、それをsub回路の定数に使う。
//         let sw = self.cs.alloc(sw);
//         self.switches.push(sw);
//         self.cs.set_one(sw); // oneとswは同じ。ONなら1だし、OFFなら0になる。
//         self.cs
//     }
//
//     pub fn finialize(self) -> &'a ConstraintSystemRef<'a, F> {
//         let cs = self.cs;
//         cs.set_one(self.one); // csを元にもしておく。
//         cs.set_switch(self.mode); //
//
//         // 全てがbit
//         enforce_bits(cs.clone(), &self.switches);
//         // 合計で1
//         let sum: V<F> = self.switches.clone().into_iter().sum();
//         cs.link(sum * cs.one(), 1);
//         // turn_onとswitchが同じか
//         let sum: V<F> = self
//             .switches
//             .into_iter()
//             .enumerate()
//             .map(|(i, s)| s * i as u64)
//             .sum();
//         cs.link(cs.one() * (sum - self.turn_on), 0);
//
//         cs
//     }
// }
