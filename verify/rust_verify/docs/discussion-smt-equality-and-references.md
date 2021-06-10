Would folks see anything wrong with opt-in smt equality for ADTs in Iron/Galv?
I’m thinking of adding a marker trait, e.g. `builtin::SmtEq` , that indiciates that a type supports structural smt equality. This lets us precisely control what from the standard library should be allowed in types that have `SmtEq`, but would require the user to explicitly tag `struct` s and `enum`s as `SmtEq`, as follows:

```
#[derive(PartialEq, Eq, SmtEq)]
struct Thing<A> {
  one: int,
  two: A,
}
```

and a `Thing<A>` would only support `SmtEq` if `A` is also `SmtEq`
(for the rust folks: the derived impl would be `unsafe impl<T: SmtEq> SmtEq for Thing<T>` , and additionally the verifier would check that `Eq` is the structural-equality auto-derived one).
The plan would also be that immutable references to types that implement `SmtEq` are handled as if they just copied the value they reference (a “functional” behaviour similar to `shared` in linear dafny).
As part of As part of the TCB, we can also then implement `SmtEq` for `std` types that have interior mutability but expose a functional interface compatible with smt equality, like `Vec<T>` whenever `T` is also `SmtEq`.
We can then (maybe further down the road) *also* have a `CustomSmtEq` trait that a user can implement to provide the smt equality semantics for a type that *does not* have structural equality and it would be mapped to a different operator, e.g. `a.smt_eq(&b)`; for example, as Chris suggested, we could define `smt_eq` for `Cell`s to be pointer equality, but that won’t imply that the contained values behave functionally (and instead we may rely on a heap, or similar).



I’m working out the details, but I think this is essentially treating everything that’s `SmtEq` the same way we treat linear dafny (`shared`and probably `inout` in function arguments). I’m not yet clear on what the implications are for handling `&mut` mutable references that don’t correspond to a borrow for a function call (like inout).

In particular: would we find it annoying to have to opt in to mathematical equality? (Although one already has to opt-in to equality in rust anyway)

(jayb)

