/*
* 元の実装はここから
* https://github.com/arkworks-rs/crypto-primitives/blob/5f41c00669079d477077ab7521940248ec1a289d/crypto-primitives/src/sponge/poseidon/mod.rs#L54
*/

use ark_crypto_primitives::sponge::poseidon::find_poseidon_ark_and_mds;
use ark_ff::PrimeField;

use crate::{ConstraintSystemRef, V, wire::Coeff, utils::pow};

// use crate::{
//     CSRef,
//     utils::pow,
//     variables::{ConstraintSystemRef, V},
// };

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
    pub ark: Vec<Vec<F>>,
    /// Maximally Distance Separating (MDS) Matrix.
    pub mds: Vec<Vec<F>>,
    /// The rate (in terms of number of field elements).
    /// See [On the Indifferentiability of the Sponge Construction](https://iacr.org/archive/eurocrypt2008/49650180/49650180.pdf)
    /// for more details on the rate and capacity of a sponge.
    pub rate: usize,
    /// The capacity (in terms of number of field elements).
    pub capacity: usize,
}

impl<F: PrimeField> PoseidonConfig<F> {
    /// Initialize the parameter for Poseidon Sponge.
    pub fn new(
        full_rounds: usize,
        partial_rounds: usize,
        alpha: u64,
        mds: Vec<Vec<F>>,
        ark: Vec<Vec<F>>,
        rate: usize,
        capacity: usize,
    ) -> Self {
        assert_eq!(ark.len(), full_rounds + partial_rounds);
        for item in &ark {
            assert_eq!(item.len(), rate + capacity);
        }
        assert_eq!(mds.len(), rate + capacity);
        for item in &mds {
            assert_eq!(item.len(), rate + capacity);
        }
        Self {
            full_rounds,
            partial_rounds,
            alpha,
            mds,
            ark,
            rate,
            capacity,
        }
    }
}

#[derive(Clone)]
/// A duplex sponge based using the Poseidon permutation.
///
/// This implementation of Poseidon is entirely from Fractal's implementation in [COS20][cos]
/// with small syntax changes.
///
/// [cos]: https://eprint.iacr.org/2019/1076
pub struct PoseidonSponge<'a, F: PrimeField> {
    /// Sponge Config
    pub parameters: PoseidonConfig<F>,

    // Sponge State
    /// Current sponge's state (current elements in the permutation block)
    pub state: Vec<V<'a, F>>,
    /// Current mode (whether its absorbing or squeezing)
    pub mode: DuplexSpongeMode,

    /// ConstraintSystem
    cs: ConstraintSystemRef<'a, F>,
    // pub ark: Vec<Vec<V<F>>>,
    // pub mds: Vec<Vec<V<F>>>,
}

