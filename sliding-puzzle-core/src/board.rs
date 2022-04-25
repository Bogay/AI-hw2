use crate::{matrix::Matrix2D, vec2::Vec2};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fmt::{Debug, Display},
    str::FromStr,
};

/// Direction on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    /// Convert direction to corresponding vector
    pub fn to_vec2(self) -> Vec2 {
        match self {
            Dir::Up => Vec2::new(0, -1),
            Dir::Down => Vec2::new(0, 1),
            Dir::Left => Vec2::new(-1, 0),
            Dir::Right => Vec2::new(1, 0),
        }
    }

    /// Get inverse direction
    pub fn inverse(&self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

/// Block on board
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block {
    /// Block's id, should be unique
    id: i8,
    /// Position of block, which is the top-left cell's position here
    pos: Vec2,
    /// Width & height og this block
    size: Vec2,
}

impl Block {
    /// Build block from positions, note that positions must be sorted in row majoring order
    pub fn from_positions(id: i8, positions: &[Vec2]) -> Result<Self, String> {
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

/// Represente a move of a board
pub type Move = (i8, Dir);

/// Board of sliding puzzle
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Board {
    /// Grid to store cells are occupied by which id
    grid: Matrix2D<i8>,
    /// Current state of board
    state: BoardState,
    /// The final state this board want to reach
    final_state: BoardState,
    _possible_moves: HashSet<Move>,
}

// FIXME: The compare only make sense iff they refer to the same board
/// Board state, store all block data
#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoardState {
    /// Positions of empty cells
    holes: BTreeSet<Vec2>,
    /// Blocks of this board, should be sorted by id and blank id is not allowed
    blocks: Vec<Block>,
}

impl BoardState {
    pub fn new(holes: Vec<Vec2>, blocks: Vec<Block>) -> Self {
        Self {
            holes: BTreeSet::from_iter(holes),
            blocks,
        }
    }
}

impl FromStr for Board {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let id_grid = input.parse::<Matrix2D<i8>>()?;
        Self::try_from(id_grid)
    }
}

impl Board {
    /// Convert positions to blocks
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

    /// Generate the final state from board size & blocks
    fn generate_final_state(size: Vec2, blocks: &[Block]) -> Result<BoardState, String> {
        let mut grid = Matrix2D::fill(size, 0);
        let mut next_block_id = 0;
        let mut result_blocks = Vec::with_capacity(blocks.len());
        let mut holes = vec![];

        for i in 0..size.y {
            for j in 0..size.x {
                let pos = Vec2::new(j, i);
                if grid.get(pos).unwrap() == &0 {
                    if let Some(block) = blocks.get(next_block_id) {
                        // TODO: return error instead of assert
                        assert_eq!(block.id, (next_block_id + 1) as i8);
                        if grid.try_fill(pos, block.size, block.id).is_ok() {
                            result_blocks.push(Block {
                                id: block.id,
                                pos,
                                size: block.size,
                            });
                            next_block_id += 1;
                        } else {
                            holes.push(pos);
                        }
                    } else {
                        holes.push(pos);
                    }
                }
            }
        }

        if result_blocks.get(next_block_id).is_some() {
            return Err(format!(
                "Cannot fit those blocks into board with size {}x{}",
                size.y, size.x
            ));
        }

        holes.sort();
        Ok(BoardState::new(holes, result_blocks))
    }

    fn generate_possible_moves(
        holes: &mut dyn Iterator<Item = &Vec2>,
        id_grid: &Matrix2D<i8>,
    ) -> HashSet<Move> {
        let moves = Self::dir_and_vecs(&[Dir::Up, Dir::Down, Dir::Left, Dir::Right]);
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
        self.is_valid_move((id, dir))?;
        let block = self
            .state
            .blocks
            .get_mut((id - 1) as usize)
            .ok_or_else(|| format!("id {} not found", id))?;
        assert_eq!(id, block.id);
        self.grid.try_fill(block.pos, block.size, 0)?;
        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let pos = &block.pos + &Vec2::new(dx, dy);
                self.state.holes.insert(pos);
            }
        }
        block.pos = &block.pos + &dir.to_vec2();
        self.grid.try_fill(block.pos, block.size, block.id)?;
        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let pos = &block.pos + &Vec2::new(dx, dy);
                self.state.holes.remove(&pos);
            }
        }

        // FIXME: This might be insufficient
        self._possible_moves =
            Self::generate_possible_moves(&mut self.state.holes.iter(), &self.grid);

        Ok(())
    }

    /// Check whether a move is valid
    fn is_valid_move(&self, (id, dir): Move) -> Result<(), String> {
        let block = self
            .state
            .blocks
            .get((id - 1) as usize)
            .ok_or_else(|| format!("id {} not found", id))?;
        assert_eq!(id, block.id);
        let move_vec = dir.to_vec2();

        for dx in 0..block.size.x {
            for dy in 0..block.size.y {
                let before_move = &block.pos + &Vec2::new(dx, dy);
                let after_move = &before_move + &move_vec;
                if let Some(next_id) = self.grid.get(after_move) {
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

    fn dir_and_vecs(dirs: &[Dir]) -> Vec<(Vec2, Dir)> {
        dirs.iter().map(|d| (d.to_vec2(), *d)).collect()
    }

    pub fn is_goal(&self) -> bool {
        self.state == self.final_state
    }

    /// Get possible moves from current state
    pub fn possible_moves(&self) -> Vec<Move> {
        self._possible_moves.clone().into_iter().collect::<Vec<_>>()
    }

    /// Get a reference to the board's id grid.
    pub fn grid(&self) -> &Matrix2D<i8> {
        &self.grid
    }

    /// Get a reference to the board's state.
    pub fn state(&self) -> &BoardState {
        &self.state
    }

    pub fn heuristic(&self) -> i32 {
        self.state
            .blocks
            .iter()
            .zip(&self.final_state.blocks)
            .map(|(curr, target)| {
                (curr.pos.x - target.pos.x).abs() as i32 + (curr.pos.y - target.pos.y).abs() as i32
            })
            .sum()
    }

    /// Randonly generate a valid board
    pub fn generate(size: Vec2, block_count: i8, shuffle_round: usize) -> Self {
        let mut next_id = 1;
        let mut possible_block_sizes = vec![
            Vec2::new(2, 1),
            Vec2::new(1, 1),
            Vec2::new(1, 2),
            Vec2::new(2, 2),
        ];
        let mut grid = Matrix2D::fill(size, 0i8);
        let mut rng = thread_rng();

        'fill: for i in 0..size.y {
            for j in 0..size.x {
                let pos = Vec2::new(j, i);
                if grid.get(pos).unwrap() == &0 {
                    possible_block_sizes.shuffle(&mut rng);
                    let id = next_id;
                    next_id += 1;
                    for block_size in &possible_block_sizes {
                        if grid.try_fill_without_cover(pos, *block_size, id).is_ok() {
                            break;
                        }
                    }
                    if next_id > block_count {
                        break 'fill;
                    }
                }
            }
        }

        let mut board: Board = Board::try_from(grid).expect("Invalid input grid");
        // Randomly shuffle board
        let mut rng = thread_rng();
        for _i in 0..shuffle_round {
            let possible_moves = board.possible_moves();
            if let Some((id, dir)) = possible_moves.choose(&mut rng) {
                let _ = board.move_block(*id, *dir);
            } else {
                break;
            }
        }

        board
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let size = self.grid.size();
        writeln!(f, "{} {}", size.x, size.y)?;
        for row in self.grid.chunks(size.x as usize) {
            writeln!(f, "{:?}", row)?;
        }
        Ok(())
    }
}

impl TryFrom<Matrix2D<i8>> for Board {
    type Error = String;

    fn try_from(grid: Matrix2D<i8>) -> Result<Self, Self::Error> {
        let size = grid.size();
        // Parse holes & blocks
        let mut blocks = HashMap::new();
        let mut holes = vec![];
        for row_i in 0..size.y {
            for col_i in 0..size.x {
                let pos = Vec2::new(col_i as i8, row_i as i8);
                let id = grid.get(pos).expect("This query should fit inside matrix");
                if id == &0 {
                    holes.push(pos);
                } else {
                    blocks
                        .entry(*id)
                        .or_insert(vec![])
                        .push(Vec2::new(col_i as i8, row_i as i8));
                }
            }
        }
        holes.sort();
        let blocks = Self::parse_blocks(blocks)?;
        let state = BoardState::new(holes, blocks);
        let final_state = Self::generate_final_state(size, &state.blocks)?;
        let _possible_moves = Self::generate_possible_moves(&mut state.holes.iter(), &grid);

        Ok(Board {
            grid,
            state,
            final_state,
            _possible_moves,
        })
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

        assert_eq!(before_move.grid, after_move.grid);

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
