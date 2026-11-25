//! `InstructionKind::for_each_root` reaches a `CATCH` variable.

mod gate_walk;

use gate_walk::each_instruction;
use interrexx_parse::{InstructionKind, parse_program, with_parser_stack};

#[test]
fn a_catch_variable_is_a_root() {
    with_parser_stack(|| {
        let program = parse_program(b"do\n  nop\ncatch err = syntax\n  nop\nend\n".to_vec())
            .expect("a CATCH with a variable parses");
        let mut found = Vec::new();
        each_instruction(&program, &mut |instruction| {
            if matches!(instruction.kind, InstructionKind::Catch(_)) {
                instruction
                    .kind
                    .for_each_root(&mut |expr| found.push(program.main.expr(expr).span()));
            }
        });
        assert_eq!(found.len(), 1, "the CATCH variable is the one root");
    })
    .expect("the parser thread spawns");
}
