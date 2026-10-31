/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Focused regressions for the Phase 9d close corrections.

use rexx_exec::{Invocation, run_program, run_program_collect_every_alloc};

fn check(source: &[u8], expected: &[u8]) {
    for result in [
        run_program("probe.rex", source.to_vec(), Invocation::none()),
        run_program_collect_every_alloc("probe.rex", source.to_vec(), Invocation::none()),
    ] {
        assert_eq!(result.exit_code, 0, "{:?}", result.stderr);
        assert_eq!(result.stdout, expected);
    }
}

#[test]
fn constant_result_failure_restores_the_callers_activation() {
    check(include_bytes!("../../../corpus/lang/constant_result_cleanup.rex"), b"caught 98.900\nafter\n");
}

#[test]
fn user_raise_keeps_additional_and_array_items_without_rendering() {
    check(include_bytes!("../../../corpus/lang/raise_additional_items.rex"), b"after additional\nafter array\n");
}

#[test]
fn method_collections_send_supplier_and_allow_reentrant_installation() {
    check(include_bytes!("../../../corpus/lang/define_methods_supplier_override.rex"), b"supplier\ninstalled 1 2 3\nadjacent 4 5\n");
}
