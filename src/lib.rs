//! Strong aliases for [`typenum`](https://lib.rs/typenum), powered by const generics. Makes compilation errors more readable.
//!
//! ### Motivation
//! `typenum` defines [convenient type aliases](https://docs.rs/typenum/latest/typenum/consts/index.html) for frequently used numbers.
//! Unfortunately, `rustc` & `rust-analyzer` expand them into their full binary representation, e. g. [`typenum::U10`](https://docs.rs/typenum/latest/typenum/consts/type.U10.html) is expanded to this:
//! ```rust
//! pub type U10 = UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>;
//! ```
//!
//! For bigger numbers it's even longer. There's <https://github.com/rust-lang/rust-analyzer/issues/1666>, which will hopefully solve the problem someday.
//!
//! `typenum_alias` aims to provide a temporary solution which works on stable Rust using only `min_const_generics`.
//!
//! ### How it works
//! `typenum_alias` defines `struct Const<const N: i32>`, `trait ToTypenum` & `trait ToConst`. Arithmetical operations are implemented for `Const<N>` in the following way:
//! 1. `Const<N>` is converted to `PInt`, `NInt` or `Z0`
//! 2. `typenum` performs the calculations
//! 3. The result is converted back to `Const<N>`
//!
//! Thanks to this technique, `UInt<UInt<UInt<UInt<UTerm, B1>, B0>, B1>, B0>` becomes just `Const<10_i32>`.
//! You can shorten it even more to `Const<10>` either by using latest nightly, which already contains the fix
//! (<https://github.com/rust-lang/rust/pull/99393>), or by waiting for `1.64.0` stable release of Rust.

#![deny(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]

#[doc(no_inline)]
pub use typenum::{consts, operator_aliases, type_operators};

use core::ops::{Add, Div, Mul, Sub};
use paste::paste;
use std::ops::Neg;
use typenum::{
    consts::*, operator_aliases::*, private::InternalMarker, type_operators::*, Diff, Integer,
    Negate, Prod, Quot, Sum,
};

#[derive(Default, Clone, Copy)]
pub struct Const<const N: i32>;

pub trait ToTypenum {
    type Output: Integer;
}

pub type Typenum<N> = <N as ToTypenum>::Output;

pub trait ToConst {
    type Output: Default;
}

pub type Constant<T> = <T as ToConst>::Output;

impl ToTypenum for Const<0> {
    type Output = Z0;
}

impl ToConst for Z0 {
    type Output = Const<0>;
}

macro_rules! const_conversion {
    ($($num:literal),+) => {
        $(impl ToTypenum for Const<$num> {
            type Output = paste!([<P $num>]);
        }

        impl ToConst for paste!([<P $num>]) {
            type Output = Const<$num>;
        }

        impl ToTypenum for Const<-$num> {
            type Output = paste!([<N $num>]);
        }

        impl ToConst for paste!([<N $num>]) {
            type Output = Const<-$num>;
        })+
    };
}

macro_rules! impl_binary_ops_for_const {
    ($(($op:ident, $out:ident, $fn:ident),)+) => {
        $(impl<const L: i32, const R: i32> $op<Const<R>> for Const<L>
        where
            Const<L>: ToTypenum,
            Const<R>: ToTypenum,
            Typenum<Const<L>>: $op<Typenum<Const<R>>>,
            $out<Typenum<Const<L>>, Typenum<Const<R>>>: ToConst,
        {
            type Output = Constant<$out<Typenum<Const<L>>, Typenum<Const<R>>>>;

            #[inline]
            fn $fn(self, _: Const<R>) -> Self::Output {
                Self::Output::default()
            }
        })+
    };

    ($(($op:ident, $out:ident),)+) => {
        $(impl<const L: i32, const R: i32> $op<Const<R>> for Const<L>
        where
            Const<L>: ToTypenum,
            Const<R>: ToTypenum,
            Typenum<Const<L>>: $op<Typenum<Const<R>>>,
            $out<Typenum<Const<L>>, Typenum<Const<R>>>: ToConst,
        {
            type Output = Constant<$out<Typenum<Const<L>>, Typenum<Const<R>>>>;
        })+
    };
}

macro_rules! impl_unary_ops_for_const {
    ($(($op:ident, $out:ident, $fn:ident),)+) => {
        $(impl<const N: i32> $op for Const<N>
        where
            Const<N>: ToTypenum,
            Typenum<Const<N>>: $op,
            $out<Typenum<Const<N>>>: ToConst,
        {
            type Output = Constant<$out<Typenum<Const<N>>>>;

            #[inline]
            fn $fn(self) -> Self::Output {
                Self::Output::default()
            }
        })+
    };
    ($(($op:ident, $out:ident),)+) => {
        $(impl<const N: i32> $op for Const<N>
        where
            Const<N>: ToTypenum,
            Typenum<Const<N>>: $op,
            $out<Typenum<Const<N>>>: ToConst,
        {
            type Output = Constant<$out<Typenum<Const<N>>>>;
        })+
    };
}

// TODO: use build.rs to generate this
// TODO: put different ranges under feature gates
const_conversion! {
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16
}

// Binary ops ---------------------------------------------

impl_binary_ops_for_const! {
    (Add, Sum,  add),
    (Sub, Diff, sub),
    (Mul, Prod, mul),
    (Div, Quot, div),
    (Max, Maximum, max),
    (Min, Minimum, min),
    (PartialDiv, PartialQuot, partial_div),
}

impl_binary_ops_for_const! {
    (Gcd, Gcf),
}

// FIXME: report this false-positive to `clippy`
#[allow(clippy::trait_duplication_in_bounds)]
impl<const L: i32, const R: i32> Cmp<Const<R>> for Const<L>
where
    Const<L>: ToTypenum,
    Const<R>: ToTypenum,
    Typenum<Const<L>>: Cmp<Typenum<Const<R>>>,
    Compare<Typenum<Const<L>>, Typenum<Const<R>>>: Default,
{
    type Output = Compare<Typenum<Const<L>>, Typenum<Const<R>>>;

    #[inline]
    fn compare<IM: InternalMarker>(&self, _: &Const<R>) -> Self::Output {
        Self::Output::default()
    }
}

// Unary ops ---------------------------------------------

impl_unary_ops_for_const! {
    (Neg, Negate, neg),
}

impl_unary_ops_for_const! {
    (Abs, AbsVal),
    (Logarithm2, Log2),
    (SquareRoot, Sqrt),
}
