//! A tag for every AST variant, and whether a program can construct it,
//! for `coverage.rs`.
#![allow(dead_code)]

use std::collections::HashSet;

use interrexx_parse::{ExprKind, InstructionKind, LoopKind, Operator, PrefixOp, Trace};

/// Whether a program can construct a variant this file enumerates.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Reach {
    /// Implemented: must be witnessed by at least one program in the corpus.
    Implemented,
    /// Structurally impossible in this position for either implementation.
    /// `Operator::Backslash` only.
    Unreachable,
}

/// Expands to a tag-and-reach function whose `match` has no wildcard arm,
/// plus the list of every `(tag, reach)` pair it can produce. One invocation
/// is the source of both, so the two cannot drift apart.
/// `interrexx-parse/tests/variants.rs`'s `tags!` produces only the tag.
macro_rules! tags {
    ($fn_name:ident, $list:ident, $ty:ty,
     { $($pat:pat => ($name:literal, $reach:expr)),+ $(,)? }
     $(, split $outer:pat in ($inner:expr) {
         $($arm:pat => ($arm_name:literal, $arm_reach:expr)),+ $(,)?
     })* $(,)?) => {
        pub(crate) fn $fn_name(k: &$ty) -> (&'static str, Reach) {
            match k {
                $($pat => ($name, $reach),)+
                $($outer => match $inner {
                    $($arm => ($arm_name, $arm_reach)),+
                },)*
            }
        }
        pub(crate) const $list: &[(&str, Reach)] =
            &[$(($name, $reach),)+ $($(($arm_name, $arm_reach),)+)*];
    };
}

tags!(instruction_tag, INSTRUCTION_TAGS, InstructionKind, {
    InstructionKind::Assignment { .. } => ("Assignment", Reach::Implemented),
    InstructionKind::Label { .. } => ("Label", Reach::Implemented),
    InstructionKind::Command { .. } => ("Command", Reach::Implemented),
    InstructionKind::Do(_) => ("Do", Reach::Implemented),
    InstructionKind::Loop(_) => ("Loop", Reach::Implemented),
    InstructionKind::If { .. } => ("If", Reach::Implemented),
    InstructionKind::Then => ("Then", Reach::Implemented),
    InstructionKind::Else { .. } => ("Else", Reach::Implemented),
    InstructionKind::Select { .. } => ("Select", Reach::Implemented),
    InstructionKind::When { .. } => ("When", Reach::Implemented),
    InstructionKind::WhenCase { .. } => ("WhenCase", Reach::Implemented),
    InstructionKind::Otherwise => ("Otherwise", Reach::Implemented),
    InstructionKind::Leave { .. } => ("Leave", Reach::Implemented),
    InstructionKind::Iterate { .. } => ("Iterate", Reach::Implemented),
    InstructionKind::End { .. } => ("End", Reach::Implemented),
    InstructionKind::Drop { .. } => ("Drop", Reach::Implemented),
    InstructionKind::Say { .. } => ("Say", Reach::Implemented),
    InstructionKind::Exit { .. } => ("Exit", Reach::Implemented),
    InstructionKind::Numeric { .. } => ("Numeric", Reach::Implemented),
    InstructionKind::Trace(_) => ("Trace", Reach::Implemented),
    InstructionKind::Nop => ("Nop", Reach::Implemented),
    InstructionKind::Interpret { .. } => ("Interpret", Reach::Implemented),
    InstructionKind::Return { .. } => ("Return", Reach::Implemented),
    InstructionKind::Procedure { .. } => ("Procedure", Reach::Implemented),
    InstructionKind::Use(_) => ("Use", Reach::Implemented),
    InstructionKind::Signal(_) => ("Signal", Reach::Implemented),
    InstructionKind::Raise(_) => ("Raise", Reach::Implemented),
    InstructionKind::Push { .. } => ("Push", Reach::Implemented),
    InstructionKind::Queue { .. } => ("Queue", Reach::Implemented),
    InstructionKind::Parse(_) => ("Parse", Reach::Implemented),
    InstructionKind::Arg(_) => ("Arg", Reach::Implemented),
    InstructionKind::Pull(_) => ("Pull", Reach::Implemented),
    InstructionKind::Message { .. } => ("Message", Reach::Implemented),
    InstructionKind::Expose { .. } => ("Expose", Reach::Implemented),
    InstructionKind::Guard(_) => ("Guard", Reach::Implemented),
    InstructionKind::Reply { .. } => ("Reply", Reach::Implemented),
    InstructionKind::Forward(_) => ("Forward", Reach::Implemented),
    InstructionKind::Address(_) => ("Address", Reach::Implemented),
    InstructionKind::Options { .. } => ("Options", Reach::Implemented),
},
split InstructionKind::Call(c) in (&**c) {
    interrexx_parse::Call::Named { .. } => ("Call::Named", Reach::Implemented),
    interrexx_parse::Call::Dynamic { .. } => ("Call::Dynamic", Reach::Implemented),
    interrexx_parse::Call::Trap(_) => ("Call::Trap", Reach::Implemented),
    interrexx_parse::Call::Qualified { .. } => ("Call::Qualified", Reach::Implemented),
},
);