> The opt-in is only at the types' definition, and also everything the standard library that one could feasibly want would already be opted in, so I don't see this being a big deal (one extra word written at types where you'd have to write `#[derive(PartialEq, Eq)]`already). Overall, having it explicit is good.

(tjhance)

>  SmtEq can be thought of as a lemma that `a == b` matches `a.smt_eq(b)`?

Ah interesting, I was thinking of special-casing here for the two “worlds”, but this may be a cleaner approach. So maybe there’s a base trait `E` that defines `.smt_eq`, which one can implement manually, but doesn’t mean mathematical equality of the visible API, and then there’s a derived trait `F` which is `F: E` (F subtype of E) and adds that lemma.

Importantly, `SmtEq` probably means, “the visible interface of this type has mathematical equality”, but the internals may be non-structural (like in `Vec`).

------


(tjhance)

how are you handling Vec anyway?


(Andrea Lattuada)

It’s one of the things I’m trying to figure out, but I think what I’m describing may work. So, if the elements are functional, it can too be functional, because it does **not** expose any of its non-functional internals.


(tjhance)

i mean, i assume you're encoding it as a sequence (ignoring internal structural details) but are you special casing Vec


(Chris Hawblitzel)

I would have assumed that SmtEq means == matches smt_eq, and the Vec would not implement SmtEq, because two Vecs may be == but not smt_eq


(Chris Hawblitzel)

But Seq would implement SmtEq


(Andrea Lattuada)

I **think** two Vecs<T> where `T: SmtEq` are only == if they are also smt_eq. (edited) 


(Chris Hawblitzel)

That depends on the internal implementation, though, right?


(Chris Hawblitzel)

Like the capacity?


(Andrea Lattuada)

True. `capacity` is a visible property that does not have mathematical equality, so yes, it wouldn’t be `SmtEq`.


(Andrea Lattuada)

So maybe the concept of `SmtEq` for runtime things that don’t have structural equality but have a functional interface doesn’t help in practice. (edited) 


(Andrea Lattuada)

For sure `Vec::I()` would get you the corresponding `Seq` that implements `SmtEq`.


(Chris Hawblitzel)

SmtEq sounds useful for things like Pair<A, B>, which would have the == <==> smt_eq property if A and B have this property.


(Andrea Lattuada)

Yea, but `Pair` also has structural equality. So, yea, I still think the trait is a good idea, but it may not be that helpful to allow folks to manually assert it.


(tjhance)

it seems like `InterpEq` (which I just made up) which says that `(a == b) <==> smt_eq(a.I(), b.I())` might be more interesting


(Andrea Lattuada)

This is a bit in the weeds maybe, but it feels like `Vec` is *very close* to having functional equality, and AFAIK `capacity` may essentially be unnecessary for verification (as none of the other commonly used method’s specifications have preconditions on `capacity`).


(Chris Hawblitzel)

I think they both sound useful. I like SmtEq because it allows us to say `==` to mean `smt_eq`. I like InterpEq because it helps us do interesting things with HashSet, etc.


(Andrea Lattuada)

In other words, it doesn’t matter what the `capacity` is to be able to push, index, iterate.


(Andrea Lattuada)

`InterpEq` also suggests there’s a `Interpretation` trait to define `I()`, something like:

```
trait Interp {
  type Interpretation;  #[spec]
  fn I(&self) -> Interpretation;
}
```


(Chris Hawblitzel)

```
trait View { type View; #[spec] fn view(&self) -> View; }
```


(Andrea Lattuada)

And `ViewEq`.


(tjhance)

yeah i really like the idea of an Interp trait, and I've written a bunch of stuff about we should consider making Interp/View first class and just using the interp type in the encoding, outside of private implementation methods


(Andrea Lattuada)

Not having a clear impl/interpretation separation in some of the early data structures we wrote (e.g. `MutableMap`) was and still is the source of confusion and timeouts.


(Andrea Lattuada)

(Maybe of those *I* wrote


(tjhance)

like in veribetrkv, we have a ton of methods that use the hashtable or whatever, and the VCs have all this information about the internals of those datatypes, even though literally every spec is just stuff about `a.I()`


(Andrea Lattuada)

> making Interp/View first class and just using the interp type in the encoding

I think this is along the same line as noticing that `Vec`’s api is essentially functional.


(tjhance)

(yeah, although it hadn't occurred to me that you might want to actually spec out the few methods that *do* interact with the capacity) (edited) 


(Andrea Lattuada)

Indeed. But is that spec actually useful to prove anything? Like, not that this is a good idea, but if you could say “treat `Vec` as mathematical, but the result of `capacity()` is now havoc” would you lose anything of value? (edited) 


(tjhance)

i mean yeah, maybe not


(Andrea Lattuada)

I think you maybe only lose the ability to encode some important program logic into the `capacity` of the `Vec` , which, yea, no.


(tjhance)

well you know the rust documentation doesn't say that `capacity` needs to return the same result each time



---

(tjhance)

The case of objects that use interior mutability but expose an immutable interface could be handled with ghost state. For example, suppose RefCell had an interface that required you to supply ghost state (representing the contents) to use it. Then, in your `List` example, the `List` struct would have another ghost field, and you could build an interpretation function could use the ghost state.



(Chris)

It's also possible that we don't need `pure_eq`. Maybe SMT equality is all we need in specs (it's good enough for Dafny, right?). Then for executable code, we could just let `eq` be nondeterministic in general. If `eq` happens to be pure, then we can write a precise postcondition for it. If it's not pure, then we'd write an imprecise postcondition like `true`, which is perfectly sound. But in both cases, we don't have to write a postcondition that depends on the heap, so we avoid specifying that `eq` depends on the heap.

