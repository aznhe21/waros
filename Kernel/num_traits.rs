use prelude::*;
use core::num::{Zero, One};
use core::ops::{Add, Sub, Mul, Div, Rem, Neg};

macro_rules! empty_trait_impl {
    ($name:ident for $($t:ty)*) => ($(
            impl $name for $t {
            }
    )*)
}

pub trait Num: PartialEq + Zero + One
    + Add<Output = Self> + Sub<Output = Self>
    + Mul<Output = Self> + Div<Output = Self> + Rem<Output = Self>
{
}

empty_trait_impl!(Num for usize u8 u16 u32 u64 isize i8 i16 i32 i64);
empty_trait_impl!(Num for f32 f64);

pub trait Signed: Num + Neg<Output = Self> {
}

empty_trait_impl!(Signed for isize i8 i16 i32 i64);

pub trait Unsigned: Num {}

empty_trait_impl!(Unsigned for usize u8 u16 u32 u64);

