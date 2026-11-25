//! `InstructionKind::for_each_root` visits the expression slots the gate
//! walk's `exprs_of_instruction` visits, in the same order.

mod gate_walk;

use gate_walk::{corpus_dir, each_instruction, exprs_of_instruction, rex_files_under};
use interrexx_parse::{Expr, Program, parse_known_program, parse_program, with_parser_stack};

/// The roots of every instruction of `program`, through both enumerations,
/// as addresses.
fn roots(program: &Program) -> (Vec<*const Expr>, Vec<*const Expr>) {
    let (mut library, mut walk) = (Vec::new(), Vec::new());
    each_instruction(program, &mut |instruction| {
        instruction
            .kind
            .for_each_root(&mut |expr| library.push(std::ptr::from_ref(expr)));
        exprs_of_instruction(&instruction.kind, &mut |expr| {
            walk.push(std::ptr::from_ref(expr));
        });
    });
    (library, walk)
}

#[test]
fn every_corpus_instruction_has_the_roots_the_gate_walk_finds() {
    with_parser_stack(|| {
        let mut compared = 0;
        for path in rex_files_under(&corpus_dir().join("..")) {
            let text = std::fs::read(&path).expect("a readable corpus file");
            let Ok(program) = parse_program(text) else {
                continue;
            };
            let (library, walk) = roots(&program);
            assert_eq!(library, walk, "{}", path.display());
            compared += library.len();
        }
        let library = parse_known_program(interrexx_lib::LIBRARY.source.to_vec())
            .expect("the library parses");
        let (found, walk) = roots(&library);
        assert_eq!(found, walk, "the library");
        compared += found.len();
        assert!(compared > 10_000, "compared only {compared} roots");
    })
    .expect("the parser thread spawns");
}
