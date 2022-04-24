use crate::board::{Board, Dir, Move};
use log::{debug, trace};

pub fn iddfs(board: Board) -> Option<Vec<Move>> {
    let mut limit = 1;

    loop {
        debug!("limit: {}", limit);
        let mut board = board.clone();
        if let Some(mut moves) = dfs(&mut board, limit) {
            moves.reverse();
            return Some(moves);
        }
        limit += 1;
    }
}

fn dfs(board: &mut Board, limit: i32) -> Option<Vec<Move>> {
    if board.is_goal() {
        return Some(vec![]);
    }
    if limit <= 0 {
        return None;
    }

    for (id, dir) in board.possible_moves() {
        if let Err(e) = board.move_block(id, dir) {
            trace!("{} {:?}", e, (id, dir));
            continue;
        }
        if let Some(mut moves) = dfs(board, limit - 1) {
            moves.push((id, dir));
            return Some(moves);
        }
        assert!(board.move_block(id, dir.inverse()).is_ok());
    }

    None
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
