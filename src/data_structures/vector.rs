#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Vector { x, y }
    }

    #[inline]
    pub fn cross_product(self, rhs: Self) -> f32 {
        (self.x * rhs.y) - (self.y * rhs.x)
    }
}
impl From<(f32, f32)> for Vector {
    fn from(value: (f32, f32)) -> Self {
        Vector::new(value.0, value.1)
    }
}

impl std::ops::Div<f32> for Vector {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self {
            x: self.x.div(rhs),
            y: self.y.div(rhs),
        }
    }
}

impl std::ops::Div<Vector> for Vector {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Vector) -> Self {
        Self {
            x: self.x.div(rhs.x),
            y: self.y.div(rhs.y),
        }
    }
}
impl std::ops::Mul<f32> for Vector {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x.mul(rhs),
            y: self.y.mul(rhs),
        }
    }
}
impl std::ops::Mul<Vector> for Vector {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x.mul(rhs.x),
            y: self.y.mul(rhs.y),
        }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x.add(rhs.x),
            y: self.y.add(rhs.y),
        }
    }
}
impl std::ops::Add<f32> for Vector {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self {
        Self {
            x: self.x.add(rhs),
            y: self.y.add(rhs),
        }
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x.sub(rhs.x),
            y: self.y.sub(rhs.y),
        }
    }
}
