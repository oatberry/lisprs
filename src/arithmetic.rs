use crate::values::Value::{self, *};
use std::ops;

// because math is hard

impl ops::Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a + b),
            (Float(a), Float(b))     => Float(a + b),
            (Integer(a), Float(b))   => Float(a as f64 + b),
            (Float(a), Integer(b))   => Float(a + (b as f64)),
            _ => unreachable!(),
        }
    }
}

impl ops::Sub for Value {
    type Output = Value;

    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a - b),
            (Float(a), Float(b))     => Float(a - b),
            (Integer(a), Float(b))   => Float(a as f64 - b),
            (Float(a), Integer(b))   => Float(a - (b as f64)),
            _ => unreachable!(),
        }
    }
}

impl ops::Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a * b),
            (Float(a), Float(b))     => Float(a * b),
            (Integer(a), Float(b))   => Float(a as f64 * b),
            (Float(a), Integer(b))   => Float(a * (b as f64)),
            _ => unreachable!(),
        }
    }
}

impl ops::Div for Value {
    type Output = Value;

    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Integer(a), Integer(b)) => Integer(a / b),
            (Float(a), Float(b))     => Float(a / b),
            (Integer(a), Float(b))   => Float(a as f64 / b),
            (Float(a), Integer(b))   => Float(a / (b as f64)),
            (_, _) => unreachable!(),
        }
    }
}

impl ops::Rem for Value {
    type Output = Value;

    fn rem(self, modulus: Value) -> Value {
        match (self, modulus) {
            (Integer(a), Integer(b)) => Integer(a % b),
            (Float(a), Float(b))     => Float(a % b),
            (Integer(a), Float(b))   => Float(a as f64 % b),
            (Float(a), Integer(b))   => Float(a % (b as f64)),
            (_, _) => unreachable!(),
        }
    }
}
