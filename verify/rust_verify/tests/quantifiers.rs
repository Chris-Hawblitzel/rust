#![feature(rustc_private)]
#![feature(stmt_expr_attributes)]
#[macro_use]
mod common;
use common::*;

test_verify_with_pervasive! {
    #[test] test1 code! {
        #[spec]
        fn tr(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr(300));
            assert(exists(|i: nat| i >= 0 && tr(i)));
        }
    } => Ok(())
}

test_verify_with_pervasive! {
    #[test] test1_fails code! {
        #[spec]
        fn tr(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(exists(|i: nat| i >= 0 && tr(i))); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_with_pervasive! {
    #[test] test2 code! {
        #[spec]
        fn tr1(i: int) -> bool {
            true
        }

        #[spec]
        fn tr2(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr1(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && tr2(i)));
        }
    } => Ok(())
}

test_verify_with_pervasive! {
    #[test] test3 code! {
        #[spec]
        fn tr1(i: int) -> bool {
            true
        }

        #[spec]
        fn tr2(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr2(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && tr2(i)));
        }
    } => Ok(())
}

////

test_verify_with_pervasive! {
    #[test] test1g code! {
        #[spec]
        fn tr<A>(a: A) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr(300));
            assert(exists(|i: nat| i >= 0 && tr(i)));
        }
    } => Ok(())
}

test_verify_with_pervasive! {
    #[test] test1g_fails code! {
        #[spec]
        fn tr<A>(a: A) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(exists(|i: nat| i >= 0 && tr(i))); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}

test_verify_with_pervasive! {
    #[test] test2g code! {
        #[spec]
        fn tr1<A>(a: A) -> bool {
            true
        }

        #[spec]
        fn tr2<A>(a: A) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr1(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && tr2(i)));
        }
    } => Ok(())
}

test_verify_with_pervasive! {
    #[test] test3g code! {
        #[spec]
        fn tr1<A>(a: A) -> bool {
            true
        }

        #[spec]
        fn tr2<A>(a: A) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr2(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && tr2(i)));
        }
    } => Ok(())
}

////

/* REVIEW: these tests need #![feature(stmt_expr_attributes)], which doesn't seem to work here
test_verify_with_pervasive! {
    #[test] test4 code! {
        #[spec]
        fn tr1(i: int) -> bool {
            true
        }

        #[spec]
        fn tr2(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr2(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && #[trigger]tr2(i)));
        }
    } => Ok(())
}

test_verify_with_pervasive! {
    #[test] test4_fails code! {
        #[spec]
        fn tr1(i: int) -> bool {
            true
        }

        #[spec]
        fn tr2(i: int) -> bool {
            true
        }

        #[proof]
        fn test1() {
            assert(tr1(300));
            assert(exists(|i: nat| i >= 0 && tr1(i) && #[trigger]tr2(i))); // FAILS
        }
    } => Err(err) => assert_one_fails(err)
}
*/
