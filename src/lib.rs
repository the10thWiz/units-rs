#![cfg_attr(nightly, feature(trait_specialization))]
pub mod base;
pub mod prefix;

use std::{fmt::Debug, marker::PhantomData, ops::Div};

use base::Unitless;
use prefix::Prefix;
use private::Sealed;
use typenum::{Integer, ToInt};

mod private {
    pub trait Sealed {}
    impl<T> Sealed for T {}
}

pub trait Unit: Copy + Sealed {
    type Base;
    fn create() -> Self;
    // fn conversion() -> Op;
}
pub trait BaseUnit: Unit {}

#[derive(Clone, Copy)]
pub struct Value<V, P: Integer, U: Unit>(V, Prefix<P, U>);

impl<V, P: Integer, U: Unit> Value<V, P, U> {
    pub fn new(v: V) -> Self {
        Self(v, Prefix(U::create(), PhantomData))
    }

    pub fn value(&self) -> &V {
        &self.0
    }
}

pub trait UnitValue: Copy {
    fn apply_prefix(&self, power_of_ten: i32) -> Self;
}

impl UnitValue for f64 {
    fn apply_prefix(&self, power_of_ten: i32) -> Self {
        self * 10f64.powi(power_of_ten)
    }
}

impl<V: UnitValue, P: Integer + ToInt<i32>, U: Unit> Value<V, P, U> {
    pub fn convert<TargetP: Integer + ToInt<i32>, TargetU: Unit>(
        &self,
    ) -> Value<V, TargetP, TargetU>
    where
        U: Div<TargetU, Output = Unitless>,
    {
        let tmp = Prefix(TargetU::create(), PhantomData);
        Value(self.1.convert(self.0, &tmp), tmp)
    }
}

// TODO: this conflicts with the blanket `From<Self> for Self` impl in std
#[cfg(nightly)]
impl<
        V: UnitValue,
        P: Integer + ToInt<i32>,
        U: Unit,
        TargetP: Integer + ToInt<i32>,
        TargetU: Unit,
    > From<Value<V, P, U>> for Value<V, TargetP, TargetU>
where
    U: Div<TargetU, Output = Unitless>,
{
    fn from(value: Value<V, P, U>) -> Value<V, TargetP, TargetU> {
        let tmp = Prefix(TargetU::create(), PhantomData);
        Value(value.1.convert(self.0, &tmp), tmp)
    }
}

impl<
        LhsV: UnitValue,
        RhsV,
        LhsP: Integer + ToInt<i32>,
        RhsP: Integer + ToInt<i32>,
        LhsU: Unit,
        RhsU: Unit,
    > PartialEq<Value<RhsV, RhsP, RhsU>> for Value<LhsV, LhsP, LhsU>
where
    LhsV: PartialEq<RhsV>,
    LhsU: Div<RhsU, Output = Unitless>,
{
    fn eq(&self, other: &Value<RhsV, RhsP, RhsU>) -> bool {
        self.convert::<RhsP, RhsU>().value().eq(other.value())
    }
}

impl<V: Debug, P: Integer + ToInt<i32>, U: Unit + Debug> Debug for Value<V, P, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?}", self.0, self.1)
    }
}

#[test]
fn test_from() {
    use crate::{base::Meter, prefix::Centi};
    // assert_eq!(
    //     Centi::<Meter>::new(100f64),
    //     Meter::new(1f64).convert()
    // );
    assert_eq!(
        Centi::<Meter>::new(100f64),
        Meter::new(1).into()
        // Note that the order here matters for rust to select the correct generic type
        // automatically
    );
}