tags!(expr_tag, EXPR_TAGS, ExprKind, {
    ExprKind::Literal(_) => ("Literal", Reach::Implemented),
    ExprKind::Constant(_) => ("Constant", Reach::Implemented),
    ExprKind::Variable(_) => ("Variable", Reach::Implemented),
    ExprKind::Stem(_) => ("Stem", Reach::Implemented),
    ExprKind::Compound(_) => ("Compound", Reach::Implemented),
    ExprKind::DotVariable(_) => ("DotVariable", Reach::Implemented),
    ExprKind::Prefix { .. } => ("Prefix", Reach::Implemented),
    ExprKind::Binary { .. } => ("Binary", Reach::Implemented),
    ExprKind::Logical(_) => ("Logical", Reach::Implemented),
    ExprKind::Call { .. } => ("Call", Reach::Implemented),
    ExprKind::VariableReference(_) => ("VariableReference", Reach::Implemented),
    ExprKind::Message { .. } => ("Message", Reach::Implemented),
    ExprKind::List(_) => ("List", Reach::Implemented),
    ExprKind::QualifiedCall { .. } => ("QualifiedCall", Reach::Implemented),
    ExprKind::ClassResolver { .. } => ("ClassResolver", Reach::Implemented),
});

tags!(loop_tag, LOOP_TAGS, LoopKind, {
    LoopKind::Simple => ("Simple", Reach::Implemented),
    LoopKind::Forever => ("Forever", Reach::Implemented),
    LoopKind::Count(_) => ("Count", Reach::Implemented),
    LoopKind::Controlled(_) => ("Controlled", Reach::Implemented),
    LoopKind::Over { .. } => ("Over", Reach::Implemented),
    LoopKind::With { .. } => ("With", Reach::Implemented),
});

tags!(prefix_op_tag, PREFIX_OP_TAGS, PrefixOp, {
    PrefixOp::Plus => ("Plus", Reach::Implemented),
    PrefixOp::Minus => ("Minus", Reach::Implemented),
    PrefixOp::Not => ("Not", Reach::Implemented),
});

tags!(end_style_tag, END_STYLE_TAGS, interrexx_parse::EndStyle, {
    interrexx_parse::EndStyle::Do => ("Do", Reach::Implemented),
    interrexx_parse::EndStyle::LabeledDo => ("LabeledDo", Reach::Implemented),
    interrexx_parse::EndStyle::Loop => ("Loop", Reach::Implemented),
    interrexx_parse::EndStyle::Select => ("Select", Reach::Implemented),
    interrexx_parse::EndStyle::Otherwise => ("Otherwise", Reach::Implemented),
    interrexx_parse::EndStyle::LabeledOtherwise => ("LabeledOtherwise", Reach::Implemented),
});

tags!(trace_tag, TRACE_TAGS, Trace, {
    Trace::Default => ("Default", Reach::Implemented),
    Trace::Setting(_) => ("Setting", Reach::Implemented),
    Trace::Skip(_) => ("Skip", Reach::Implemented),
    Trace::Value(_) => ("Value", Reach::Implemented),
});

