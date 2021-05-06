use air::ast::{Ident, Span};
use std::fmt::Debug;
use std::rc::Rc;

/*
In SMT-LIB format (used by Z3), symbols are built of letters, digits, and:
  ~ ! @ $ % ^ & * _ - + = < > . ? /
(although some words, like "pop" and "declare-fun", are reserved words.)
Symbols starting with . or @ are supposed to be reserved for the solver internals.
Z3 seems to like to introduce symbols with !
$ and % and & and ? are probably safe for prefixes and suffixes.
. and @ are safe for suffixes.
AIR uses @ as a suffix for versions of mutable variables (x@0, x@1, ...).

For VIR -> AIR, we use these suffixes:
- globals
    - x.
    - x.y.z
- locals inside functions
    - x@ (works well with AIR's mutable variable convention)
- shadowed locals inside functions
    - x$0, x$1, ...
- bindings inside expressions (e.g. let, forall)
    - x$
Other generated names:
- fuel_x for global name x
*/

// List of constant strings that can appear in generated AIR code
pub const SUFFIX_GLOBAL: &str = ".";
pub const SUFFIX_LOCAL: &str = "@";
pub const SUFFIX_TYPE_PARAM: &str = "&";
pub const TYPE_PATH_SEPARATOR: &str = ".";
pub const VARIANT_SEPARATOR: &str = "/";
pub const PREFIX_FUEL_ID: &str = "fuel%";
pub const PREFIX_REQUIRES: &str = "req%";
pub const PREFIX_ENSURES: &str = "ens%";
pub const FUEL_ID: &str = "FuelId";
pub const FUEL_BOOL: &str = "fuel_bool";
pub const FUEL_BOOL_DEFAULT: &str = "fuel_bool_default";
pub const FUEL_DEFAULTS: &str = "fuel_defaults";
pub const RETURN_VALUE: &str = "%return";
pub const U_HI: &str = "uHi";
pub const I_LO: &str = "iLo";
pub const I_HI: &str = "iHi";
pub const U_CLIP: &str = "uClip";
pub const I_CLIP: &str = "iClip";
pub const NAT_CLIP: &str = "nClip";
pub const U_INV: &str = "uInv";
pub const I_INV: &str = "iInv";
pub const ARCH_SIZE: &str = "SZ";
pub const SNAPSHOT_CALL: &str = "CALL";
pub const POLY: &str = "Poly";
pub const BOX_INT: &str = "I";
pub const BOX_BOOL: &str = "B";
pub const UNBOX_INT: &str = "%I";
pub const UNBOX_BOOL: &str = "%B";
pub const PREFIX_BOX: &str = "Poly%";
pub const PREFIX_UNBOX: &str = "%Poly%";
pub const TYPE: &str = "Type";
pub const TYPE_ID_BOOL: &str = "BOOL";
pub const TYPE_ID_INT: &str = "INT";
pub const TYPE_ID_NAT: &str = "NAT";
pub const TYPE_ID_UINT: &str = "UINT";
pub const TYPE_ID_SINT: &str = "SINT";
pub const PREFIX_TYPE_ID: &str = "TYPE%";

pub fn suffix_global_id(ident: &Ident) -> Ident {
    Rc::new(ident.to_string() + SUFFIX_GLOBAL)
}

pub fn suffix_local_id(ident: &Ident) -> Ident {
    Rc::new(ident.to_string() + SUFFIX_LOCAL)
}

pub fn suffix_typ_param_id(ident: &Ident) -> Ident {
    Rc::new(ident.to_string() + SUFFIX_TYPE_PARAM)
}

pub fn prefix_type_id(ident: &Ident) -> Ident {
    Rc::new(PREFIX_TYPE_ID.to_string() + ident)
}

pub fn prefix_box(ident: &Ident) -> Ident {
    Rc::new(PREFIX_BOX.to_string() + ident)
}

pub fn prefix_unbox(ident: &Ident) -> Ident {
    Rc::new(PREFIX_UNBOX.to_string() + ident)
}

pub fn prefix_fuel_id(ident: &Ident) -> Ident {
    Rc::new(PREFIX_FUEL_ID.to_string() + ident)
}

pub fn prefix_requires(ident: &Ident) -> Ident {
    Rc::new(PREFIX_REQUIRES.to_string() + ident)
}

pub fn prefix_ensures(ident: &Ident) -> Ident {
    Rc::new(PREFIX_ENSURES.to_string() + ident)
}

pub struct Spanned<X> {
    pub span: Span,
    pub x: X,
}

impl<X> Spanned<X> {
    pub fn new(span: Span, x: X) -> Rc<Spanned<X>> {
        Rc::new(Spanned { span: span, x: x })
    }
}

impl<X: Debug> Debug for Spanned<X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_tuple("Spanned").field(&self.span.as_string).field(&self.x).finish()
    }
}
