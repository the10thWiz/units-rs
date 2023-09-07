use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Add, Mul, Div},
};
use typenum::{op, Integer, ToInt};

use crate::{base::Unitless, Unit, Value, UnitValue};

#[derive(Clone, Copy)]
pub struct Prefix<Power, U>(pub(crate) U, pub(crate) PhantomData<Power>);

impl<P: ToInt<i32>, U: Debug> Debug for Prefix<P, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name() {
            Some("") => write!(f, "{:?}", self.0),
            Some(name) => write!(f, "{name} {:?}", self.0),
            None => write!(f, "{:?} x10^{}", self.0, P::to_int()),
        }
    }
}

impl<UP: Integer, VP: Integer, U: Unit, V: Unit> Mul<Prefix<VP, V>> for Prefix<UP, U>
where
    U: Mul<V>,
    UP: Add<VP>,
{
    type Output = Prefix<op!(UP + VP), op!(U * V)>;
    fn mul(self, rhs: Prefix<VP, V>) -> Self::Output {
        Prefix(self.0 * rhs.0, PhantomData)
    }
}

impl<Power: ToInt<i32>, U> Prefix<Power, U> {
    pub fn power(&self) -> i32 {
        Power::to_int()
    }

    pub fn name(&self) -> Option<&'static str> {
        match self.power() {
            -24 => Some("Yotta"),
            -21 => Some("Zetta"),
            -18 => Some("Exa"),
            -15 => Some("Peta"),
            -12 => Some("Tera"),
            -9 => Some("Giga"),
            -6 => Some("Mega"),
            -3 => Some("Kilo"),
            -2 => Some("Hecta"),
            -1 => Some("Deka"),
            0 => Some(""),
            1 => Some("Deci"),
            2 => Some("Centi"),
            3 => Some("Milli"),
            6 => Some("Micro"),
            9 => Some("Nano"),
            12 => Some("Pico"),
            15 => Some("Fempto"),
            18 => Some("Atto"),
            21 => Some("Zepto"),
            24 => Some("Yocto"),
            _ => None,
        }
    }
}

impl<Power: Integer, U: Unit> Prefix<Power, U> {
    pub fn new<V>(val: V) -> Value<V, Power, U> {
        Value::new(val)
    }
}

impl<Power: ToInt<i32>, U: Unit> Prefix<Power, U> {
    pub fn convert<V: UnitValue, RhsPower: ToInt<i32>, RhsUnits: Unit>(
        &self,
        value: V,
        _rhs: &Prefix<RhsPower, RhsUnits>,
    ) -> V
    where
        U: Div<RhsUnits, Output = Unitless>,
    {
        let power = Power::to_int() - RhsPower::to_int();
        value.apply_prefix(-power)
    }
}

pub type Yotta<U> = Prefix<typenum::consts::N24, U>;
pub type Zetta<U> = Prefix<typenum::consts::N21, U>;
pub type Exa<U> = Prefix<typenum::consts::N18, U>;
pub type Peta<U> = Prefix<typenum::consts::N15, U>;
pub type Tera<U> = Prefix<typenum::consts::N12, U>;
pub type Giga<U> = Prefix<typenum::consts::N9, U>;
pub type Mega<U> = Prefix<typenum::consts::N6, U>;
pub type Kilo<U> = Prefix<typenum::consts::N3, U>;
pub type Hecta<U> = Prefix<typenum::consts::N2, U>;
pub type Deka<U> = Prefix<typenum::consts::N1, U>;
pub type Base<U> = Prefix<typenum::consts::Z0, U>;
pub type Deci<U> = Prefix<typenum::consts::P1, U>;
pub type Centi<U> = Prefix<typenum::consts::P2, U>;
pub type Milli<U> = Prefix<typenum::consts::P3, U>;
pub type Micro<U> = Prefix<typenum::consts::P6, U>;
pub type Nano<U> = Prefix<typenum::consts::P9, U>;
pub type Pico<U> = Prefix<typenum::consts::P12, U>;
pub type Fempto<U> = Prefix<typenum::consts::P15, U>;
pub type Atto<U> = Prefix<typenum::consts::P18, U>;
pub type Zepto<U> = Prefix<typenum::consts::P21, U>;
pub type Yocto<U> = Prefix<typenum::consts::P24, U>;

#[cfg(test)]
mod tests {
    use crate::base::Meter;

    use super::*;

    #[test]
    fn with_prefix() {
        assert_eq!(Kilo::<Meter>::new(1f64), Meter::new(1000));
        assert_eq!(format!("{:?}", Kilo::<Meter>::new(1f64)), format!("{:?} Kilo Meters", 1f64));
        assert_eq!(format!("{:?}", Meter::new(1000f64)), format!("{:?} Meters", 1000f64));
    }
}
