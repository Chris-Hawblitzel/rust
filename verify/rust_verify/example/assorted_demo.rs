#![feature(stmt_expr_attributes)]
extern crate builtin;
#[allow(unused_imports)] use builtin::*;
mod pervasive; #[allow(unused_imports)] use pervasive::*;
#[macro_use] extern crate builtin_macros;

fn main() {
    let x = 3;
    let y = 4;
    assert(x != y);
}

#[derive(Eq, PartialEq, Structural)]
struct Train {
    cars: int,
}

fn main2() {
    let t = Train { cars: 10 };
    let q = Train { cars: 12 };
    assert(t != q);
}

#[spec]
fn mul(a: int, b: int)  -> int {
    a * b
}

#[spec]
fn divides(v: int, d: int) -> bool {
    exists(|k: int| mul(d, k) == 0)
}

#[verifier(external)]
fn gcd_external(a: int, b: int) -> int {
    let mut i = a;
    while i >= 1 {
        if a % i == 0 && b % i == 0 {
            break;
        }
    }
    i
}

#[verifier(no_verify)]
fn gcd(a: int, b: int) -> int {
    requires([a >= 0, b >= 0]);
    ensures(|result: int| [divides(a, result), divides(b, result)]);

    gcd_external(a, b)
}

fn main3() {
    let x = 42;
    let y = 182;

    let z = gcd(x, y);

    assert(divides(x, z));
    assert(divides(y, z));
    assert(x % z == 0);
}
