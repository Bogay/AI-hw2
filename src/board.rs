use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::{matrix::Matrix2D, vec2::Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Dir::Up => Vec2::new(0, -1),
            Dir::Down => Vec2::new(0, 1),
            Dir::Left => Vec2::new(-1, 0),
            Dir::Right => Vec2::new(1, 0),
        }
    }

    pub fn inverse(&self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    id: i8,
    pos: Vec2,
    size: Vec2,
}

impl Block {
    pub fn from_positions(id: i8, positions: &Vec<Vec2>) -> Result<Self, String> {
        match positions.len() {
            1 => Ok(Block {
                id,
                pos: positions[0],
                size: Vec2::new(1, 1),
            }),
            2 => {
                let pos = positions[0];
                let size = if positions[1] == &pos + &Vec2::new(1, 0) {
                    Vec2::new(2, 1)
                } else if positions[1] == &pos + &Vec2::new(0, 1) {
                    Vec2::new(1, 2)
                } else {
                    return Err("Positions cannot form a block".to_string());
                };

                Ok(Block { id, pos, size })
            }
            4 => {
                let pos = positions[0];
                let deltas = vec![Vec2::new(1, 0), Vec2::new(0, 1), Vec2::new(1, 1)];

                for (i, delta) in deltas.iter().enumerate() {
                    if positions[i + 1] != &pos + delta {
                        return Err("Positions cannot form a block".to_string());
                    }
                }

                Ok(Block {
                    id,
                    pos,
                    size: Vec2::new(2, 2),
                })
            }
            len => {
                return Err(format!(
                    "Invalid position size {}, allowed values are 1, 2, 4",
                    len
                ));
            }
        }
    }
}

pub type Move = (i8, Dir);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Board {
    pub(crate) blocks: Vec<Block>,
    pub(crate) id_grid: Matrix2D<i8>,
    pub(crate) holes: HashSet<Vec2>,
    pub(crate) _possible_moves: HashSet<Move>,
    // Final state cache
    pub(crate) final_hole_positions: HashSet<Vec2>,
    pub(crate) final_state: Vec<Vec2>,
}

impl FromStr for Board {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut input = input.lines();
        let line = input.next().ok_or("Missing first line".to_string())?;
        let size = Self::parse_size(line)?;

        if size.x <= 0 || size.y <= 0 {
            return Err("Either row or column size should >= 0".to_string());
        }

        let mut blocks = HashMap::new();
        let mut holes = HashSet::new();
        let mut id_grid = Vec::with_capacity((size.x * size.y) as usize);
        for (row_i, line) in input.into_iter().take(size.y as usize).enumerate() {
            let row = line
                .split_whitespace()
                .map(|v| {
                    v.parse::<i8>()
                        .map_err(|e| format!("Failed to parse block id: {}", e))
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
            for (col_i, id) in row.iter().enumerate() {
                if id == &0 {
                    holes.insert(Vec2::new(col_i as i8, row_i as i8));
                } else {
                    blocks
                        .entry(*id)
                        .or_insert(vec![])
                        .push(Vec2::new(col_i as i8, row_i as i8));
                }
            }
            id_grid.extend(row);
        }
        let id_grid = Matrix2D::from_vec(size, id_grid)?;
        let blocks = Self::parse_blocks(blocks)?;
        let (final_state, final_holes) = Self::generate_final_state(size, &blocks)?;
        let _possible_moves = Self::generate_possible_moves(&holes, &id_grid);

        Ok(Board {
            blocks,
            id_grid,
            holes,
            _possible_moves,
            final_hole_positions: final_holes,
            final_state,
        })
    }
}

impl Board {
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

    fn parse_blocks(blocks: HashMap<i8, Vec<Vec2>>) -> Result<Vec<Block>, String> {
        let mut results = vec![];
        let block_cnt = blocks.len() as i8;

        for id in 1..=block_cnt {
            let block = match blocks.get(&id) {
                Some(positions) => Block::from_positions(id, positions)?,
                None => return Err(format!("Missing block id {}", id)),
            };
            results.push(block);
        }

        Ok(results)
    }