impl<'a, F: PrimeField> PoseidonSponge<'a, F> {
    fn apply_s_box(&self, state: &mut [V<'a, F>], is_full_round: bool) {
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

    fn apply_ark(&self, state: &mut [V<'a, F>], round_number: usize) {
        for (i, state_elem) in state.iter_mut().enumerate() {
            let ark_i = self.parameters.ark[round_number][i];
            *state_elem += Coeff::new(ark_i) * self.cs.one();
        }
    }

    fn apply_mds(&self, state: &mut [V<'a, F>]) {
        let mut new_state = Vec::new();
        for i in 0..state.len() {
            let mut cur = self.cs.one() * 0u32;
            for (j, state_elem) in state.iter().enumerate() {
                // wireはVVしか受け付けないので、oneを掛けてVVにしている。
                let term = state_elem * Coeff::new(self.parameters.mds[i][j]);
                let term = self.cs.wire(self.cs.one() * term);
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
    fn absorb_internal(&mut self, mut rate_start_index: usize, elements: &[V<'a, F>]) {
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
    fn squeeze_internal(&mut self, mut rate_start_index: usize, output: &mut [V<'a,F>]) {
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

impl<'a, F: PrimeField> PoseidonSponge<'a, F> {
    pub fn new(cs: ConstraintSystemRef<'a, F>, parameters: &PoseidonConfig<F>) -> Self {
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

    pub fn absorb(&mut self, input: &[V<'a, F>]) {
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

/// This Poseidon configuration generator produces a Poseidon configuration with custom parameters
pub fn poseidon_custom_config<F: PrimeField>(
    full_rounds: usize,
    partial_rounds: usize,
    alpha: u64,
    rate: usize,
    capacity: usize,
) -> PoseidonConfig<F> {
    let (ark, mds) = find_poseidon_ark_and_mds::<F>(
        F::MODULUS_BIT_SIZE as u64,
        rate,
        full_rounds as u64,
        partial_rounds as u64,
        0,
    );

    PoseidonConfig::new(full_rounds, partial_rounds, alpha, mds, ark, rate, capacity)
}

/// This Poseidon configuration generator agrees with Circom's Poseidon(4) in the case of BN254's scalar field
pub fn circom_bn254_poseidon_canonical_config<F: PrimeField>() -> PoseidonConfig<F> {
    // 120 bit security target as in
    // https://eprint.iacr.org/2019/458.pdf
    // t = rate + 1

    let full_rounds = 8;
    let partial_rounds = 60;
    let alpha = 5;
    let rate = 4;

    poseidon_custom_config(full_rounds, partial_rounds, alpha, rate, 1)
}

#[cfg(test)]
mod tests {
    use crate::ConstraintSystemRef;
    use ark_bn254::Fr;
    use ark_crypto_primitives::sponge::{
        CryptographicSponge, FieldBasedCryptographicSponge,
        poseidon::{
            PoseidonConfig as ArkPoseidonConfig, PoseidonSponge as ArkPoseidonSponge,
            find_poseidon_ark_and_mds,
        },
    };
    use ark_ff::PrimeField;

    use super::{PoseidonSponge as CWPoseidonSponge, circom_bn254_poseidon_canonical_config};

    /// This Poseidon configuration generator produces a Poseidon configuration with custom parameters
    pub fn poseidon_custom_config<F: PrimeField>(
        full_rounds: usize,
        partial_rounds: usize,
        alpha: u64,
        rate: usize,
        capacity: usize,
    ) -> ArkPoseidonConfig<F> {
        let (ark, mds) = find_poseidon_ark_and_mds::<F>(
            F::MODULUS_BIT_SIZE as u64,
            rate,
            full_rounds as u64,
            partial_rounds as u64,
            0,
        );

        ArkPoseidonConfig::new(full_rounds, partial_rounds, alpha, mds, ark, rate, capacity)
    }

    /// This Poseidon configuration generator agrees with Circom's Poseidon(4) in the case of BN254's scalar field
    pub fn poseidon_canonical_config<F: PrimeField>() -> ArkPoseidonConfig<F> {
        // 120 bit security target as in
        // https://eprint.iacr.org/2019/458.pdf
        // t = rate + 1

        let full_rounds = 8;
        let partial_rounds = 60;
        let alpha = 5;
        let rate = 4;

        poseidon_custom_config(full_rounds, partial_rounds, alpha, rate, 1)
    }

    #[test]
    pub fn test_poseidon() {
        let values: Vec<Fr> = (0..10).map(Fr::from).collect();

        // Arkのposeidon
        let mut sponge = ArkPoseidonSponge::<Fr>::new(&poseidon_canonical_config());
        for v in values.iter() {
            sponge.absorb(v);
        }
        let ark_hash = sponge.squeeze_native_field_elements(1)[0];

        // cswireのposeidon
        let cs = ConstraintSystemRef::new();
        let config = circom_bn254_poseidon_canonical_config::<Fr>();
        let mut sponge = CWPoseidonSponge::<Fr>::new(cs.clone(), &config);
        for v in values.iter() {
            sponge.absorb(&[cs.alloc(*v) * 1]); // absorbがWireを受け付けないので、係数を掛けてVにしてる。
        }
        let cw_hash = sponge.squeeze_native_field_elements(1)[0].clone();

        assert_eq!(ark_hash, cw_hash.raw())
    }
}
