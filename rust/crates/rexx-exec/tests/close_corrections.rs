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

fn check(source: &[u8], expected: &[u8]) -> Vec<u8> {
    let mut trace = None;
    for result in [
        run_program("probe.rex", source.to_vec(), Invocation::none()),
        run_program_collect_every_alloc("probe.rex", source.to_vec(), Invocation::none()),
    ] {
        assert_eq!(result.exit_code, 0, "{:?}", result.stderr);
        assert_eq!(result.stdout, expected);
        if let Some(previous) = &trace {
            assert_eq!(&result.stderr, previous);
        }
        trace = Some(result.stderr);
    }
    trace.unwrap()
}

#[test]
fn constant_result_failure_restores_the_callers_activation() {
    check(
        include_bytes!("../../../corpus/lang/constant_result_cleanup.rex"),
        b"caught 98.900\nafter\n",
    );
}

#[test]
fn user_raise_keeps_additional_and_array_items_without_rendering() {
    check(
        include_bytes!("../../../corpus/lang/raise_additional_items.rex"),
        b"after additional\nafter array\n",
    );
}

#[test]
fn method_collections_send_supplier_and_allow_reentrant_installation() {
    check(
        include_bytes!("../../../corpus/lang/define_methods_supplier_override.rex"),
        b"supplier\nevents DEFINE|DEFINEMETHODS\ninstalled 1 2 3\nadjacent 4 5\n",
    );
}

#[test]
fn a_copied_protected_supplier_uses_normal_security_dispatch() {
    check(
        include_bytes!("../../../corpus/lang/define_methods_protected_supplier.rex"),
        b"supplier checkpoint\ninstalled 9\n",
    );
}

#[test]
fn supplier_validation_stays_atomic_and_shared_callers_send_supplier() {
    check(include_bytes!("../../../corpus/lang/method_table_supplier_boundaries.rex"), b"bad supplier\natomic 0 8\ngood supplier\nclass method 7\ngood supplier\nenhanced 7\nplain 4\n");
}

#[test]
fn raise_substitutions_and_intermediate_trace_rendering_stay_separate() {
    let mut expected = Vec::new();
    for condition in ["user zz", "syntax 40.4"] {
        for operand in ["additional (.array~of(bad))", "array (bad)"] {
            for mode in ["off", "r", "i"] {
                let value = if condition == "user zz" && mode != "i" {
                    "raised"
                } else {
                    "98.900"
                };
                expected.extend_from_slice(
                    format!("{condition} {operand} {mode} {value}\n").as_bytes(),
                );
            }
        }
    }
    let trace = check(
        include_bytes!("../../../corpus/lang/raise_additional_modes.rex"),
        &expected,
    );
    assert_eq!(trace, include_bytes!("close_corrections/raise-modes.stderr"));
}
