use std::{fmt::Display, ops::Add};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vec2 {
    pub x: i8,
    pub y: i8,
}

impl Vec2 {
    pub fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

impl Add for &Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec2({}, {})", self.x, self.y)
    }
}
