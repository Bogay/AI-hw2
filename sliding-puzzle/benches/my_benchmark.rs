use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use sliding_puzzle_core::{Board, Vec2};
use sliding_puzzle_search::search;

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sliding puzzle");
    let board_params = vec![(Vec2::new(5, 5), 8), (Vec2::new(8, 8), 48)];
    let shuffles = vec![4, 8, 12];
    for (size, block_cnt) in board_params {
        for shuffle in &shuffles {
            let board = Board::generate(size, block_cnt, *shuffle);
            let label = format!("{}x{}@{}", size.x, size.y, shuffle);
            group.bench_with_input(BenchmarkId::new("IDDFS", &label), &board, |b, i| {
                b.iter(|| search::iddfs(i.clone()))
            });
            group.bench_with_input(BenchmarkId::new("IDA*", &label), &board, |b, i| {
                b.iter(|| search::idastar(i.clone()))
            });
        }
    }
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