tags!(operator_tag, OPERATOR_TAGS, Operator, {
    Operator::Plus => ("Plus", Reach::Implemented),
    Operator::Subtract => ("Subtract", Reach::Implemented),
    Operator::Multiply => ("Multiply", Reach::Implemented),
    Operator::Divide => ("Divide", Reach::Implemented),
    Operator::IntDiv => ("IntDiv", Reach::Implemented),
    Operator::Remainder => ("Remainder", Reach::Implemented),
    Operator::Power => ("Power", Reach::Implemented),
    Operator::Abuttal => ("Abuttal", Reach::Implemented),
    Operator::Concatenate => ("Concatenate", Reach::Implemented),
    Operator::Blank => ("Blank", Reach::Implemented),
    Operator::Equal => ("Equal", Reach::Implemented),
    Operator::BackslashEqual => ("BackslashEqual", Reach::Implemented),
    Operator::GreaterThan => ("GreaterThan", Reach::Implemented),
    Operator::BackslashGreaterThan => ("BackslashGreaterThan", Reach::Implemented),
    Operator::LessThan => ("LessThan", Reach::Implemented),
    Operator::BackslashLessThan => ("BackslashLessThan", Reach::Implemented),
    Operator::GreaterThanEqual => ("GreaterThanEqual", Reach::Implemented),
    Operator::LessThanEqual => ("LessThanEqual", Reach::Implemented),
    Operator::StrictEqual => ("StrictEqual", Reach::Implemented),
    Operator::StrictBackslashEqual => ("StrictBackslashEqual", Reach::Implemented),
    Operator::StrictGreaterThan => ("StrictGreaterThan", Reach::Implemented),
    Operator::StrictBackslashGreaterThan => ("StrictBackslashGreaterThan", Reach::Implemented),
    Operator::StrictLessThan => ("StrictLessThan", Reach::Implemented),
    Operator::StrictBackslashLessThan => ("StrictBackslashLessThan", Reach::Implemented),
    Operator::StrictGreaterThanEqual => ("StrictGreaterThanEqual", Reach::Implemented),
    Operator::StrictLessThanEqual => ("StrictLessThanEqual", Reach::Implemented),
    Operator::LessThanGreaterThan => ("LessThanGreaterThan", Reach::Implemented),
    Operator::GreaterThanLessThan => ("GreaterThanLessThan", Reach::Implemented),
    Operator::And => ("And", Reach::Implemented),
    Operator::Or => ("Or", Reach::Implemented),
    Operator::Xor => ("Xor", Reach::Implemented),
    // `\` is prefix-only; a dyadic one is error 35.1 in both implementations.
    Operator::Backslash => ("Backslash", Reach::Unreachable),
});

/// One category's seen-set against its full `(tag, reach)` list.
pub(crate) struct Coverage {
    pub(crate) category: &'static str,
    pub(crate) all: &'static [(&'static str, Reach)],
    pub(crate) seen: HashSet<&'static str>,
}

impl Coverage {
    pub(crate) fn new(category: &'static str, all: &'static [(&'static str, Reach)]) -> Self {
        Coverage {
            category,
            all,
            seen: HashSet::new(),
        }
    }

    /// Implemented variants with no witness in the corpus.
    pub(crate) fn unwitnessed(&self) -> Vec<&'static str> {
        self.all
            .iter()
            .filter(|(name, reach)| *reach == Reach::Implemented && !self.seen.contains(name))
            .map(|(name, _)| *name)
            .collect()
    }
}

#[test]
fn only_backslash_is_unreachable() {
    for (category, all) in [
        ("InstructionKind", INSTRUCTION_TAGS),
        ("ExprKind", EXPR_TAGS),
        ("LoopKind", LOOP_TAGS),
        ("PrefixOp", PREFIX_OP_TAGS),
        ("EndStyle", END_STYLE_TAGS),
        ("Trace", TRACE_TAGS),
        ("Operator", OPERATOR_TAGS),
    ] {
        for (name, reach) in all {
            if *reach == Reach::Unreachable {
                assert_eq!(
                    (category, *name),
                    ("Operator", "Backslash"),
                    "only Operator::Backslash is structurally unreachable; a \
                     second Unreachable arm needs its own justification, not \
                     a copy of this one"
                );
            }
        }
    }
}
