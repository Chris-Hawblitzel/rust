#[macro_export]
macro_rules! unsupported {
    ($msg: expr) => {
        panic!("The verifier does not yet support the following Rust feature: {}", $msg)
    };
    ($msg: expr, $info: expr) => {
        dbg!($info);
        panic!("The verifier does not yet support the following Rust feature: {}", $msg)
    };
    ($msg: expr, $info1: expr, $info2: expr) => {
        dbg!($info1);
        dbg!($info2);
        panic!("The verifier does not yet support the following Rust feature: {}", $msg)
    };
}

#[macro_export]
macro_rules! unsupported_unless {
    ($assertion: expr, $msg: expr) => {
        if (!$assertion) {
            panic!("The verifier does not yet support the following Rust feature: {}", $msg)
        }
    };
    ($assertion: expr, $msg: expr, $info: expr) => {
        if (!$assertion) {
            dbg!($info);
            panic!("The verifier does not yet support the following Rust feature: {}", $msg)
        }
    };
    ($assertion: expr, $msg: expr, $info1: expr, $info2: expr) => {
        if (!$assertion) {
            dbg!($info1);
            dbg!($info2);
            panic!("The verifier does not yet support the following Rust feature: {}", $msg)
        }
    };
}

pub(crate) fn slice_vec_map<A, B, F: Fn(&A) -> B>(slice: &[A], f: F) -> Vec<B> {
    slice.iter().map(f).collect::<Vec<B>>()
}
