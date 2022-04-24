mod board;
mod matrix;
mod search;
mod vec2;

use board::{Board, Move};
use clap::{ArgEnum, Parser, Subcommand};
use matrix::Matrix2D;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};
use vec2::Vec2;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Algorithm {
    IDDFS,
    IDAStar,
    Manual,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Search {
        /// Path to the input file
        #[clap(short, long)]
        input: String,
        /// Path to the output file, default to stdout
        #[clap(short, long)]
        output: Option<String>,
        /// Algorithm to use, default to IDDFS
        #[clap(arg_enum, short, long, default_value_t = Algorithm::IDDFS)]
        algorithm: Algorithm,
    },
    Generate {
        /// Path to the output file, default to stdout
        #[clap(short, long)]
        output: Option<String>,
        /// The output board size
        #[clap(short, long, parse(try_from_str = vec2_from_str))]
        size: Vec2,
        /// At most how many blocks should be generated
        #[clap(short = 'n', long)]
        block_count: i8,
        #[clap(long, default_value_t = 8)]
        shuffle_round: usize,
    },
}

fn vec2_from_str(input: &str) -> Result<Vec2, String> {
    let input = input.split(',').collect::<Vec<_>>();

    if input.len() != 2 {
        return Err("Input shoud be 2 comma-delimited number. e.g. 4,2".to_string());
    }

    let x = input[0]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse x: {}", e))?;
    let y = input[1]
        .parse::<i8>()
        .map_err(|e| format!("Cannot parse y: {}", e))?;
    Ok(Vec2::new(x, y))
}

fn write_success_result(
    duration: Duration,
    moves: Vec<Move>,
    output: &mut dyn Write,
) -> std::io::Result<()> {
    writeln!(
        output,
        "Total run time = {:.4} seconds.",
        duration.as_secs_f32()
    )?;
    writeln!(output, "An optimal solution has {} moves:", moves.len())?;
    let moves = moves
        .into_iter()
        .map(|(id, dir)| {
            let dir = match dir {
                board::Dir::Up => 'U',
                board::Dir::Down => 'D',
                board::Dir::Left => 'L',
                board::Dir::Right => 'R',
            };
            format!("{}{} ", id, dir)
        })
        .collect::<String>();
    writeln!(output, "{}", &moves)?;

    Ok(())
}

fn write_fail_result(output: &mut dyn Write) -> std::io::Result<()> {
    writeln!(output, "no solution")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let cli = Cli::parse();
    pretty_env_logger::init();
    match cli.command {
        Command::Search {
            input,
            output,
            algorithm,
        } => {
            let board = fs::read_to_string(input)?
                .parse::<Board>()
                .expect("Invalid input file");
            let mut output = get_output(output);
            let moves = match algorithm {
                Algorithm::IDDFS => search::iddfs(board),
                Algorithm::IDAStar => search::idastar(board),
                Algorithm::Manual => search::manual(board),
            };
            match moves {
                Some(moves) => {
                    let duration = start.elapsed();
                    write_success_result(duration, moves, &mut output)?;
                }
                None => {
                    write_fail_result(&mut output)?;
                }
            }
        }
        Command::Generate {
            output,
            size,
            block_count,
            shuffle_round,
        } => {
            let mut rng = thread_rng();
            // Generate IDs to be filled
            let mut next_id = 1;
            let mut possible_block_sizes = vec![
                Vec2::new(2, 1),
                Vec2::new(1, 1),
                Vec2::new(1, 2),
                Vec2::new(2, 2),
            ];
            let mut grid = Matrix2D::fill(size, 0i8);

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
            // Write to output file
            let mut output = get_output(output);
            let grid = board.id_grid();
            writeln!(output, "{} {}", size.y, size.x)?;
            for row in grid.chunks(size.x as usize) {
                let row = row
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                writeln!(output, "{}", row)?;
            }
        }
    }

    Ok(())
}

fn get_output(output: Option<String>) -> std::io::BufWriter<Box<dyn Write>> {
    let output: Box<dyn Write> = match output {
        // FIXME: Handle file creation error
        Some(output) => Box::new(fs::File::create(output).unwrap()),
        None => Box::new(std::io::stdout()),
    };
    std::io::BufWriter::new(output)
}
