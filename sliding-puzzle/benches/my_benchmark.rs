use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sliding_puzzle_core::{Board, Vec2};
use sliding_puzzle_search::search;

fn generate_board_with_exact_step(
    size: Vec2,
    block_count: i8,
    shuffle_round: usize,
    step: usize,
) -> Result<Board, String> {
    let mut remain_try = 64;
    let board = loop {
        let board = Board::generate(size, block_count, shuffle_round);
        let moves = search::idastar(board.clone()).unwrap_or_default().len();

        if moves != step {
            remain_try -= 1;
            if remain_try == 0 {
                return Err("Max tries reached".to_string());
            }
            continue;
        }

        break board;
    };

    Ok(board)
}

fn bnech_iddfs(c: &mut Criterion) {
    let mut group = c.benchmark_group("IDS");
    let board_params = vec![(Vec2::new(5, 5), 8), (Vec2::new(8, 8), 48)];
    let shuffles = vec![4, 8];
    for (size, block_count) in board_params {
        for shuffle in &shuffles {
            let board = Board::generate(size, block_count, *shuffle);
            let label = format!("{:02}x{:02}@{:02}", size.x, size.y, shuffle);
            group.bench_with_input(BenchmarkId::new("", &label), &board, |b, i| {
                b.iter(|| search::iddfs(i.clone()))
            });
        }
    }
}

fn bench_idastar(c: &mut Criterion) {
    let mut group = c.benchmark_group("IDA*");
    let board_params = vec![
        (Vec2::new(5, 5), 8),
        (Vec2::new(8, 8), 48),
        (Vec2::new(16, 16), 96),
    ];
    let shuffles = vec![4, 8, 12];
    for (size, block_count) in board_params {
        for shuffle in &shuffles {
            let board = Board::generate(size, block_count, *shuffle);
            let label = format!("{:02}x{:02}@{:02}", size.x, size.y, shuffle);
            group.bench_with_input(BenchmarkId::new("", &label), &board, |b, i| {
                b.iter(|| search::idastar(i.clone()))
            });
        }
    }
}

criterion_group!(benches, bench_idastar, bnech_iddfs);
criterion_main!(benches);
