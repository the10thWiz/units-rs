use std::{ops::{Div, Mul}, fmt::Debug};
use typenum::op;

pub use crate::{BaseUnit, Unit, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unitless;

impl Unit for Unitless {
    type Base = Self;
    fn create() -> Self {
        Self
    }
    // fn conversion() -> Op {
    //     Op::Mult(1f64)
    // }
}
// impl BaseUnit for Unitless {}

impl<U: BaseUnit> Mul<U> for Unitless {
    type Output = U;
    fn mul(self, rhs: U) -> Self::Output {
        rhs
    }
}
impl<U: BaseUnit> Div<U> for Unitless {
    type Output = Inverse<U>;
    fn div(self, rhs: U) -> Self::Output {
        Inverse(rhs)
    }
}
impl Unitless {
    pub fn new(val: impl Into<f64>) -> Value<f64, typenum::Z0, Self> {
        Value::new(val.into())
    }
}

macro_rules! base_unit {
    ($name:ident : $($ty:ident),*) => {
        #[derive(Clone, Copy, PartialEq, Eq)]
        pub struct $name;
        impl Unit for $name {
            type Base = Self;
            fn create() -> Self {
                Self
            }

            // fn conversion() -> Op {
            //     Op::Mult(1f64)
            // }
        }

        impl BaseUnit for $name {}

        $(impl crate::$ty for $name {
            type TypedBase = Self::Base;
        })*

        impl<U: BaseUnit> Mul<U> for $name {
            type Output = Mult<$name, U>;
            fn mul(self, rhs: U) -> Self::Output {
                Mult(self, rhs)
            }
        }

        impl Mul<Unitless> for $name {
            type Output = $name;
            fn mul(self, _rhs: Unitless) -> Self::Output {
                self
            }
        }

        impl Mul<Inverse<Self>> for $name {
            type Output = Unitless;
            fn mul(self, _rhs: Inverse<Self>) -> Self::Output {
                Unitless
            }
        }

        impl Div<$name> for $name {
            type Output = Unitless;
            fn div(self, _rhs: Self) -> Self::Output {
                Unitless
            }
        }

        impl<U: BaseUnit> Div<Inverse<U>> for $name
            where Self: Mul<U>
        {
            type Output = op!(Self * U);
            fn div(self, rhs: Inverse<U>) -> Self::Output {
                self * rhs.0
            }
        }

        impl $name {
            pub fn new(val: impl Into<f64>) -> Value<f64, typenum::Z0, Self> {
                Value::new(val.into())
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!(stringify!($name), "s"))
            }
        }
    };
}

base_unit!(Second: Time);
base_unit!(Meter: Length);
base_unit!(Gram: Mass);
base_unit!(Ampere: Current);
base_unit!(Kelvin: Tempature);
base_unit!(Mole: Amount);
base_unit!(Candela: LuminousIntesity);

pub trait Invert: Unit {
    type Inverse: Unit;
    fn invert(self) -> Self::Inverse;
}

// represents 1/U
#[derive(Clone, Copy)]
pub struct Inverse<U: BaseUnit>(U);
impl<U: BaseUnit> Unit for Inverse<U> {
    type Base = U;

    fn create() -> Self {
        Self(U::create())
    }
}

impl<U: BaseUnit + Debug> Debug for Inverse<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "1/{:?}", self.0)
    }
}

impl<U: BaseUnit> Invert for Inverse<U> {
    type Inverse = U;
    fn invert(self) -> Self::Inverse {
        self.0
    }
}

impl<U: BaseUnit> Invert for U {
    type Inverse = Inverse<U>;
    fn invert(self) -> Self::Inverse {
        Inverse(self)
    }
}

// Multiply: Inv(U) * Rhs => Rhs * Inv(U)
impl<Rhs, U: BaseUnit> Mul<Rhs> for Inverse<U>
where
    Rhs: Mul<Self>,
{
    type Output = op!(Rhs * Self);
    fn mul(self, rhs: Rhs) -> Self::Output {
        rhs * self
    }
}