// // #[derive(Debug, Clone, Copy)]
// // pub struct Unitless;
// // impl Unit for Unitless {
// //     type Base = Self;
// //     fn create() -> Self { Unitless }
// //     fn conversion() -> Op { Op::Mult(1.0) }
// // }
// // impl BaseUnit for Unitless {}
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum Op {
//     Mult(f64),
//     Div(f64),
//     Add(f64),
//     Sub(f64),
//     Pair(Box<Self>, Box<Self>),
// }
//
// impl Op {
//     fn combine(self, other: Self) -> Self {
//         Self::Pair(Box::new(self), Box::new(other))
//     }
//
//     fn convert(&self, value: f64) -> f64 {
//         match self {
//             Self::Mult(m) => value * m,
//             Self::Div(m) => value / m,
//             Self::Add(m) => value + m,
//             Self::Sub(m) => value - m,
//             Self::Pair(a, b) => b.convert(a.convert(value)),
//         }
//     }
//
//     fn inverse(self) -> Self {
//         match self {
//             Self::Mult(m) => Self::Div(m),
//             Self::Div(m) => Self::Mult(m),
//             Self::Add(m) => Self::Sub(m),
//             Self::Sub(m) => Self::Add(m),
//             Self::Pair(a, b) => Self::Pair(Box::new(b.inverse()), Box::new(a.inverse())),
//         }
//     }
// }
//
macro_rules! unit_types {
    ($($name:ident),+) => {
        $(
            pub trait $name: Unit {
                type TypedBase: $name;
            }
        )+
    };
}
unit_types!(
    Length,
    Time,
    Mass,
    Amount,
    Current,
    Tempature,
    LuminousIntesity
);
//
// macro_rules! impl_inheirt {
//     ($name:ident) => {
//         impl<T: BaseUnit + Length> Length for $name<T> {
//             type TypedBase = Self::Base;
//         }
//         impl<T: BaseUnit + Time> Time for $name<T> {
//             type TypedBase = Self::Base;
//         }
//     };
// }
//
// macro_rules! base_unit {
//     ($name:ident : $($ty:ident),*) => {
//         #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//         pub struct $name();
//         impl Unit for $name {
//             type Base = Self;
//             fn create() -> Self {
//                 Self()
//             }
//
//             fn conversion() -> Op {
//                 Op::Mult(1f64)
//             }
//         }
//
//         impl BaseUnit for $name {}
//
//         $(impl $ty for $name {
//             type TypedBase = Self::Base;
//         })*
//
//         impl $name {
//             pub fn new(val: impl Into<f64>) -> Value<Self> {
//                 Value::new(val.into())
//             }
//         }
//     };
// }
//
// macro_rules! metric_prefix {
//     ($name:ident => * $num:literal) => {
//         metric_prefix!($name => Op::Mult($num));
//     };
//     ($name:ident => / $num:literal) => {
//         metric_prefix!($name => Op::Div($num));
//     };
//     ($name:ident => + $num:literal) => {
//         metric_prefix!($name => Op::Add($num));
//     };
//     ($name:ident => * $num2:literal + $num:literal) => {
//         metric_prefix!($name => Op::Mult($num2).combine(Op::Add($num)));
//     };
//     ($name:ident => $conv:expr) => {
//         #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//         pub struct $name<T: BaseUnit>(T);
//         impl<T: BaseUnit> Unit for $name<T> {
//             type Base = T;
//             fn create() -> Self {
//                 Self(T::create())
//             }
//
//             fn conversion() -> Op {
//                 $conv
//             }
//         }
//
//         impl<T: BaseUnit> $name<T> {
//             pub fn new(val: impl Into<f64>) -> Value<Self> {
//                 Value::new(val.into())
//             }
//         }
//         impl_inheirt!($name);
//     };
// }
//
// macro_rules! other_unit {
//     ($name:ident: $base:ident $(+ $ty:ident)* => * $num:literal) => {
//         other_unit!($name: $base $(+ $ty)* => Op::Mult($num));
//     };
//     ($name:ident: $base:ident $(+ $ty:ident)* => / $num:literal) => {
//         other_unit!($name: $base $(+ $ty)* => Op::Div($num));
//     };
//     ($name:ident: $base:ident $(+ $ty:ident)* => - $num:literal) => {
//         other_unit!($name: $base $(+ $ty)* => Op::Sub($num));
//     };
//     ($name:ident: $base:ident $(+ $ty:ident)* => * $num2:literal + $num:literal) => {
//         other_unit!($name: $base $(+ $ty)* => Op::Mult($num2).combine(Op::Add($num)));
//     };
//     ($name:ident: $base:ident $(+ $ty:ident)* => $conv:expr) => {
//         #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//         pub struct $name();
//         impl Unit for $name {
//             type Base = $base;
//             fn create() -> Self {
//                 Self()
//             }
//
//             fn conversion() -> Op {
//                 $conv
//             }
//         }
//
//         $(impl $ty for $name {
//             type TypedBase = Self::Base;
//         })*
//
//         impl $name {
//             pub fn new(val: impl Into<f64>) -> Value<Self> {
//                 Value::new(val.into())
//             }
//         }
//     };
// }
//
// macro_rules! multi_unit {
//     ($name:ident => $g0:ident: $base0:ident $(* $g:ident: $base:ident)* $(/ $gl:ident: $basel:ident)*) => {
//         #[derive(Debug, Clone, Copy)]
//         pub struct $name<$g0: $base0 $(, $g: $base)* $(, $gl: $basel)*>($g0 $(, $g)* $(, $gl)*);
//
//         impl<$g0: $base0 $(, $g: $base)* $(, $gl: $basel)*> Unit for $name<$g0 $(, $g)* $(, $gl)*> {
//             type Base = $name<$g0::TypedBase $(, $g::TypedBase)* $(, $gl::TypedBase)*>;
//             fn create() -> Self {
//                 Self($g0::create() $(, $g::create())* $(, $gl::create())*)
//             }
//
//             fn conversion() -> Op {
//                 $g0::conversion()
//                     $(.combine($g::conversion()))*
//                     $(.combine($gl::conversion().inverse()))*
//             }
//         }
//
//         impl<$g0: $base0 + BaseUnit $(, $g: $base + BaseUnit)* $(, $gl: $basel + BaseUnit)*> BaseUnit for $name<$g0 $(, $g)* $(, $gl)*> {
//         }
//
//         impl<$g0: $base0  $(, $g: $base )* $(, $gl: $basel )*> $name<$g0 $(, $g)* $(, $gl)*> {
//             pub fn new(val: impl Into<f64>) -> Value<Self> {
//                 Value::new(val.into())
//             }
//
//             //pub fn from()
//         }
//     };
// }
//
// #[derive(Debug, Clone, Copy)]
// pub struct Area<U: Length>(U);
//
// impl<U: Length> Unit for Area<U> {
//     type Base = Area<U::TypedBase>;
//     fn create() -> Self {
//         Self(U::create())
//     }
//     fn conversion() -> Op {
//         U::conversion().combine(U::conversion())
//     }
// }
//
// impl<U: Length + BaseUnit> BaseUnit for Area<U> {}
//
// impl<U: Length> Area<U> {
//     pub fn new(val: impl Into<f64>) -> Value<Self> {
//         Value::new(val.into())
//     }
// }
//
// #[derive(Debug, Clone, Copy)]
// pub struct Volume<U: Length>(U);
//
// impl<U: Length> Unit for Volume<U> {
//     type Base = Volume<U::TypedBase>;
//     fn create() -> Self {
//         Self(U::create())
//     }
//     fn conversion() -> Op {
//         U::conversion()
//             .combine(U::conversion())
//             .combine(U::conversion())
//     }
// }
//
// impl<U: Length + BaseUnit> BaseUnit for Volume<U> {}
//
// impl<U: Length> Volume<U> {
//     pub fn new(val: impl Into<f64>) -> Value<Self> {
//         Value::new(val.into())
//     }
// }
//
// base_unit!(Second: Time);
// base_unit!(Meter: Length);
// base_unit!(Gram: Mass);
// base_unit!(Ampere: Current);
// base_unit!(Kelvin: Tempature);
// base_unit!(Mole: Amount);
// base_unit!(Candela: LuminousIntesity);
//
// other_unit!(Celsius: Kelvin + Tempature => - 213f64);
//
// multi_unit!(Charge => A: Current * B: Time);
// pub type Coulomb = Charge<Ampere, Second>;
//
// multi_unit!(Catalytic => A: Amount / B: Time);
// pub type Kat = Catalytic<Mole, Second>;
//
// multi_unit!(Force => A: Mass * B: Length / C: Time / D: Time);
// pub type Newton = Force<Kilo<Gram>, Meter, Second, Second>;
//
// multi_unit!(Energy => A: Mass * B: Length * C: Length / D: Time / E: Time);
// pub type Joule = Energy<Kilo<Gram>, Meter, Meter, Second, Second>;
//
// // Note: commeted out lines are too large (or small) for f64 values
// //metric_prefix!(Yotta  => / 1000000000000000000000000);
// //metric_prefix!(Zetta  => / 1000000000000000000000);
// metric_prefix!(Exa    => / 1000000000000000000f64);
// metric_prefix!(Peta   => / 1000000000000000f64);
// metric_prefix!(Tera   => / 1000000000000f64);
// metric_prefix!(Giga   => / 1000000000f64);
// metric_prefix!(Mega   => / 1000000f64);
// metric_prefix!(Kilo   => / 1000f64);
// metric_prefix!(Hecta  => / 100f64);
// metric_prefix!(Deka   => / 10f64);
// metric_prefix!(Deci   => * 10f64);
// metric_prefix!(Centi  => * 100f64);
// metric_prefix!(Milli  => * 1000f64);
// metric_prefix!(Micro  => * 1000000f64);
// metric_prefix!(Nano   => * 1000000000f64);
// metric_prefix!(Pico   => * 1000000000000f64);
// metric_prefix!(Fempto => * 1000000000000000f64);
// metric_prefix!(Atto   => * 1000000000000000000f64);
// //metric_prefix!(Zepto  => * 1000000000000000000000);
// //metric_prefix!(Yocto  => * 1000000000000000000000000);
//
// #[derive(Debug, Clone, Copy)]
// pub struct Value<U: Unit>(f64, U);
//
// impl<U: Unit> Value<U> {
//     pub fn new(val: impl Into<f64>) -> Self {
//         Self(val.into(), U::create())
//     }
// }
//
// impl<B: BaseUnit + Unit, U: Unit<Base = B>> Value<U> {
//     fn convert<T: Unit<Base = B>>(self) -> Value<T> {
//         let val = U::conversion()
//             .inverse()
//             .combine(T::conversion())
//             .convert(self.0);
//         Value(val, T::create())
//     }
// }
//
// impl<B: BaseUnit + Unit, U: Unit<Base = B>> Value<U> {
//     fn add<R: Unit<Base = B>, T: Unit<Base = B>>(self, rhs: Value<T>) -> Value<R> {
//         let s = self.convert::<R>().0;
//         let o = rhs.convert::<R>().0;
//         Value(s + o, R::create())
//     }
// }
//
// impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::ops::Add<Value<T>>
//     for Value<U>
// {
//     type Output = Self;
//     fn add(self, rhs: Value<T>) -> Self {
//         self.add(rhs)
//     }
// }
//
// impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::ops::AddAssign<Value<T>>
//     for Value<U>
// {
//     fn add_assign(&mut self, rhs: Value<T>) {
//         self.0 += rhs.convert::<U>().0;
//     }
// }
//
// impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::cmp::PartialEq<Value<T>>
//     for Value<U>
// {
//     fn eq(&self, other: &Value<T>) -> bool {
//         self.convert::<T>().0 == other.0
//     }
// }
//
// impl<Lhs, Rhs> std::ops::Mul<Value<Rhs>> for Value<Lhs>
// where
//     Lhs: Length,
//     Rhs: Length,
// {
//     type Output = Value<Area<Lhs::TypedBase>>;
//     fn mul(self, rhs: Value<Rhs>) -> Self::Output {
//         let s = Lhs::conversion().convert(self.0);
//         let o = Rhs::conversion().convert(rhs.0);
//         Value(s * o, Area::create())
//     }
// }
//
// impl<Lhs, Rhs> std::ops::Mul<Value<Area<Rhs>>> for Value<Lhs>
// where
//     Lhs: Length,
//     Rhs: Length,
// {
//     type Output = Value<Volume<Lhs::TypedBase>>;
//     fn mul(self, rhs: Value<Area<Rhs>>) -> Self::Output {
//         let s = Lhs::conversion().convert(self.0);
//         let o = Rhs::conversion().convert(rhs.0);
//         Value(s * o, Volume::create())
//     }
// }
//
// impl<Lhs, Rhs> std::ops::Mul<Value<Rhs>> for Value<Area<Lhs>>
// where
//     Lhs: Length,
//     Rhs: Length,
// {
//     type Output = Value<Volume<Lhs::TypedBase>>;
//     fn mul(self, rhs: Value<Rhs>) -> Self::Output {
//         let s = Lhs::conversion().convert(self.0);
//         let o = Rhs::conversion().convert(rhs.0);
//         Value(s * o, Volume::create())
//     }
// }
//
// pub mod common {
//     use super::*;
//
//     pub type NanoMeter = Nano<Meter>;
//     pub type MicroMeter = Micro<Meter>;
//     pub type MilliMeter = Milli<Meter>;
//     pub type CentiMeter = Centi<Meter>;
//     pub type KiloMeter = Kilo<Meter>;
//
//     pub type SqMeter = Area<Meter>;
//     pub type SqKiloMeter = Area<Kilo<Meter>>;
// }
