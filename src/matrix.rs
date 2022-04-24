use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    str::FromStr,
};

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
            store: vec![fillin_value; size.x as usize * size.y as usize],
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
}

impl<T> Matrix2D<T>
where
    T: Clone + Default + PartialEq,
{
    pub fn try_fill_without_cover(
        &mut self,
        anchor: Vec2,
        size: Vec2,
        value: T,
    ) -> Result<(), String> {
        for dy in 0..size.y {
            for dx in 0..size.x {
                match self.get(&anchor + &Vec2::new(dx, dy)) {
                    Some(value) if value != &T::default() => {
                        return Err("Fill area covers non-default value".to_string())
                    }
                    None => return Err("Fill area out of range".to_string()),
                    _ => {}
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
}

impl<T> Matrix2D<T> {
    #[must_use]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    fn is_inside(&self, pos: &Vec2) -> bool {
        pos.x >= 0 && pos.x < self.size.x && pos.y >= 0 && pos.y < self.size.y
    }

    pub fn get(&self, pos: Vec2) -> Option<&T> {
        if !self.is_inside(&pos) {
            return None;
        }
        self.store
            .get(pos.y as usize * self.size.x as usize + pos.x as usize)
    }

    pub fn get_mut(&mut self, pos: Vec2) -> Option<&mut T> {
        if !self.is_inside(&pos) {
            return None;
        }
        self.store
            .get_mut(pos.y as usize * self.size.x as usize + pos.x as usize)
    }

    pub fn from_vec(size: Vec2, vec: Vec<T>) -> Result<Self, String> {
        let expect_size = size.x as usize * size.y as usize;
        if expect_size != vec.len() {
            return Err(format!(
                "Invalid vector size. expect {}, got {}",
                expect_size,
                vec.len()
            ));
        }

        Ok(Self { size, store: vec })
    }

    fn parse_size(line: &str) -> Result<Vec2, String> {
        let size = line.split_whitespace().collect::<Vec<_>>();
        if size.len() != 2 {
            return Err("First line should be the board row & column size".to_string());
        }
        let size = size
            .into_iter()
            .map(|s| {
                s.parse::<i8>()
                    .map_err(|e| format!("Failed to parse size: {}", e))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Vec2::new(size[1], size[0]))
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

impl<T> FromStr for Matrix2D<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut input = input.lines();
        let line = input
            .next()
            .ok_or_else(|| "Missing first line".to_string())?;
        let size = Self::parse_size(line)?;

        if size.x <= 0 || size.y <= 0 {
            return Err("Either row or column size should >= 0".to_string());
        }

        let mut id_grid = Vec::with_capacity(size.x as usize * size.y as usize);
        for (row_i, line) in input.into_iter().take(size.y as usize).enumerate() {
            let row = line
                .split_whitespace()
                .map(|v| {
                    v.parse::<T>()
                        .map_err(|e| format!("Failed to parse block id: {:?}", e))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if row.len() != size.x as usize {
                return Err(format!(
                    "Invalid line {}: expect {} block, got {}",
                    row_i,
                    size.x,
                    row.len(),
                ));
            }
            id_grid.extend(row);
        }
        Matrix2D::from_vec(size, id_grid)
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