    fn generate_final_state(
        size: Vec2,
        blocks: &Vec<Block>,
    ) -> Result<(Vec<Vec2>, HashSet<Vec2>), String> {
        let mut grid = Matrix2D::fill(size, 0);
        let mut next_block_id = 0;
        let mut final_block_positions = Vec::with_capacity(blocks.len());
        let mut holes = HashSet::new();

        for i in 0..size.y {
            for j in 0..size.x {
                let pos = Vec2::new(j, i);
                if grid.get(pos).unwrap() == &0 {
                    if let Some(block) = blocks.get(next_block_id) {
                        // TODO: return error instead of assert
                        assert_eq!(block.id, (next_block_id + 1) as i8);
                        if grid.try_fill(pos, block.size, block.id).is_ok() {
                            final_block_positions.push(pos);
                            next_block_id += 1;
                        } else {
                            holes.insert(pos);
                        }
                    } else {
                        holes.insert(pos);
                    }
                }
            }
        }

        if blocks.get(next_block_id).is_some() {
            return Err(format!(
                "Cannot fit those blocks into board with size {}x{}",
                size.y, size.x
            ));
        }

        Ok((final_block_positions, holes))
    }

    fn generate_possible_moves(holes: &HashSet<Vec2>, id_grid: &Matrix2D<i8>) -> HashSet<Move> {
        let moves = Self::dir_and_vecs(&vec![Dir::Up, Dir::Down, Dir::Left, Dir::Right]);
        let mut possible_moves = HashSet::new();

        for hole in holes {
            for (v, d) in &moves {
                if let Some(id) = id_grid.get(hole + v) {
                    if id != &0 {
                        possible_moves.insert((*id, d.inverse()));
                    }
                }
            }
        }

        possible_moves
    }

