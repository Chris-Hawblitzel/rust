personally, I like the idea of `view` being pure (i.e., no `reads`-clause-equivalent) and taking advantage of the borrowing system to know that `a.view()` is constant for some shared-borrow `a: &T`



(in part, this is because a lot of the stuff I've been doing hinges on this idea)

but also, if I understand right, the motivation for this `View` trait is for hiding implementation details from the smt VCs right? It seems going to have a lot harder time of that if it isn't a pure function

like, if `view` were not a pure function, we wouldn't be able to just say "okay well we can just replace the variable a by a.view() everywhere in the VCs"

On the other hand, if we do say `view` is impure, then we could represent it in the smt encoding as a function taking a "heap" argument (like Dafny): `view(t, heap)`. Then the "havocing" happens automatically whenever "heap" is havoc'ed

in other words, I don't think the quesiton is so much "when do we havoc a view" but rather "how do we specify modifications of the heap, in general?"

but again, my feeling is that we should be encouraging "pure" verification by default, since we have more techniques now for actually making this work

for example, with ghost state, we could do an "interior mutability but with an externally immutable interface" object with no specification of heap dependence, with a pure "view" function, and so on ... I should probably actually write up this example so you know what I'm talking about, shouldn't I (edited)

here: https://github.com/Chris-Hawblitzel/rust/wiki/externally_immutable.rs

![img](https://ca.slack-edge.com/T7K7R6W2W-U011U58V81M-1a52450bbf56-48)

**[Andrea Lattuada](https://app.slack.com/team/U011U58V81M)**[13 days ago](https://securefoundations.slack.com/archives/C012YMS0U00/p1623860926197500?thread_ts=1623780220.195200&cid=C012YMS0U00)

Thanks for the notes! It’s super helpful to get this kind of feedback, as I’m sure I’m gonna miss some of the aspects of the design otherwise.

> but also, if I understand right, the motivation for this `View` trait is for hiding implementation details from the smt VCs right?

Yes. I’m still figuring out of the mechanism: it’s possible that it turns out easier to not replace `a` with `a.view()` everywhere, but just have `a` be an opaque variable and have `(view a)` carry facts (but I need to write up a couple tests to make sure I understand the semantics of the `air`/`z3` encoding). (edited) 

> my feeling is that we should be encouraging "pure" verification by default, since we have more techniques now for actually making this work

This is something I was trying to decide on, and I think I agree. Especially now that you’ve demonstrated that we can still have interior mutability with a pure `view` and interior mutability. In the context of the doc the `DoublyLinkedList` would be marked `Immutable`, which would let you specify a `View`. The point of `Immutable` is also to allow one to write “external” specifications for libraries that aren’t dust (like the standard library), so you can assert that a `Vec` has a pure view, and then write the spec in terms of `view` 

doing the replacement everywhere is kind of tricky and there are a bunch of properties that need to hold; for example if there is a (public) pure function `a -> foo` and you want to replace `a` with `view a`everywhere, then that pure function needs to satisfy `view x == view y ==> f x == f y

https://securefoundations.slack.com/archives/C012YMS0U00/p1623861465198400?thread_ts=1623780220.195200&cid=C012YMS0U00)

> feeling is that we should be encouraging "pure" verification by default, since we have more techniques now for actually making this work

Which maybe means that implementing `View` is already asserting purity, no need for a separate `Immutable` trait. And either (a) this is dust type and it will be enforced by the verifier, or (b) this is an external type and the `View` implementation is in the TCB. 

Of course `Structural` (which will likely replace `StructEq` with the stronger property that (a) all the fields are `Structural` (or native), and both equality and clone are consistent with smt equality) already implies purity.

it's gotta be in the TCB right? because to do this implementation-hiding, the encoder *has* to know about this special View type, right?

No, when you implement `View` for a dust `Structural` type it’s enforced by the type system + verifier. 

(The *definition* of the `View` trait is in the TCB, yes)(yeah, that's what I meant)

>  doing the replacement everywhere is kind of tricky and there are a bunch of properties that need to hold

Which is maybe areason to not do the replacement?

>  if there is a (public) pure function `a -> foo`

To verify its body this we’d need to look inside `a` and `foo`, though, and then possibly describe its spec in terms of the respective views.

Which makes me think of invariants. I guess we can start with just having a public predicate, and then if we get sick of writing requires(self.Inv()) ensures(self.Inv()) we can add an `Invariant` trait:

```
trait Inv {
  #[spec]
  fn inv(&self) -> bool;
}
```

