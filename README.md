# Primitive Wrapper

Primitive Wrapper is a small derive macro for generating operation trait implementations for primitive
wrapper structs, making the inner value transparent:

```rust
#[derive(Debug, Primitive)]
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
- `PartialEq`/`PartialOrd` with the inner type
