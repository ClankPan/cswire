# cswire

> ⚠️ **Warning**: This library is still under active development. Please do not use it in production environments.

`cswire` is a Rust library that simplifies the creation of Rank-1 Constraint Systems (R1CS) for zero-knowledge proofs, focusing on usability and readability. It is designed to address usability frustrations experienced when using existing libraries like `arkworks-relations`.

## Motivation

While developing custom Zero-Knowledge Virtual Machines (ZKVM) using `arkworks-relations`, the repeated pattern of explicitly allocating variables (`FpVar`) and enforcing equality (`enforce_equal`) became cumbersome and repetitive. Specifically:

* Values computed using `FpVar` required explicit constraint checks with `enforce_equal`.
* The same computation needed to be duplicated for allocation and enforcement, increasing code redundancy.
* Writing code for inside and outside of circuits was significantly different, reducing readability and maintainability.

`cswire` aims to overcome these limitations by allowing developers to use plain Rust code seamlessly for both circuit generation and witness computation.

## Features

* **Unified Coding**: Write the same Rust code for both circuit definition and witness calculation.
* **Automatic AST Tracking**: Internal data types (`Wire`, `V`, `VV`) automatically build an Abstract Syntax Tree (AST), enabling R1CS generation from standard computations.
* **Clear Syntax**: Intuitive syntax closely matching standard Rust semantics, improving readability.
* **Compile-time Constraint Checks**: Prevent invalid operations (e.g., multiplication of more than two variables) at compile time.

## Installation

```toml
[dependencies]
cswire = { git = "https://github.com/ClankPan/cswire.git" }
```

## Quick Start

Here's a simple example demonstrating basic usage:

```rust
use ark_bn254::Fr;
use ark_ff::Field;
use cswire::*;
 
fn main() {
    println!("Hello, world!");
    let cs = ConstraintSystem::<Fr>::new_ref(Mode::Compile);
    let a: Wire<Fr> = cs.alloc(Fr::from(11));
    let b: Wire<Fr> = cs.alloc(Fr::from(22));
    let c: Wire<Fr> = cs.wire(a * b - 1u64);
    assert!(c.raw() == Fr::from(11) * Fr::from(22) - Fr::ONE);

    // let r1cs = cs.compile();
    let witnesses = cs.witnesses();

    assert!(witnesses == vec![Fr::ONE, Fr::from(11), Fr::from(22), Fr::from(241)]);
}
```

## Advanced Usage

### Type System and Constraints

* **`Wire<F>`**: Represents an allocated witness variable.
* **`V<F>`**: Represents a linear combination of variables (including constants).
* **`VV<F>`**: Represents quadratic combinations (exactly two variables multiplied).

Example:

```rust
let cs = ConstraintSystem::<F>::new_ref(Mode::Compile);
let a: Wire<F> = cs.alloc(F::from(11));
let b: Wire<F> = cs.alloc(F::from(22));

let c: V<F> = a + b + 1; // Linear combination
let d: V<F> = 2 * a + 3 * b + 4; // Linear combination with constants
let e: VV<F> = a * b; // Quadratic combination
// let f = a * b * b; // Compile error, not allowed

let e_wire: Wire<F> = cs.wire(a * b); // Converts quadratic combination to wire with witness
```

### Example: Range Checks

Performing a ranged linear combination and a bit-range check:

```rust
pub fn ranged_linear_combination(cs: CSRef<Fr>, a: Wire<Fr>, b: Wire<Fr>, c: Wire<Fr>) -> Wire<Fr> {
    let d = cs.wire(a + b * c);
    range_check(cs, &d, 32);
    d
}

pub fn range_check(cs: CSRef<Fr>, v: &Wire<Fr>, bit_range: usize) {
    let one = cs.one();
    let bits = v.raw().into_bigint().to_bits_le();
    let bits: Vec<_> = bits.iter().map(|b| cs.alloc(*b)).collect();
    bits.iter().for_each(|b| cs.anchor((one - b) * b));

    let sum = (0..bit_range)
        .map(|i| 1 << i)
        .zip(bits)
        .map(|(coeff, b)| coeff * b)
        .sum::<V<Fr>>();

    cs.anchor(v - sum);
}
```

## Limitations

* Conditional branching based on witness values is unsupported. The user must ensure that computations produce the same AST for each input to maintain correctness.

## License

`cswire` is licensed under the MIT license.

## Contributions

Contributions are welcome! Please submit pull requests or open issues on GitHub.
