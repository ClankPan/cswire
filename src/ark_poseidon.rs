/*
* 元の実装はここから
* https://github.com/arkworks-rs/crypto-primitives/blob/5f41c00669079d477077ab7521940248ec1a289d/crypto-primitives/src/sponge/poseidon/mod.rs#L54
*/

use ark_crypto_primitives::sponge::FieldElementSize;
use ark_ff::{BigInteger, Field, PrimeField};

use crate::{
    CSRef,
    utils::pow,
    variables::{ConstraintSystemRef, V, Wire},
};

/// The mode structure for duplex sponges
#[derive(Clone, Debug)]
pub enum DuplexSpongeMode {
    /// The sponge is currently absorbing data.
    Absorbing {
        /// next position of the state to be XOR-ed when absorbing.
        next_absorb_index: usize,
    },
    /// The sponge is currently squeezing data out.
    Squeezing {
        /// next position of the state to be outputted when squeezing.
        next_squeeze_index: usize,
    },
}

/// Config and RNG used
#[derive(Clone, Debug)]
pub struct PoseidonConfig<F: PrimeField> {
    /// Number of rounds in a full-round operation.
    pub full_rounds: usize,
    /// Number of rounds in a partial-round operation.
    pub partial_rounds: usize,
    /// Exponent used in S-boxes.
    pub alpha: u64,
    /// Additive Round keys. These are added before each MDS matrix application to make it an affine shift.
    /// They are indexed by `ark[round_num][state_element_index]`
    pub ark: Vec<Vec<V<F>>>,
    /// Maximally Distance Separating (MDS) Matrix.
    pub mds: Vec<Vec<V<F>>>,
    /// The rate (in terms of number of field elements).
    /// See [On the Indifferentiability of the Sponge Construction](https://iacr.org/archive/eurocrypt2008/49650180/49650180.pdf)
    /// for more details on the rate and capacity of a sponge.
    pub rate: usize,
    /// The capacity (in terms of number of field elements).
    pub capacity: usize,
}

#[derive(Clone)]
/// A duplex sponge based using the Poseidon permutation.
///
/// This implementation of Poseidon is entirely from Fractal's implementation in [COS20][cos]
/// with small syntax changes.
///
/// [cos]: https://eprint.iacr.org/2019/1076
pub struct PoseidonSponge<F: PrimeField> {
    /// Sponge Config
    pub parameters: PoseidonConfig<F>,

    // Sponge State
    /// Current sponge's state (current elements in the permutation block)
    pub state: Vec<V<F>>,
    /// Current mode (whether its absorbing or squeezing)
    pub mode: DuplexSpongeMode,
    ///
    cs: ConstraintSystemRef<F>,
}

impl<F: PrimeField> PoseidonSponge<F> {
    fn apply_s_box(&self, state: &mut [V<F>], is_full_round: bool) {
        // Full rounds apply the S Box (x^alpha) to every element of state
        if is_full_round {
            for elem in state {
                *elem = pow(self.cs.clone(), elem.clone(), self.parameters.alpha);
            }
        }
        // Partial rounds apply the S Box (x^alpha) to just the first element of state
        else {
            state[0] = pow(self.cs.clone(), state[0].clone(), self.parameters.alpha);
        }
    }

    fn apply_ark(&self, state: &mut [V<F>], round_number: usize) {
        for (i, state_elem) in state.iter_mut().enumerate() {
            *state_elem += self.parameters.ark[round_number][i].clone();
        }
    }

    fn apply_mds(&self, state: &mut [V<F>]) {
        let mut new_state = Vec::new();
        for i in 0..state.len() {
            let mut cur = self.cs.one() * 0u32;
            for (j, state_elem) in state.iter().enumerate() {
                let term = self.cs.wire(state_elem * &self.parameters.mds[i][j]);
                cur += term;
            }
            new_state.push(cur);
        }
        state.clone_from_slice(&new_state[..state.len()])
    }

    fn permute(&mut self) {
        let full_rounds_over_2 = self.parameters.full_rounds / 2;
        let mut state = self.state.clone();
        for i in 0..full_rounds_over_2 {
            self.apply_ark(&mut state, i);
            self.apply_s_box(&mut state, true);
            self.apply_mds(&mut state);
        }

        for i in full_rounds_over_2..(full_rounds_over_2 + self.parameters.partial_rounds) {
            self.apply_ark(&mut state, i);
            self.apply_s_box(&mut state, false);
            self.apply_mds(&mut state);
        }

        for i in (full_rounds_over_2 + self.parameters.partial_rounds)
            ..(self.parameters.partial_rounds + self.parameters.full_rounds)
        {
            self.apply_ark(&mut state, i);
            self.apply_s_box(&mut state, true);
            self.apply_mds(&mut state);
        }
        self.state = state;
    }

