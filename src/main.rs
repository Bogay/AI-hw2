mod search;
mod util;

use clap::{Parser, Subcommand};
use sliding_puzzle_core::{Board, Dir, Move, Vec2};
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

/// CLI entry
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Search optimal solution of given board
    Search {
        /// Path to the input file
        #[clap(short, long)]
        input: String,
        /// Path to the output file, default to stdout
        #[clap(short, long)]
        output: Option<String>,
        /// Algorithm to use, default to IDDFS
        #[clap(arg_enum, short, long, default_value_t = search::Algorithm::IDDFS)]
        algorithm: search::Algorithm,
    },
    /// Generate a board
    Generate {
        /// Path to the output file, default to stdout
        #[clap(short, long)]
        output: Option<String>,
        /// The output board size
        #[clap(short, long, parse(try_from_str = util::vec2_from_str))]
        size: Vec2,
        /// At most how many blocks should be generated
        #[clap(short = 'n', long)]
        block_count: i8,
        /// At most how many round to shuffle the board
        #[clap(long, default_value_t = 8)]
        shuffle_round: usize,
    },
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
                Dir::Up => 'U',
                Dir::Down => 'D',
                Dir::Left => 'L',
                Dir::Right => 'R',
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

/// Get output from given path. If not, use stdout
fn get_output(output: Option<String>) -> std::io::Result<std::io::BufWriter<Box<dyn Write>>> {
    let output: Box<dyn Write> = match output {
        Some(output) => Box::new(fs::File::create(output)?),
        None => Box::new(std::io::stdout()),
    };
    Ok(std::io::BufWriter::new(output))
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
            let mut output = get_output(output)?;
            match search::execute(algorithm, board) {
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
            let board = Board::generate(size, block_count, shuffle_round);
            // Write to output file
            let mut output = get_output(output)?;
            writeln!(output, "{}", board)?;
        }
    }

    Ok(())
}
