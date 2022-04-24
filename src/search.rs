use crate::board::{Board, BoardState, Dir, Move};
use log::{debug, trace};
use std::collections::BTreeSet;

pub fn iddfs(board: Board) -> Option<Vec<Move>> {
    let mut limit = 1;

    loop {
        debug!("limit: {}", limit);
        if let Some(mut moves) = dfs(&mut board.clone(), limit, &mut Default::default()) {
            moves.reverse();
            return Some(moves);
        }
        limit += 1;
    }
}

fn dfs(board: &mut Board, limit: i32, visited: &mut BTreeSet<BoardState>) -> Option<Vec<Move>> {
    if board.is_goal() {
        return Some(vec![]);
    }
    if limit <= 0 {
        return None;
    }
    if visited.get(board.state()).is_some() {
        return None;
    } else {
        visited.insert(board.state().clone());
    }

    for (id, dir) in board.possible_moves() {
        if let Err(e) = board.move_block(id, dir) {
            trace!("{} {:?}", e, (id, dir));
            continue;
        }
        if let Some(mut moves) = dfs(board, limit - 1, visited) {
            moves.push((id, dir));
            return Some(moves);
        }
        assert!(board.move_block(id, dir.inverse()).is_ok());
    }

    visited.remove(board.state());
    None
}

pub fn idastar(board: Board) -> Option<Vec<Move>> {
    let mut f_limit = board.heuristic();
    loop {
        match _idastar(&mut board.clone(), 0, f_limit, &mut Default::default()) {
            Ok(mut moves) => {
                moves.reverse();
                return Some(moves);
            }
            Err(new_limit) => {
                if new_limit <= f_limit {
                    return None;
                } else {
                    f_limit = new_limit;
                }
            }
        }
    }
}

fn _idastar(
    board: &mut Board,
    g_value: i32,
    mut f_limit: i32,
    visited: &mut BTreeSet<BoardState>,
) -> Result<Vec<Move>, i32> {
    if board.is_goal() {
        return Ok(vec![]);
    }
    if visited.get(board.state()).is_some() {
        return Err(f_limit);
    } else {
        visited.insert(board.state().clone());
    }

    for (id, dir) in board.possible_moves() {
        if let Err(e) = board.move_block(id, dir) {
            trace!("{} {:?}", e, (id, dir));
            continue;
        }
        let f_value = g_value + board.heuristic();
        if f_value < f_limit {
            if let Ok(mut moves) = _idastar(board, g_value + 1, f_limit, visited) {
                moves.push((id, dir));
                return Ok(moves);
            }
        }
        f_limit = std::cmp::max(f_limit, f_value);
        assert!(board.move_block(id, dir.inverse()).is_ok());
    }

    visited.remove(board.state());
    Err(f_limit)
}

pub fn manual(mut board: Board) -> Option<Vec<Move>> {
    use std::io;

    let input = io::stdin();
    let mut buffer = String::new();
    let mut moves = vec![];

    eprintln!("{}", board);
    loop {
        eprintln!("Enter a move: ");
        eprintln!("Possible values are: {:?}", board.possible_moves());
        let bytes = input.read_line(&mut buffer).expect("Read move fail");
        if bytes == 0 {
            break;
        }
        match parse_cmd(buffer.trim()) {
            Ok((id, dir)) => {
                if let Err(e) = board.move_block(id, dir) {
                    eprintln!("{}", e);
                }
                moves.push((id, dir));
            }
            Err(e) => {
                eprintln!("Invalid command: {}", e);
                continue;
            }
        }
        eprintln!("{}", board);
        if board.is_goal() {
            eprintln!("Reach goal");
            break;
        }
        buffer.clear();
    }

    Some(moves)
}

fn parse_cmd(cmd: &str) -> Result<Move, String> {
    let dir = cmd.chars().last().ok_or("Empty command")?;
    let dir = match dir {
        'U' => Dir::Up,
        'D' => Dir::Down,
        'L' => Dir::Left,
        'R' => Dir::Right,
        _ => return Err(format!("Invalid direction: {}", dir)),
    };

    let id = {
        let mut chars = cmd.chars();
        chars.next_back();
        chars
            .as_str()
            .parse::<i8>()
            .map_err(|e| format!("Invalid id: {}", e))?
    };

    Ok((id, dir))
}