    // Absorbs everything in elements, this does not end in an absorbtion.
    fn absorb_internal(&mut self, mut rate_start_index: usize, elements: &[V<F>]) {
        let mut remaining_elements = elements;

        loop {
            // if we can finish in this call
            if rate_start_index + remaining_elements.len() <= self.parameters.rate {
                for (i, element) in remaining_elements.iter().enumerate() {
                    self.state[self.parameters.capacity + i + rate_start_index] += element;
                }
                self.mode = DuplexSpongeMode::Absorbing {
                    next_absorb_index: rate_start_index + remaining_elements.len(),
                };

                return;
            }
            // otherwise absorb (rate - rate_start_index) elements
            let num_elements_absorbed = self.parameters.rate - rate_start_index;
            for (i, element) in remaining_elements
                .iter()
                .enumerate()
                .take(num_elements_absorbed)
            {
                self.state[self.parameters.capacity + i + rate_start_index] += element;
            }
            self.permute();
            // the input elements got truncated by num elements absorbed
            remaining_elements = &remaining_elements[num_elements_absorbed..];
            rate_start_index = 0;
        }
    }

    // Squeeze |output| many elements. This does not end in a squeeze
    fn squeeze_internal(&mut self, mut rate_start_index: usize, output: &mut [V<F>]) {
        let mut output_remaining = output;
        loop {
            // if we can finish in this call
            if rate_start_index + output_remaining.len() <= self.parameters.rate {
                output_remaining.clone_from_slice(
                    &self.state[self.parameters.capacity + rate_start_index
                        ..(self.parameters.capacity + output_remaining.len() + rate_start_index)],
                );
                self.mode = DuplexSpongeMode::Squeezing {
                    next_squeeze_index: rate_start_index + output_remaining.len(),
                };
                return;
            }
            // otherwise squeeze (rate - rate_start_index) elements
            let num_elements_squeezed = self.parameters.rate - rate_start_index;
            output_remaining[..num_elements_squeezed].clone_from_slice(
                &self.state[self.parameters.capacity + rate_start_index
                    ..(self.parameters.capacity + num_elements_squeezed + rate_start_index)],
            );

            // Repeat with updated output slices
            output_remaining = &mut output_remaining[num_elements_squeezed..];
            // Unless we are done with squeezing in this call, permute.
            if !output_remaining.is_empty() {
                self.permute();
            }

            rate_start_index = 0;
        }
    }
}

impl<F: PrimeField> PoseidonSponge<F> {
    pub fn new(cs: CSRef<F>, parameters: &PoseidonConfig<F>) -> Self {
        let state = vec![cs.one() * 0u32; parameters.rate + parameters.capacity];
        let mode = DuplexSpongeMode::Absorbing {
            next_absorb_index: 0,
        };

        Self {
            parameters: parameters.clone(),
            state,
            mode,
            cs,
        }
    }

    pub fn absorb(&mut self, input: &[V<F>]) {
        let elems = input;
        if elems.is_empty() {
            return;
        }

        match self.mode {
            DuplexSpongeMode::Absorbing { next_absorb_index } => {
                let mut absorb_index = next_absorb_index;
                if absorb_index == self.parameters.rate {
                    self.permute();
                    absorb_index = 0;
                }
                self.absorb_internal(absorb_index, elems);
            }
            DuplexSpongeMode::Squeezing {
                next_squeeze_index: _,
            } => {
                self.absorb_internal(0, elems);
            }
        };
    }
    pub fn squeeze_native_field_elements(&mut self, num_elements: usize) -> Vec<V<F>> {
        let mut squeezed_elems = vec![self.cs.one() * 0u32; num_elements];
        match self.mode {
            DuplexSpongeMode::Absorbing {
                next_absorb_index: _,
            } => {
                self.permute();
                self.squeeze_internal(0, &mut squeezed_elems);
            }
            DuplexSpongeMode::Squeezing { next_squeeze_index } => {
                let mut squeeze_index = next_squeeze_index;
                if squeeze_index == self.parameters.rate {
                    self.permute();
                    squeeze_index = 0;
                }
                self.squeeze_internal(squeeze_index, &mut squeezed_elems);
            }
        };

        squeezed_elems
    }
}
