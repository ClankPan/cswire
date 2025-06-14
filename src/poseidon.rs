use ark_ff::Field;

use crate::variables::V;

pub struct Poseidon<F: Field> {
    state: Vec<V<F>>,
    full_rounds: usize,
    partial_rounds: usize,
    round_constants: Vec<Vec<F>>,
    mds_matrix: Vec<Vec<F>>,
}