    pub fn move_block(&mut self, id: i8, dir: Dir) -> Result<(), String> {
        self.validate_move(id, dir)?;
        let moves = Self::dir_and_vecs(&vec![Dir::Up, Dir::Down, Dir::Left, Dir::Right]);
        let block = self
            .blocks
            .get_mut((id - 1) as usize)
            .ok_or_else(|| format!("id {} not found", id))?;
        assert_eq!(id, block.id);
        self.id_grid.try_fill(block.pos, block.size, 0)?;
        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let pos = &block.pos + &Vec2::new(dx, dy);
                self.holes.insert(pos);
            }
        }
        block.pos = &block.pos + &dir.to_vec2();
        self.id_grid.try_fill(block.pos, block.size, block.id)?;
        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let pos = &block.pos + &Vec2::new(dx, dy);
                self.holes.remove(&pos);
            }
        }

        // FIXME: This might be insufficient
        self._possible_moves.clear();
        for hole in &self.holes {
            for (v, d) in &moves {
                if let Some(id) = self.id_grid.get(hole + v) {
                    if id != &0 {
                        self._possible_moves.insert((*id, d.inverse()));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_move(&self, id: i8, dir: Dir) -> Result<(), String> {
        let block = self
            .blocks
            .get((id - 1) as usize)
            .ok_or_else(|| format!("id {} not found", id))?;
        assert_eq!(id, block.id);
        let move_vec = dir.to_vec2();

        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let before_move = &block.pos + &Vec2::new(dx, dy);
                let after_move = &before_move + &move_vec;
                if let Some(next_id) = self.id_grid.get(after_move) {
                    if next_id != &0 && next_id != &id {
                        return Err(format!(
                            "Invalid move, {} has occupied by {}",
                            after_move, next_id,
                        ));
                    }
                } else {
                    return Err("Move out of range".to_string());
                }
            }
        }

        Ok(())
    }

    fn dir_and_vecs(dirs: &Vec<Dir>) -> Vec<(Vec2, Dir)> {
        dirs.into_iter().map(|d| (d.to_vec2(), *d)).collect()
    }

    pub fn is_goal(&self) -> bool {
        if self.holes != self.final_hole_positions {
            return false;
        }

        assert_eq!(self.final_state.len(), self.blocks.len());
        self.final_state
            .iter()
            .zip(&self.blocks)
            .all(|(expect, Block { pos: curr, .. })| curr == expect)
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        let result = self._possible_moves.clone().into_iter().collect::<Vec<_>>();
        // result.sort();
        result
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.id_grid.size();
        writeln!(f, "{} {}", size.x, size.y)?;
        for row in self.id_grid.chunks(size.x as usize) {
            writeln!(f, "{:?}", row)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_block() -> Result<(), String> {
        let mut before_move = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 0 5 5\n\
        4 0 7 6\n\
        9 10 8 6\n\
        "
        .parse::<Board>()?;
        before_move.move_block(5, Dir::Left)?;
        let after_move = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 5 5 0\n\
        4 0 7 6\n\
        9 10 8 6\n\
        "
        .parse::<Board>()?;

        assert_eq!(before_move.id_grid, after_move.id_grid);

        Ok(())
    }

    #[test]
    fn test_move_out_of_range() -> Result<(), String> {
        let mut board = "3 3\n\
        1 1 2\n\
        0 3 0\n\
        0 4 4\n\
        "
        .parse::<Board>()?;
        assert!(board.move_block(2, Dir::Right).is_err());

        Ok(())
    }

    #[test]
    fn test_init_goal_checking() -> Result<(), String> {
        let board = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 5 5 6\n\
        4 7 8 6\n\
        9 10 0 0\n\
        "
        .parse::<Board>()?;

        assert!(board.is_goal());

        Ok(())
    }

    #[test]
    fn test_goal_checking_after_move() -> Result<(), String> {
        let reach_goal_at_init = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 5 5 6\n\
        4 7 8 6\n\
        9 10 0 0\n\
        "
        .parse::<Board>()?;
        let mut board = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 5 5 6\n\
        4 7 8 6\n\
        9 0 10 0\n\
        "
        .parse::<Board>()?;
        assert_eq!(board.final_state, reach_goal_at_init.final_state);
        assert_eq!(
            board.final_hole_positions,
            reach_goal_at_init.final_hole_positions
        );
        board.move_block(10, Dir::Left)?;

        assert!(board.is_goal());

        Ok(())
    }

    #[test]
    fn test_init_possible_moves() -> Result<(), String> {
        let board = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 0 5 5\n\
        4 0 7 6\n\
        9 10 8 6\n\
        "
        .parse::<Board>()?;

        let expected = HashSet::from_iter([
            (4, Dir::Right),
            (2, Dir::Down),
            (5, Dir::Left),
            (7, Dir::Left),
            (10, Dir::Up),
        ]);
        assert_eq!(expected, board._possible_moves);

        Ok(())
    }

    #[test]
    fn test_possible_moves_after_move() -> Result<(), String> {
        let mut board = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 0 5 5\n\
        4 0 7 6\n\
        9 10 8 6\n\
        "
        .parse::<Board>()?;
        board.move_block(10, Dir::Up)?;

        let expected = HashSet::from_iter([
            (4, Dir::Right),
            (2, Dir::Down),
            (5, Dir::Left),
            (10, Dir::Up),
            (10, Dir::Down),
            (9, Dir::Right),
            (8, Dir::Left),
        ]);

        assert_eq!(expected, board._possible_moves);

        Ok(())
    }

    #[test]
    fn test_move_is_recoverable() -> Result<(), String> {
        let mut board = "5 4\n\
        1 2 2 3\n\
        1 2 2 3\n\
        4 0 5 5\n\
        4 0 7 6\n\
        9 10 8 6\n\
        "
        .parse::<Board>()?;
        let original = board.clone();

        board.move_block(5, Dir::Left)?;
        assert_ne!(board, original);
        board.move_block(5, Dir::Right)?;
        assert_eq!(board, original);

        Ok(())
    }
}
