//! Parse throughput on the library program the interpreter parses at every
//! start (`interrexx_lib::LIBRARY`).

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use interrexx_parse::{DirectiveKind, parse_program};
use std::time::Duration;

/// Total instructions inside a directive's own body, 0 for a directive with
/// none (an external method, an abstract method, `::OPTIONS`, ...).
fn body_len(kind: &DirectiveKind) -> usize {
    match kind {
        DirectiveKind::Method(m) => m.body.as_ref().map_or(0, |b| b.instructions.len()),
        DirectiveKind::Attribute(a) => a.body.as_ref().map_or(0, |b| b.instructions.len()),
        DirectiveKind::Routine(r) => r.body.as_ref().map_or(0, |b| b.instructions.len()),
        _ => 0,
    }
}

fn parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    // Matches the settings `interrexx-bench` and `interrexx-num` use (`rust/crates
    // /interrexx-bench/benches/interpreter.rs`, `rust/crates/interrexx-num/benches
    // /arith.rs`), and for the same reason: criterion's defaults (sample_size
    // 100, 5s measurement) cost more time than this measurement is worth.
    // Matching them keeps this run's methodology comparable with the other
    // benchmarks' -- `perf-baseline.md` has no row for `interrexx-parse`, so
    // "comparable" means the settings agree, not that this number is checked
    // against a value recorded there.
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(30));

    let library = &interrexx_lib::LIBRARY;
    group.throughput(criterion::Throughput::Bytes(library.source.len() as u64));
    group.bench_function("library", |bencher| {
        bencher.iter_batched(
            // Untimed: the owned buffer `parse_program` needs, cloned here
            // rather than inside the timed closure.
            || library.source.to_vec(),
            |text| {
                let program = parse_program(text)
                    .unwrap_or_else(|e| panic!("the library failed to parse: {e:?}"));
                // The library is mostly class bodies, so a parse that drops
                // them is not measured as a fast one.
                let nested: usize = program.directives.iter().map(|d| body_len(&d.kind)).sum();
                assert!(
                    nested > 1000,
                    "the library parsed to {nested} nested instructions"
                );
                program
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, parse);
criterion_main!(benches);
