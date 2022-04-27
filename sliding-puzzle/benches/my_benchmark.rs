use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sliding_puzzle_core::{Board, Move, Vec2};
use sliding_puzzle_search::search;

fn generate_board_with_exact_step(
    size: Vec2,
    block_count: i8,
    shuffle_round: usize,
    step: usize,
) -> Result<Board, String> {
    let mut remain_try = 128;
    let board = loop {
        let mut board = Board::generate(size, block_count, shuffle_round);
        let moves = search::idastar(board.clone()).unwrap_or_default();

        if moves.len() < step {
            remain_try -= 1;
            if remain_try == 0 {
                return Err("Max tries reached".to_string());
            }
            continue;
        }

        let diff = moves.len() - step;
        if diff > 0 {
            for (id, dir) in moves.into_iter().take(diff) {
                board.move_block(id, dir)?;
            }
        }

        break board;
    };

    Ok(board)
}

fn my_search_bench<SF>(
    group_name: String,
    function_name: String,
    board_params: Vec<(Vec2, i8)>,
    shuffles: Vec<usize>,
    search_fn: SF,
) -> impl FnOnce(&mut Criterion)
where
    SF: Fn(Board) -> Option<Vec<Move>>,
{
    move |c: &mut Criterion| {
        let mut group = c.benchmark_group(group_name);
        for (size, block_count) in board_params {
            for shuffle in &shuffles {
                let label = format!("{:02}x{:02}@{:02}", size.x, size.y, shuffle);
                let board = match generate_board_with_exact_step(
                    size,
                    block_count,
                    shuffle * 3,
                    *shuffle,
                ) {
                    Ok(board) => board,
                    Err(e) => panic!("label {}: {}", label, e),
                };
                let benchmark_id = BenchmarkId::new(&function_name, &label);
                group.bench_with_input(benchmark_id, &board, |bencher, board| {
                    bencher.iter(|| search_fn(board.clone()))
                });
            }
        }
    }
}

fn bnech_iddfs(c: &mut Criterion) {
    let group_name = "sliding-puzzle".to_string();
    let function_name = "IDS".to_string();
    let board_params = vec![(Vec2::new(5, 5), 8), (Vec2::new(8, 8), 24)];
    let shuffles = vec![4, 6];
    my_search_bench(
        group_name,
        function_name,
        board_params,
        shuffles,
        search::iddfs,
    )(c);
}

fn bench_idastar(c: &mut Criterion) {
    let group_name = "sliding-puzzle".to_string();
    let function_name = "IDA*".to_string();
    let board_params = vec![
        (Vec2::new(5, 5), 8),
        (Vec2::new(8, 8), 24),
        (Vec2::new(16, 16), 96),
    ];
    let shuffles = vec![4, 8, 12];
    my_search_bench(
        group_name,
        function_name,
        board_params,
        shuffles,
        search::idastar,
    )(c);
}

criterion_group!(benches, bench_idastar, bnech_iddfs);
criterion_main!(benches);
