use std::{fmt::Display, ops::Add};

/// A (x, y) vector
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

pub struct Square {
    offset: Vec2,
    size: Vec2,
}

impl Square {
    #[must_use]
    pub fn new(offset: Vec2, size: Vec2) -> Self {
        // TODO: error handling
        if size.x <= 0 || size.y <= 0 {
            panic!("x & y of size should be positive");
        }
        Self { offset, size }
    }

    #[must_use]
    pub fn at_origin(size: Vec2) -> Self {
        Self::new(Vec2::new(0, 0), size)
    }

    pub fn row_iter(&self) -> impl Iterator<Item = Vec2> + '_ {
        (0..self.size.y).flat_map(move |dy| {
            (0..self.size.x).map(move |dx| Vec2::new(dx + self.offset.x, self.offset.y + dy))
        })
    }

    pub fn col_iter(&self) -> impl Iterator<Item = Vec2> + '_ {
        (0..self.size.x).flat_map(move |dx| {
            (0..self.size.y).map(move |dy| Vec2::new(dx + self.offset.x, self.offset.y + dy))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_iter() {
        let squ = Square::new(Vec2::new(2, 2), Vec2::new(2, 2));
        let expected = vec![
            Vec2::new(2, 2),
            Vec2::new(3, 2),
            Vec2::new(2, 3),
            Vec2::new(3, 3),
        ];
        assert_eq!(squ.row_iter().collect::<Vec<_>>(), expected);
    }
    #[test]
    fn test_col_iter() {
        let squ = Square::new(Vec2::new(2, 2), Vec2::new(2, 2));
        let expected = vec![
            Vec2::new(2, 2),
            Vec2::new(2, 3),
            Vec2::new(3, 2),
            Vec2::new(3, 3),
        ];
        assert_eq!(squ.col_iter().collect::<Vec<_>>(), expected);
    }
}
