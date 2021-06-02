# Dust: equality and references support and encoding

Andrea Lattuada, June 2nd 2021

In the document I will refer to both `struct`s and `enum`s as "algebraic data types", adts in short.

## Design dimensions

When designing the interface and encoding of equality and reference support for Dust, we need to consider both (a) the user facing language and diagnostics, and (b) the encoding to _z3_ (via _air_).

### Language

Rust uses the `PartialEq` and `Eq` trait to define the `==` (`.eq`) operator for types that implement them. An `==` implementation however does not necessarily conform to structural equality. `Eq` is implemented for `Cell`, but `Cell` has interior mutability (with an unsafe implmentation).

If the programmer uses `#[derive(PartialEq, Eq)]` for an adt without interior mutability, and all the recursively enclosed types have the same property, they obtain an `==` implementation that is structural equality. The built-in `StructuralEq` trait marks those adts where the `==` was automatically derived to be structural equality, but this property is shallow, and does not say anything about whether the enclosed types don't have interior mutability and have structural `==`.

The user needs to know, and sometimes affirm, when a type can be treated as immutable (lacking interior mutability) for the purpose of encoding; additionally, depending on the encoding, it may be important to distinguish between types that have structural equality and those that have an immutable api, but do not have structural equality, like `Vec` (with the exception of `Vec::capacity` and related functions).

We also need to determine whether and how to support verifying the bodies functions for types that have an immutable api but do not have structural equality, e.g. `Vec`. We may decide to restrict this support to the safe interior mutability mechanisms provided by the standard library (`Cell`, `RefCell`, `Mutex`, ...).

### Encoding

#### "Immutable" adts (no interior mutability)

Adts that only contain primitive types (possibly by means of other adts with the same property) can always have an equality implementation that conforms to smt equality (with structural equality). These can be encoded as _air_ datatypes, like in the following example:

```rust
struct Thing { a: int, b: bool }
...
let t = Thing { a: 12, b: true };
```

```
(declare-datatypes () ((Thing (Thing/Thing (Thing/Thing/a Int) (Thing/Thing/b Bool)))))
...
(declare-const t@ Thing)
(assume
 (= t@ (Thing/Thing 12 true))
)
```

For these types, if the `==` implementation is structural equality, we can encode `==` to smt equality:

```rust
affirm(Thing { a: 12, b: true } != Thing { a: 14, b: true });
```

```
(assert
 "<span>" (req%affirm (not (= (Thing/Thing 12 true) (Thing/Thing 14 true)))))
```

This encoding is however unsound in general for any other adt, for example, for the following struct:

```rust
#[derive(Eq)]
struct Custom { a: int }

impl std::cmp::PartialEq for Custom {
    fn eq(&self, other: &Self) -> bool {
        self.a == other.a + 1
    }
}
```

