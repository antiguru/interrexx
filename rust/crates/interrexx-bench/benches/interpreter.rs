//! Times each `bench-programs/*.rex` file under the interpreter named by
//! `REXX_BENCH_BINARY`. It asserts nothing and gates
//! nothing -- it only establishes the numbers a performance gate compares
//! against.
//! Run against the C++ oracle with:
//! ```sh
//! REXX_BENCH_BINARY=../build/bin/rexx cargo bench -p interrexx-bench -- --save-baseline cpp-linux
//! ```

use criterion::{Criterion, criterion_group, criterion_main};
use interrexx_bench::{PROGRAMS, interpreter_under_test, program_path, programs_dir};
use interrexx_oracle::private_service::{PrivateService, Side};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

fn interrexx_rxapi(binary: &Path) -> Option<PathBuf> {
    let sibling = binary.with_file_name("rxapi");
    if !sibling.is_file() {
        return None;
    }
    // ooRexx also has a sibling named rxapi, but lacks interrexx's stop flag.
    let version = Command::new(binary).arg("-v").output().ok()?;
    (version.status.success() && version.stdout.starts_with(b"interrexx ")).then_some(sibling)
}

fn bench_interpreter(c: &mut Criterion) {
    let interpreter = interpreter_under_test();
    let cwd = programs_dir();
    // REXX_BENCH_BINARY can name either implementation or a launcher.
    let candidate =
        PrivateService::for_benchmark(Side::Crate, interrexx_rxapi(&interpreter.binary))
            .expect("private interrexx API service");
    let oracle =
        PrivateService::for_benchmark(Side::Oracle, None).expect("private ooRexx API service");
    let services: Vec<_> = candidate.iter().chain(oracle.iter()).collect();

    let mut group = c.benchmark_group("interpreter");
    // Every sample here is a whole process launch costing 0.5-2s (the sizing
    // target), so criterion's default sample_size=100
    // would take minutes per benchmark. 10 is criterion's floor; it is still
    // enough for a point estimate and a confidence interval on a
    // launch-dominated measurement.
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(30));

    for name in PROGRAMS {
        let program = program_path(name);
        let mut interpreter = interpreter.clone();
        interpreter
            .library_paths
            .extend(interrexx_bench::child::axis_library_dir(name));
        group.bench_function(*name, |b| {
            b.iter(|| {
                interpreter
                    .run_with_api_services(&program, &[], &cwd, &services)
                    .expect("interpreter runs")
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_interpreter);
criterion_main!(benches);
