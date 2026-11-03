//! The extension libraries this crate runs in Rust: `rxmath`
//! (`docs/superpowers/specs/extensions-behaviour.md`).
//!
//! A program sees each as the oracle's shared library: [`Interp::resolve_library`]
//! answers a library whose tables name the routines and methods with no
//! address ([`rexx_api::load::names_only`]) before any `dlopen`, so
//! `::REQUIRES ... LIBRARY`, `RxFuncAdd`, `Package~loadLibrary`, `EXTERNAL`
//! directives and `loadExternalRoutine`/`loadExternalMethod` bind, merge and
//! report it as any library. Where one of its rows is called, its body here
//! runs in place of a native call.
//!
//! [`Interp::resolve_library`]: crate::Interp::resolve_library

use rexx_core::ObjRef;

use crate::{Failure, Interp};

/// A routine body: the interpreter and the call's arguments.
pub(crate) type RoutineBody = fn(&mut Interp, &[Option<ObjRef>]) -> Result<ObjRef, Failure>;

/// A method body: the interpreter, the receiver, the method's scope and the
/// arguments.
pub(crate) type MethodBody =
    fn(&mut Interp, ObjRef, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// One library.
pub(crate) struct RustLibrary {
    /// The library name, matched byte for byte as a shared library's is.
    name: &'static str,
    routines: &'static [(&'static str, RoutineBody)],
    methods: &'static [(&'static str, MethodBody)],
}

impl RustLibrary {
    /// The library a load of this name answers.
    pub(crate) fn open(&self) -> rexx_api::load::Library {
        let routines: Vec<&str> = self.routines.iter().map(|(name, _)| *name).collect();
        let methods: Vec<&str> = self.methods.iter().map(|(name, _)| *name).collect();
        rexx_api::load::names_only(self.name, &routines, &methods)
    }
}

/// Every library this crate runs in Rust.
static LIBRARIES: &[RustLibrary] = &[RustLibrary {
    name: "rxmath",
    routines: &[
        ("MathLoadFuncs", crate::builtin::rxmath::load_funcs),
        ("MathDropFuncs", crate::builtin::rxmath::drop_funcs),
        ("RxCalcPi", crate::builtin::rxmath::pi),
        ("RxCalcSqrt", crate::builtin::rxmath::sqrt),
        ("RxCalcExp", crate::builtin::rxmath::exp),
        ("RxCalcLog", crate::builtin::rxmath::log),
        ("RxCalcLog10", crate::builtin::rxmath::log10),
        ("RxCalcSinH", crate::builtin::rxmath::sinh),
        ("RxCalcCosH", crate::builtin::rxmath::cosh),
        ("RxCalcTanH", crate::builtin::rxmath::tanh),
        ("RxCalcPower", crate::builtin::rxmath::power),
        ("RxCalcSin", crate::builtin::rxmath::sin),
        ("RxCalcCos", crate::builtin::rxmath::cos),
        ("RxCalcTan", crate::builtin::rxmath::tan),
        ("RxCalcCotan", crate::builtin::rxmath::cotan),
        ("RxCalcArcSin", crate::builtin::rxmath::arc_sin),
        ("RxCalcArcCos", crate::builtin::rxmath::arc_cos),
        ("RxCalcArcTan", crate::builtin::rxmath::arc_tan),
    ],
    methods: &[],
}];

/// The library `name` names, or `None` for a name this crate leaves to the
/// shared-library search.
pub(crate) fn library(name: &[u8]) -> Option<&'static RustLibrary> {
    LIBRARIES
        .iter()
        .find(|library| library.name.as_bytes() == name)
}

/// The body of a row of `library`, a library [`RustLibrary::open`] answered,
/// named `row`; `None` for any other library or row.
fn of<B: Copy>(
    library: &rexx_api::load::Library,
    row: &[u8],
    table: fn(&RustLibrary) -> &'static [(&'static str, B)],
) -> Option<B> {
    let held = LIBRARIES
        .iter()
        .find(|candidate| library.name() == Some(candidate.name.as_bytes()))?;
    table(held)
        .iter()
        .find(|(name, _)| name.as_bytes() == row)
        .map(|(_, body)| *body)
}

/// The body of `library`'s routine row named `row`, where that row carries no
/// address and the library is one of these.
pub(crate) fn routine_body(library: &rexx_api::load::Library, row: &[u8]) -> Option<RoutineBody> {
    of(library, row, |held| held.routines)
}

/// The body of `library`'s method row named `row`, as [`routine_body`].
pub(crate) fn method_body(library: &rexx_api::load::Library, row: &[u8]) -> Option<MethodBody> {
    of(library, row, |held| held.methods)
}
