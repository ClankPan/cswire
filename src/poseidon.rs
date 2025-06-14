use ark_crypto_primitives::sponge::poseidon::find_poseidon_ark_and_mds;
use ark_ff::{Field, PrimeField};

use crate::{
    utils::pow, variables::{Wire, V}, CSRef
};

pub struct Poseidon<F: Field> {
    cs: CSRef<F>,
    state: Vec<V<F>>,
    full_rounds: usize,
    t: usize,
    alpha: usize,
    partial_rounds: usize,
    round_constants: Vec<Vec<V<F>>>,
    mds_matrix: Vec<Vec<V<F>>>,
    pos: usize,
}

impl<F: PrimeField> Poseidon<F> {
    pub fn new(
        cs: CSRef<F>,
        full_rounds: usize,
        partial_rounds: usize,
        alpha: usize,
        rate: usize,
        capacity: usize,
    ) -> Self {
        let zero = cs.one() * 0u32;
        let t = rate + capacity;

        let (ark, mds) = find_poseidon_ark_and_mds::<F>(
            F::MODULUS_BIT_SIZE as u64,
            rate,
            full_rounds as u64,
            partial_rounds as u64,
            0,
        );

        assert_eq!(ark.len(), full_rounds + partial_rounds);
        assert_eq!(mds.len(), t);
        assert!(mds.iter().all(|row| row.len() == t));

        Self {
            cs,
            state: vec![zero; t],
            full_rounds,
            t,
            alpha,
            partial_rounds,
            round_constants: ark
                .into_iter()
                .map(|raw| raw.into_iter().map(|v| v.into()).collect())
                .collect(),
            mds_matrix: mds
                .into_iter()
                .map(|raw| raw.into_iter().map(|v| v.into()).collect())
                .collect(),
            pos: 0,
        }
    }
    pub fn absorb(&mut self, input: &[Wire<F>]) {
        for elem in input {
            self.state[self.pos] += elem;
            self.pos += 1;
            if self.pos == self.t {
                self.permute();
                self.pos = 0;
            }
        }
    }
    pub fn squeeze(&mut self, num: usize) -> Vec<V<F>> {
        let mut output = Vec::with_capacity(num);
        for _ in 0..num {
            if self.pos == self.t {
                self.permute();
                self.pos = 0;
            }
            output.push(self.state[self.pos].clone());
            self.pos += 1;
        }
        output
    }

    fn permute(&mut self) {
        let half_full_rounds = self.full_rounds / 2;
        let mut round_idx = 0;

        // === 前半のFull rounds ===
        for _ in 0..half_full_rounds {
            self.add_round_constants(round_idx);
            self.apply_full_sbox();
            self.apply_mds();
            round_idx += 1;
        }

        // === 中間のPartial rounds ===
        for _ in 0..self.partial_rounds {
            self.add_round_constants(round_idx);
            self.apply_partial_sbox();
            self.apply_mds();
            round_idx += 1;
        }

        // === 後半のFull rounds ===
        for _ in 0..half_full_rounds {
            self.add_round_constants(round_idx);
            self.apply_full_sbox();
            self.apply_mds();
            round_idx += 1;
        }
    }

    // ラウンド定数を加算する
    fn add_round_constants(&mut self, round: usize) {
        for (s, rc) in self.state.iter_mut().zip(&self.round_constants[round]) {
            *s += rc;
        }
    }

    // 全要素にS-box (例:3乗)を適用 (Full rounds)
    fn apply_full_sbox(&mut self) {
        for s in &mut self.state {
            *s = pow(self.cs.clone(), s.clone(), self.alpha); // s^apha
        }
    }

    // 最初の要素のみS-boxを適用 (Partial rounds)
    fn apply_partial_sbox(&mut self) {
        self.state[0] = pow(self.cs.clone(), self.state[0].clone(), self.alpha); // s^apha
    }

    // MDS行列による線形変換
    fn apply_mds(&mut self) {
        let new_state: Vec<V<F>> = self
            .mds_matrix
            .iter()
            .map(|row| {
                row.iter()
                    .zip(&self.state)
                    .map(|(m, s)| self.cs.wire(m * s))
                    .sum()
            })
            .collect();

        self.state = new_state;
    }
}

