# Dust: equality and references support and encoding

Andrea Lattuada, June 2nd 2021

In the document I will refer to both `struct`s and `enum`s as "algebraic data types", adts in short.

## Design dimensions

When designing the interface and encoding of equality and reference support for Dust, we need to consider both (a) the user facing language and diagnostics, and (b) the encoding to _z3_ (via _air_).

### Language

Rust uses the `PartialEq` and `Eq` trait to define the `==` (`.eq`) operator for types that implement them. An `==` implementation however does not necessarily conform to structural equality. `Eq` is implemented for `Cell`, but `Cell` has interior mutability (with an unsafe implmentation).

If the programmer uses `#[derive(PartialEq, Eq)]` for an adt without interior mutability, and all the recursively enclosed types have the same property, they obtain an `==` implementation that is structural equality. The built-in `StructuralEq` trait marks those adts where the `==` was automatically derived to be structural equality, but this property is shallow, and does not say anything about whether the enclosed types don't have interior mutability and have structural `==`.

The user needs to know, and sometimes affirm, when a type can be treated as immutable (lacking interior mutability) for the purpose of encoding; additionally, depending on the encoding, it may be important to distinguish between types that have structural equality and those that have an immutable api, but do not have structural equality, like `Vec<int>` (with the exception of `Vec::capacity` and related functions).

We also need to determine whether and how to support verifying the bodies functions for types that have an immutable api but do not have structural equality, e.g. `Vec<int>`. We may decide to restrict this support to the safe interior mutability mechanisms provided by the standard library (`Cell`, `RefCell`, `Mutex`, ...).

### Encoding

#### Adts with structural equality

Adts that only contain primitive types (possibly by means of other adts with the same property) can always have an equality implementation that conforms to smt equality (with structural equality). These can be encoded as _air_ datatypes, like in the following example:

```rust
#[derive(PartialEq, Eq)]
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

This also extends to generic adts whenever the type parameters also have equality that conforms to smt equality (with structural equality).

```rust
#[derive(PartialEq, Eq)]
struct Foo<A> { a: A, b: bool }
```

In this example `==` of `Foo<Thing>` would conform to smt equality, but `Foo<Custom>` would not.

#### Adts that have interior mutability (and/or raw pointers) but expose an "immutable" interface

This set of types can also be defined those adts that do _not_ have well-defined structural equality (i.e. where at least one of the fields has interior mutability, is a raw pointer, or is another adt that recursively contains these) but "hide" the interior mutability in their public interface, and act like "immutable" types. 

`RefCell` is not one such type, as one can change its value while holding a shared reference. The following is such a (slightly contrived) type:

```rust
struct List {
  contents: RefCell<Box<[u64]>>,
}

impl List {
  /// Makes an empty List
  pub fn new() -> List {
    List { contents: RefCell::new(Box::new()) }
  }

  /// Push an item at the end of the list
  pub fn push(&mut self, v: u64) {
    let borrowed = self.contents.borrow_mut();
    // use borrowed to reallocate the boxed slice, and copy data over
  }
  
  /// Get the item at position i, or None if it doesn't exist
  pub fn get(&self, i: usize) -> Option<u64> {
    self.contents.borrow().get(i)
  }
}
```

Again, we treat generic adts as having this property if the type parameters have at least structural equality. The design needs to clarify whether there's a difference between "immutable" adts with generic arguments that have structural equality and those with generic arguments that are themselves "immutable" adts.

Possibly with the exception of `Vec::capacity` and related methods, `Vec<T>` is a type that satisfies this property, when `T` is either structural (`Vec<Thing>`) or "immutable" (`Vec<Vec<Thing>>`).

## The `builtin::StructEq` trait

Because the `std::marker::StructuralEq` reflects only shallow structural equality, we add a verifier-specific marker trait, `builtin::StructEq`, which can only be implemented for an adt if its `==` implementation conforms to **structural** equality. Adts that implement this trait are encoded as `air` datatypes, and `==` for these types is encoded as smt equality.

