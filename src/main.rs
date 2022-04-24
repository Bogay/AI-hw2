mod board;
mod matrix;
mod search;
mod vec2;

use board::{Board, Move};
use clap::{ArgEnum, Parser};
use log;
use std::fs;
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Algorithm {
    IDDFS,
    IDAStar,
    Manual,
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[clap(short, long)]
    input: String,
    /// Path to the output file, default to stdout
    #[clap(short, long)]
    output: Option<String>,
    /// Algorithm to use, default to IDDFS
    #[clap(arg_enum, short, long, default_value_t = Algorithm::IDDFS)]
    algorithm: Algorithm,
    /// Output verbose level
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
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

fn set_log_level(args: &Args) {
    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 | _ => log::LevelFilter::Trace,
    };
    log::set_max_level(log_level);
}

fn main() -> std::io::Result<()> {
    let start = Instant::now();
    let args = Args::parse();
    set_log_level(&args);
    let board = fs::read_to_string(args.input)?
        .parse::<Board>()
        .expect("Invalid input file");
    let output: Box<dyn Write> = match args.output {
        Some(output) => Box::new(fs::File::create(output).unwrap()),
        None => Box::new(std::io::stdout()),
    };
    let mut output = std::io::BufWriter::new(output);

    let moves = match args.algorithm {
        Algorithm::IDDFS => search::iddfs(board),
        Algorithm::IDAStar => todo!(),
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

    Ok(())
}
