use clap::ArgEnum;
use sliding_puzzle_core::{Board, Move};
use sliding_puzzle_search::search;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
#[allow(clippy::upper_case_acronyms)]
pub enum Algorithm {
    IDDFS,
    IDAStar,
    Manual,
}

pub fn execute(algorithm: Algorithm, board: Board) -> Option<Vec<Move>> {
    match algorithm {
        Algorithm::IDDFS => search::iddfs(board),
        Algorithm::IDAStar => search::idastar(board),
        Algorithm::Manual => search::manual(board),
    }
}
