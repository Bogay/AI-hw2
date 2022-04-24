use std::ops::{Deref, DerefMut};

use crate::vec2::Vec2;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Matrix2D<T> {
    store: Vec<T>,
    size: Vec2,
}

impl<T> Matrix2D<T>
where
    T: Clone,
{
    pub fn fill(size: Vec2, fillin_value: T) -> Self {
        Self {
            size,
            store: vec![fillin_value; (size.x * size.y) as usize],
        }
    }

    pub fn try_fill(&mut self, anchor: Vec2, size: Vec2, value: T) -> Result<(), String> {
        for dy in 0..size.y {
            for dx in 0..size.x {
                if self.get(&anchor + &Vec2::new(dx, dy)).is_none() {
                    return Err("Fill area out of range".to_string());
                }
            }
        }

        for dy in 0..size.y {
            for dx in 0..size.x {
                *self.get_mut(&anchor + &Vec2::new(dx, dy)).unwrap() = value.clone();
            }
        }

        Ok(())
    }

    #[must_use]
    pub fn size(&self) -> Vec2 {
        self.size
    }
}

impl<T> Matrix2D<T> {
    fn is_inside(&self, pos: &Vec2) -> bool {
        pos.x >= 0 && pos.x < self.size.x && pos.y >= 0 && pos.y < self.size.y
    }

    pub fn get(&self, pos: Vec2) -> Option<&T> {
        if !self.is_inside(&pos) {
            return None;
        }
        self.store.get((pos.y * self.size.x + pos.x) as usize)
    }

    pub fn get_mut(&mut self, pos: Vec2) -> Option<&mut T> {
        if !self.is_inside(&pos) {
            return None;
        }
        self.store.get_mut((pos.y * self.size.x + pos.x) as usize)
    }

    pub fn from_vec(size: Vec2, vec: Vec<T>) -> Result<Self, String> {
        let expect_size = (size.x * size.y) as usize;
        if expect_size != vec.len() {
            return Err(format!(
                "Invalid vector size. expect {}, got {}",
                expect_size,
                vec.len()
            ));
        }

        Ok(Self { size, store: vec })
    }
}

impl<T> Deref for Matrix2D<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.store
    }
}

impl<T> DerefMut for Matrix2D<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.store
    }
}

mod tests {
    use crate::{matrix::Matrix2D, vec2::Vec2};

    #[test]
    fn test_eq() {
        let a = Matrix2D::fill(Vec2::new(3, 3), 5);
        let b = Matrix2D::fill(Vec2::new(3, 3), 5);

        assert_eq!(a, b);
    }

    #[test]
    fn test_ne() {
        let a = Matrix2D::fill(Vec2::new(3, 3), 2);
        let b = Matrix2D::fill(Vec2::new(3, 3), 5);

        assert_ne!(a, b);
    }

    #[test]
    fn test_get() -> Result<(), String> {
        let size = Vec2::new(3, 3);
        let mut v = vec![];
        for i in 0..9 {
            v.push(i);
        }
        let mat = Matrix2D::from_vec(size, v)?;

        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(Some(&(y * 3 + x)), mat.get(Vec2::new(x, y)));
            }
        }

        Ok(())
    }

    #[test]
    fn test_get_out_of_range() {
        let mat = Matrix2D::fill(Vec2::new(2, 2), 7);

        assert_eq!(mat.get(Vec2::new(3, 1)), None);
        assert_eq!(mat.get(Vec2::new(1, 3)), None);
        assert_eq!(mat.get(Vec2::new(3, 3)), None);
    }
}
