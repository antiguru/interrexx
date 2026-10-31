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

use super::*;

impl Interp {
    /// Runs the `::CONSTANT` expression of `program`'s directive `directive`
    /// as a method of its own, as `ClassDirective::resolveConstants` does
    /// (`instructions/ClassDirective.cpp:257`): a guarded `METHOD` level named
    /// `::CONSTANT` with `run`'s receiver and scope, stopped on the directive's
    /// clause, whose executable is a `Method` of its own and which the
    /// package's `::OPTIONS TRACE` traces as a method. While a package
    /// installs, below it is the level installing the package, stopped on
    /// directive `level`, the `::CLASS` installed last.
    pub(super) fn eval_constant_expression(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        run: ConstantRun<'_>,
        directive: usize,
        expr: &Expr,
        level: Option<usize>,
    ) -> Result<ObjRef, Failure> {
        let ConstantRun {
            name,
            receiver,
            scope: class,
            executable,
            arguments,
        } = run;
        let empty_body = CodeBody::default();
        // Taken before the frame is pushed, as every activation's is.
        let caller = self.install_caller.clone().flatten();
        let start = self.package_start(id, caller.as_ref());
        let executable = match executable {
            Some(executable) => executable,
            None => self.constant_executable(id, directive, class)?,
        };
        let (line, text) = self.directive_text(id, program, &program.directives[directive]);
        let saved_clause = self.save_clause_state();
        let installing = self.install_call.map(|call_type| {
            let frame = self.roots.push_slots(0);
            let level_id = self.next_activation_id();
            let mut installing = Activation::new(
                level_id,
                Rc::clone(program),
                id,
                Rc::new(Plan::default()),
                frame,
            );
            installing.call_type = call_type;
            installing.body = None;
            installing.cold_mut().directive_clause = level;
            self.push_activation(installing);
            self.record_call_convention();
            let line = level.map_or(0, |class| {
                directive_clause(program, &program.directives[class]).0
            });
            self.enter_directive_clause(line);
            frame
        });
        let frame = self.roots.push_slots(0);
        let activation_id = self.next_activation_id();
        let name: Rc<[u8]> = Rc::from(name);
        let mut activation = Activation::method(
            activation_id,
            Rc::clone(program),
            id,
            directive,
            Rc::new(Plan::default()),
            frame,
            MethodIdentity {
                name: Rc::clone(&name),
                scope: class,
                receiver,
            },
        );
        // It starts from the package's `::OPTIONS` settings, and under
        // `NUMERIC INHERIT` from the installing activation's. Measured:
        // `::constant k (1+b)` under `::OPTIONS NOVALUE SYNTAX` is 98.986,
        // `(1/3)` under `::OPTIONS DIGITS 5` is `0.33333`, and under
        // `NUMERIC INHERIT` in a file a `DIGITS 4` package requires, `0.3333`.
        if let Some(start) = start {
            start.apply(&mut activation);
        }
        // No body: the expression is all it runs, so a call in it finds no
        // label of its own.
        activation.body = None;
        activation.cold_mut().executable = Some(executable);
        activation.cold_mut().directive_clause = Some(directive);
        let saved_context = std::mem::replace(
            &mut self.activity.call_context,
            CallContext {
                name,
                arguments: Rc::from(arguments),
                receiver: Some(receiver),
            },
        );
        self.push_activation(activation);
        // Guarded, as the method object reports, and waited for here: the
        // level runs no clause where a queued wait would be served.
        self.trace_package_invocation_entry();
        self.reserve_for_method(crate::guards::GuardKey {
            object: receiver,
            scope: class,
        });
        let reserved = self.serve_guard_wait(false);
        self.enter_directive_clause(line);
        self.record_call_convention();
        let self_slot = self.slot_of(b"SELF");
        self.set_variable(frame, self_slot, receiver);
        let super_slot = self.slot_of(b"SUPER");
        // The receiver's behaviour's super scope, as `Interp::begin_method`
        // writes it, `.nil` for the topmost scope.
        let super_scope = self.super_scope_of(receiver, class);
        self.set_variable(frame, super_slot, super_scope.unwrap_or(ObjRef::NIL));
        let result = reserved.and_then(|()| {
            self.trace_clause(line, 0, &text);
            let code = Code {
                body: &empty_body,
                symbols: &program.symbols,
                slots: &[],
                plan: None,
            };
            self.eval(&code, expr)
        }).and_then(|value| {
            if let Some(rendered) = self.result_text(value)? {
                self.trace_result(0, &rendered);
            }
            Ok(value)
        });
        self.trace_invocation_exit(|| result.as_ref().ok().copied());
        self.guard_off_at_end();
        self.activity.call_context = saved_context;
        self.pop_activation();
        self.roots.activity_mut().pop_slots(frame);
        if let Some(level_frame) = installing {
            self.pop_activation();
            self.roots.activity_mut().pop_slots(level_frame);
        }
        self.restore_clause_state(saved_clause);
        result
    }

}
