pub trait Unit: Copy {
    type Base;
    fn new() -> Self;
    fn conversion() -> Op;
}
pub trait BaseUnit: Unit {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Mult(usize),
    Div(usize),
}

impl Op {
    fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Mult(a), Self::Mult(b)) => Self::Mult(a * b),
            (Self::Div(a), Self::Div(b)) => Self::Div(a * b),
            (Self::Mult(a), Self::Div(b)) if a < b => Self::Div(b / a),
            (Self::Mult(a), Self::Div(b)) => Self::Mult(a / b),
            (Self::Div(a), Self::Mult(b)) if a < b => Self::Mult(b / a),
            (Self::Div(a), Self::Mult(b)) => Self::Div(a / b),
        }
    }

    fn convert(&self, value: usize) -> usize {
        match self {
            Self::Mult(m) => value * m,
            Self::Div(m) => value / m,
        }
    }

    fn inverse(self) -> Self {
        match self {
            Self::Mult(m) => Self::Div(m),
            Self::Div(m) => Self::Mult(m),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Area<U: BaseUnit>(U);

impl<U: BaseUnit> Unit for Area<U> {
    type Base = Self;
    fn new() -> Self {
        Self(U::new())
    }
    fn conversion() -> Op {
        U::conversion().combine(U::conversion())
    }
}

impl<U: BaseUnit> BaseUnit for Area<U> {}

macro_rules! base_unit {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name();
        impl Unit for $name {
            type Base = Self;
            fn new() -> Self {
                Self()
            }

            fn conversion() -> Op {
                Op::Mult(1)
            }
        }
        impl BaseUnit for $name {}
    };
}

base_unit!(Meter);
base_unit!(Gram);
base_unit!(Second);

macro_rules! metric_prefix {
    ($name:ident => * $num:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name<T: BaseUnit>(T);
        impl<T: BaseUnit> Unit for $name<T> {
            type Base = T;
            fn new() -> Self {
                Self(T::new())
            }

            fn conversion() -> Op {
                Op::Mult($num)
            }
        }
    };
    ($name:ident => / $num:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name<T: BaseUnit>(T);
        impl<T: BaseUnit> Unit for $name<T> {
            type Base = T;
            fn new() -> Self {
                Self(T::new())
            }

            fn conversion() -> Op {
                Op::Div($num)
            }
        }
    };
}

metric_prefix!(Centi => * 100);
metric_prefix!(Kilo => / 1000);

#[derive(Debug, Clone, Copy)]
pub struct Value<U: Unit>(usize, U);

impl<U: Unit> Value<U> {
    pub fn new(val: usize) -> Self {
        Self(val, U::new())
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>> Value<U> {
    fn convert<T: Unit<Base = B>>(self) -> Value<T> {
        let val = U::conversion()
            .inverse()
            .combine(T::conversion())
            .convert(self.0);
        Value(val, T::new())
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>> Value<U> {
    fn add<R: Unit<Base = B>, T: Unit<Base = B>>(self, rhs: Value<T>) -> Value<R> {
        let s = self.convert::<R>().0;
        let o = rhs.convert::<R>().0;
        Value(s + o, R::new())
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::ops::Add<Value<T>>
    for Value<U>
{
    type Output = Self;
    fn add(self, rhs: Value<T>) -> Self {
        self.add(rhs)
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::ops::AddAssign<Value<T>>
    for Value<U>
{
    fn add_assign(&mut self, rhs: Value<T>) {
        self.0 += rhs.convert::<U>().0;
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::cmp::PartialEq<Value<T>>
    for Value<U>
{
    fn eq(&self, other: &Value<T>) -> bool {
        self.convert::<T>().0 == other.0
    }
}

impl<B: BaseUnit + Unit, U: Unit<Base = B>, T: Unit<Base = B>> std::ops::Mul<Value<T>>
    for Value<U>
{
    type Output = Value<Area<B>>;
    fn mul(self, rhs: Value<T>) -> Self::Output {
        let s = U::conversion().convert(self.0);
        let o = T::conversion().convert(rhs.0);
        Value(s * o, Area::new())
    }
}

pub type KiloMeter = Kilo<Meter>;
pub type CentiMeter = Centi<Meter>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(
            Value::<Meter>::new(1).convert::<CentiMeter>(),
            Value::<CentiMeter>::new(100)
        );
        assert_eq!(
            Value::<KiloMeter>::new(1).convert::<Meter>(),
            Value::<Meter>::new(1000)
        );
        // Verify that the soft matching also works
        assert_eq!(Value::<KiloMeter>::new(1), Value::<Meter>::new(1000));
        assert_eq!(
            Value::<Meter>::new(1) * Value::<Meter>::new(1),
            Value::<Area<Meter>>::new(1)
        );
    }
}
