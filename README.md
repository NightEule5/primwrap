# Primitive Wrapper

Primitive Wrapper is a small derive macro for generating operation trait implementations for primitive
wrapper structs, making the inner value transparent:

```rust
#[derive(Primitive)]
struct Int(u64);

fn main() {
    let input = Int(0xFF0000);
    let mask = Int(0xFF);
    let result = input >> 16 & mask;
    assert_eq!(result, 0xFF);
}
```

The implemented traits are:
- Arithmetic: `Add`, `Sub`, `Mul`, `Div`, `Rem`, `Neg`
- Bitwise: `Not`, `BitAnd`, `BitOr`, `BitXor`, `Shl`, `Shr`
- Passthrough Formatting: `Debug`, `Display`, `Binary`, `LowerExp`, `LowerHex`, `Octal`, `UpperExp`, `UpperHex`
- Comparison: `PartialEq`/`PartialOrd` with the inner type
- Accumulation: `Sum` and `Product`

By default, all of the above traits are implemented. These groups can also be selected individually:

```rust
#[derive(Primitive)]
#[primwrap(arithmetic, bitwise, formatting, comparison, accumulation)]
struct Int(u64);
```

## Prior Art

This crate provides similar functionality to the [newtype_derive](https://crates.io/crates/newtype_derive)
crate, but the derived traits are specified individually. It is more generalized for all new-type patterns,
whereas this crate is designed only for new-types wrapping integers, floats, and `bool`. Use newtype_derive if you
need more fine-grained control over the traits implemented.

[amplify_derive](https://crates.io/crates/amplify_derive) also provides this functionality, with `#[derive(Wrapper)]`.
This macro is more powerful and flexible—it derives more traits and supports more complex types—but requires
the nightly Rust compiler for most derived traits. Prim Wrap works only for simple, known primitive types,
sidestepping the `trivial_bounds` feature-gate. Amplify also only implements traits for the wrapper type itself,
rather than the inner type (i.e. `Add<[inner type]>`).