#[derive(Clone, Copy)]
pub struct Mult<U, V>(U, V);

impl<U: Unit, V: Unit> Unit for Mult<U, V> {
    type Base = Mult<U::Base, V::Base>;
    fn create() -> Self {
        Self(U::create(), V::create())
    }
}

impl<U: Debug, V: Debug> Debug for Mult<U, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} * {:?}", self.0, self.1)
    }
}

impl<U: Unit + Invert, V: Unit + Invert> Invert for Mult<U, V> {
    type Inverse = Mult<U::Inverse, V::Inverse>;

    fn invert(self) -> Self::Inverse {
        Mult(self.0.invert(), self.1.invert())
    }
}

// Multiply: (U*V) * W => ((U*W)*V)
// Recurses to base case - U is a base unit (or Inverse)
impl<U: Unit, V: BaseUnit, W: BaseUnit> Mul<W> for Mult<U, V>
where
    U: Mul<W>,
{
    type Output = Mult<op!(U * W), V>;
    fn mul(self, rhs: W) -> Self::Output {
        Mult(self.0 * rhs, self.1)
    }
}

// Divide (U*V) / W => (U*V) * Inv(W)
impl<U: Unit, V: BaseUnit, W: BaseUnit> Div<W> for Mult<U, V>
where
    Self: Mul<Inverse<W>>,
{
    type Output = <Self as Mul<Inverse<W>>>::Output;
    // type Output = op!(Self * Inverse<W>); // I think this needs an alias (since typenum op! only
    // works on identifiers)
    fn div(self, rhs: W) -> Self::Output {
        self * Inverse(rhs)
    }
}

// Multipy (U/V) * V => U
impl<U: Unit, V: BaseUnit> Mul<V> for Mult<U, Inverse<V>> {
    type Output = U;
    fn mul(self, _rhs: V) -> Self::Output {
        self.0
    }
}

// Multipy (U/V) * V => U
impl<U: Unit, V: BaseUnit> Mul<Inverse<V>> for Mult<U, V> {
    type Output = U;
    fn mul(self, _rhs: Inverse<V>) -> Self::Output {
        self.0
    }
}

// Multipy (U/V) * V => U
impl<U: Unit, V: BaseUnit> Mul<Inverse<V>> for Mult<U, Inverse<V>> {
    type Output = Mult<Mult<U, Inverse<V>>, Inverse<V>>;
    fn mul(self, rhs: Inverse<V>) -> Self::Output {
        Mult(self, rhs)
    }
}

// Multipy (U*V) * (A*B) => ((U*V) * B) * A
impl<U: Unit, V: Unit, A: Unit, B: Unit> Mul<Mult<A, B>> for Mult<U, V>
where
    Self: Mul<B>,
    op!(Self * B): Mul<A>,
{
    type Output = op!((Self * B) * A);
    fn mul(self, rhs: Mult<A, B>) -> Self::Output {
        (self * rhs.1) * rhs.0
    }
}

// Multiply: (U*V) * Unitless => (U*V)
impl<U: Unit, V: BaseUnit> Mul<Unitless> for Mult<U, V> {
    type Output = Mult<U, V>;
    fn mul(self, _rhs: Unitless) -> Self::Output {
        self
    }
}

// Divide: (U*V) / Unitless => (U*V)
impl<U: Unit, V: BaseUnit> Div<Unitless> for Mult<U, V> {
    type Output = Mult<U, V>;
    fn div(self, _rhs: Unitless) -> Self::Output {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! assert_has_type {
        ($val: expr => $ty:ty) => {
            (|_x: $ty| ())($val);
        };
    }

    #[test]
    fn useability() {
        let a = Second;
        let b = Second;
        let d = Second;
        let c = (a * b) / d;
        assert_has_type!(c => Second);
        assert_has_type!(Unitless / Second => Inverse<Second>);
    }
}
