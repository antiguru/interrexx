//! The `LIBRARY REXX` entry-point registry: the method names the `REXX`
//! package exports, and what a send to a `::METHOD` bound to one of them
//! runs.
//! ```text
//! ::routine r external 'LIBRARY REXX Filespec'     rc 0, and the routine
//!                                                  runs: it resolves
//!                                                  against a routine table
//!                                                  this registry is not
//! ::routine r external 'LIBRARY zzznolib zzzr'     98.903 rc 158
//! ::attribute at external 'LIBRARY zzznolib zzz'   98.903 rc 158
//! ::method m external 'LIBRARY zzznolib zzzr'      98.903 rc 158
//! ```

use rexx_parse::{
    AttributeDirective, AttributeStyle, ExternalSpec, MethodDirective, RoutineDirective,
};

use super::{Arity, NativeMethod, ScopedBegin, native_file_path_separator, native_file_separator};
use crate::accessor_setter_name;
use crate::internal_routines::Package;

/// Which part of the interpreter an entry point belongs to.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Family {
    /// `platform/unix/TimeSupport.cpp`: the timers `.Alarm` and `.Ticker`
    /// drive.
    Timer,
    /// `streamLibrary/StreamNative.cpp`: `.Stream`'s body.
    Stream,
    /// `classes/RexxQueueMethods.cpp`: `.RexxQueue`'s body.
    Queue,
    /// `streamLibrary/FileNative.cpp`: `.File`'s body.
    File,
    /// `concurrency/CommandHandler.cpp`: `.CommandHandler`'s body.
    Command,
}

impl Family {
    /// The family's short name, used to name its corpus program and to report
    /// it. Lower case, because it names a group of entry points and not a
    /// class.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Family::Timer => "timer",
            Family::Stream => "stream",
            Family::Queue => "queue",
            Family::File => "file",
            Family::Command => "command",
        }
    }
}

/// One entry point the `REXX` package exports to `::METHOD ... EXTERNAL`.
pub(crate) struct NativeExternal {
    /// The name, spelled as `interpreter/runtime/NativeMethods.h` spells it.
    /// Matched caselessly, so the spelling is the C++'s for a reader's sake
    /// and never for the lookup's.
    pub(crate) name: &'static str,
    pub(crate) family: Family,
    /// **Not `pub(crate)`**: `dispatch::Interp::invoke` is the only reader,
    /// and a body is a [`NativeMethod`], which takes the seam's own
    /// `Cleared` and cannot be named outside this module's parent.
    pub(in crate::dispatch) body: ExternalBody,
}

/// What a send to a method bound to an entry point runs.
pub(in crate::dispatch) enum ExternalBody {
    /// A body this phase runs, with the parameter count the C++ entry
    /// declares.
    Implemented { arity: Arity, run: NativeMethod },
    /// A body that runs in the method's scope, with the parameter count the
    /// C++ entry declares.
    Scoped { arity: Arity, begin: ScopedBegin },
}

const fn implemented(
    name: &'static str,
    family: Family,
    arity: Arity,
    run: NativeMethod,
) -> NativeExternal {
    NativeExternal {
        name,
        family,
        body: ExternalBody::Implemented { arity, run },
    }
}

const fn scoped(
    name: &'static str,
    family: Family,
    arity: Arity,
    begin: ScopedBegin,
) -> NativeExternal {
    NativeExternal {
        name,
        family,
        body: ExternalBody::Scoped { arity, begin },
    }
}

/// The `REXX` package's exported method table.
static LIBRARY_REXX_METHODS: &[NativeExternal] = &[
    scoped(
        "alarm_startTimer",
        Family::Timer,
        Arity::Fixed(2),
        super::time_support::alarm_start_timer,
    ),
    scoped(
        "alarm_stopTimer",
        Family::Timer,
        Arity::Fixed(1),
        super::time_support::stop_timer,
    ),
    scoped(
        "ticker_createTimer",
        Family::Timer,
        Arity::Fixed(0),
        super::time_support::ticker_create_timer,
    ),
    scoped(
        "ticker_waitTimer",
        Family::Timer,
        Arity::Fixed(3),
        super::time_support::ticker_wait_timer,
    ),
    scoped(
        "ticker_stopTimer",
        Family::Timer,
        Arity::Fixed(1),
        super::time_support::stop_timer,
    ),
    implemented(
        "stream_init",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::init,
    ),
    implemented(
        "stream_open",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::open,
    ),
    implemented(
        "stream_chars",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::chars,
    ),
    implemented(
        "stream_lines",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::lines,
    ),
    implemented(
        "stream_position",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::position,
    ),
    implemented(
        "stream_state",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::state,
    ),
    implemented(
        "stream_description",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::description,
    ),
    implemented(
        "stream_query_position",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::query_position,
    ),
    implemented(
        "stream_charout",
        Family::Stream,
        Arity::Counted,
        super::stream::charout,
    ),
    implemented(
        "stream_charin",
        Family::Stream,
        Arity::Counted,
        super::stream::charin,
    ),
    implemented(
        "stream_linein",
        Family::Stream,
        Arity::Counted,
        super::stream::linein,
    ),
    implemented(
        "stream_lineout",
        Family::Stream,
        Arity::Counted,
        super::stream::lineout,
    ),
    implemented(
        "stream_arrayin",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::arrayin,
    ),
    implemented(
        "qualify",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::qualify,
    ),
    implemented(
        "query_exists",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_exists,
    ),
    implemented(
        "query_size",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_size,
    ),
    implemented(
        "query_time",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_time,
    ),
    implemented(
        "handle_set",
        Family::Stream,
        Arity::Fixed(1),
        super::stream::handle_set,
    ),
    implemented(
        "std_set",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::std_set,
    ),
    implemented(
        "stream_flush",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::flush,
    ),
    implemented(
        "query_handle",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_handle,
    ),
    implemented(
        "query_streamtype",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::query_streamtype,
    ),
    implemented(
        "stream_close",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::close,
    ),
    implemented(
        "stream_uninit",
        Family::Stream,
        Arity::Fixed(0),
        super::stream::uninit,
    ),
    scoped(
        "rexx_create_queue",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::create,
    ),
    scoped(
        "rexx_open_queue",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::open,
    ),
    scoped(
        "rexx_queue_exists",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::exists,
    ),
    scoped(
        "rexx_delete_queue",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::delete,
    ),
    scoped(
        "rexx_query_queue",
        Family::Queue,
        Arity::Fixed(0),
        super::queue_methods::query,
    ),
    scoped(
        "rexx_push_queue",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::push,
    ),
    scoped(
        "rexx_queue_queue",
        Family::Queue,
        Arity::Fixed(1),
        super::queue_methods::queue,
    ),
    scoped(
        "rexx_pull_queue",
        Family::Queue,
        Arity::Fixed(0),
        super::queue_methods::pull,
    ),
    scoped(
        "rexx_linein_queue",
        Family::Queue,
        Arity::Fixed(0),
        super::queue_methods::linein,
    ),
    scoped(
        "rexx_clear_queue",
        Family::Queue,
        Arity::Fixed(0),
        super::queue_methods::clear,
    ),
    implemented(
        "file_separator",
        Family::File,
        Arity::Fixed(0),
        native_file_separator,
    ),
    implemented(
        "file_path_separator",
        Family::File,
        Arity::Fixed(0),
        native_file_path_separator,
    ),
    implemented(
        "file_case_sensitive",
        Family::File,
        Arity::Fixed(0),
        super::files::case_sensitive,
    ),
    implemented(
        "file_list_roots",
        Family::File,
        Arity::Fixed(0),
        super::files::list_roots,
    ),
    implemented(
        "file_qualify",
        Family::File,
        Arity::Fixed(1),
        super::files::qualify,
    ),
    implemented(
        "file_exists",
        Family::File,
        Arity::Fixed(1),
        super::files::exists,
    ),
    implemented(
        "file_delete_file",
        Family::File,
        Arity::Fixed(1),
        super::files::delete_file,
    ),
    implemented(
        "file_delete_directory",
        Family::File,
        Arity::Fixed(1),
        super::files::delete_directory,
    ),
    implemented(
        "file_isDirectory",
        Family::File,
        Arity::Fixed(1),
        super::files::is_directory,
    ),
    implemented(
        "file_isFile",
        Family::File,
        Arity::Fixed(1),
        super::files::is_file,
    ),
    implemented(
        "file_isHidden",
        Family::File,
        Arity::Fixed(1),
        super::files::is_hidden,
    ),
    implemented(
        "file_get_last_modified",
        Family::File,
        Arity::Fixed(1),
        super::files::get_last_modified,
    ),
    implemented(
        "file_set_last_modified",
        Family::File,
        Arity::Fixed(2),
        super::files::set_last_modified,
    ),
    implemented(
        "file_set_read_only",
        Family::File,
        Arity::Fixed(1),
        super::files::set_read_only,
    ),
    implemented(
        "file_length",
        Family::File,
        Arity::Fixed(1),
        super::files::length,
    ),
    implemented(
        "file_list",
        Family::File,
        Arity::Fixed(1),
        super::files::list,
    ),
    implemented(
        "file_make_dir",
        Family::File,
        Arity::Fixed(1),
        super::files::make_dir,
    ),
    implemented(
        "file_can_read",
        Family::File,
        Arity::Fixed(1),
        super::files::can_read,
    ),
    implemented(
        "file_can_write",
        Family::File,
        Arity::Fixed(1),
        super::files::can_write,
    ),
    implemented(
        "file_rename",
        Family::File,
        Arity::Fixed(2),
        super::files::rename,
    ),
    implemented(
        "this_file_case_sensitive",
        Family::File,
        Arity::Fixed(1),
        super::files::this_file_case_sensitive,
    ),
    implemented(
        "file_get_last_accessed",
        Family::File,
        Arity::Fixed(1),
        super::files::get_last_accessed,
    ),
    implemented(
        "file_set_last_accessed",
        Family::File,
        Arity::Fixed(2),
        super::files::set_last_accessed,
    ),
    implemented(
        "file_set_writable",
        Family::File,
        Arity::Fixed(1),
        super::files::set_writable,
    ),
    implemented(
        "file_temporary_path",
        Family::File,
        Arity::Fixed(0),
        super::files::temporary_path,
    ),
    implemented(
        "file_search_path_impl",
        Family::File,
        Arity::Fixed(2),
        super::files::search_path_impl,
    ),
    implemented(
        "cmdhandler_environments",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::environments,
    ),
    implemented(
        "cmdhandler_handler",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::handler,
    ),
    implemented(
        "cmdhandler_handlers",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::handlers,
    ),
    implemented(
        "cmdhandler_register",
        Family::Command,
        Arity::Fixed(2),
        super::command_handler::register,
    ),
    implemented(
        "cmdhandler_deregister",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::deregister,
    ),
    implemented(
        "cmdcontext_isRedirected",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::is_redirected,
    ),
    implemented(
        "cmdcontext_isValid",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::is_valid,
    ),
    implemented(
        "cmdcontext_check",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::check,
    ),
    implemented(
        "cmdcontext_getVar",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::get_var,
    ),
    implemented(
        "cmdcontext_hasVar",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::has_var,
    ),
    implemented(
        "cmdcontext_setVar",
        Family::Command,
        Arity::Fixed(2),
        super::command_handler::set_var,
    ),
    implemented(
        "cmdcontext_dropVar",
        Family::Command,
        Arity::Fixed(1),
        super::command_handler::drop_var,
    ),
    implemented(
        "cmdcontext_variables",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::variables,
    ),
    implemented(
        "cmdcontext_forward",
        Family::Command,
        Arity::Fixed(2),
        super::command_handler::forward,
    ),
    implemented(
        "cmdcontext_lineIn",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::line_in,
    ),
    implemented(
        "cmdcontext_write",
        Family::Command,
        Arity::Fixed(2),
        super::command_handler::write,
    ),
    implemented(
        "cmdnative_kind",
        Family::Command,
        Arity::Fixed(0),
        super::command_handler::kind,
    ),
];

/// Every row of the registry, flattened for a caller outside this module --
/// `crate::native_entry_points` is the one, and its own doc says why the
/// registry's types stay in here.
pub(crate) fn entry_points() -> impl Iterator<Item = crate::NativeEntryPoint> {
    LIBRARY_REXX_METHODS
        .iter()
        .map(|entry| crate::NativeEntryPoint {
            entry: entry.name,
            family: entry.family.label(),
            // Every row has a body this crate runs.
            owner: None,
            implemented: true,
        })
}

/// The entry point the `REXX` package exports under `name`, or `None` for a
/// name it does not export.
pub(crate) fn entry_point(name: &[u8]) -> Option<&'static NativeExternal> {
    LIBRARY_REXX_METHODS
        .iter()
        .find(|entry| entry.name.as_bytes().eq_ignore_ascii_case(name))
}

/// The method entry point `package` exports under `name`. `REXXUTIL` exports
/// routines only: measured, oracle, `::method m external 'LIBRARY rexxutil
/// SysFileTree'` is 90.998 rc 166 and `loadExternalMethod('x', 'LIBRARY
/// rexxutil SysSleep')` answers `.nil`.
pub(crate) fn package_entry_point(
    package: Package,
    name: &[u8],
) -> Option<&'static NativeExternal> {
    match package {
        Package::Rexx => entry_point(name),
        Package::RexxUtil => None,
    }
}

/// What a `::METHOD` or `::ATTRIBUTE` directive's `EXTERNAL` names, or `None`
/// for a directive that carries no `EXTERNAL` at all.
pub(crate) enum MethodExternal {
    /// `'LIBRARY REXX [entry]'` on a `::METHOD` with no `ATTRIBUTE` keyword:
    /// one method bound to one entry point. `Ok` is the entry point the `REXX`
    /// package exports; `Err` is the name it does not export, which the
    /// install reports as 90.998.
    LibraryRexx(Result<&'static NativeExternal, Vec<u8>>),
    /// `'LIBRARY REXX [entry]'` on a `::ATTRIBUTE`, or on a `::METHOD`
    /// carrying `ATTRIBUTE`: the accessors [`attribute_binds`] derives, in the
    /// order the oracle creates them.
    Attribute(Vec<AttributeBind>),
    /// `'LIBRARY <name>'` for any name but `REXX`: a shared library to load,
    /// with the dictionary key each of its procedures fills.
    OtherLibrary {
        /// The library name as written. Matched byte for byte: measured,
        /// oracle, `LIBRARY RXREGEXP` is `98.903 Unable to load library
        /// "RXREGEXP"` where `LIBRARY rxregexp` loads.
        library: Vec<u8>,
        binds: Vec<LibraryBind>,
    },
}

/// One procedure a library-backed `EXTERNAL` names.
pub(crate) struct LibraryBind {
    /// The dictionary key this procedure fills, upcased, as
    /// [`AttributeBind::key`] is.
    pub(crate) key: Vec<u8>,
    /// The procedure to look up in the library's own table.
    pub(crate) procedure: Vec<u8>,
}

/// One accessor an attribute-shaped `EXTERNAL` binds.
pub(crate) struct AttributeBind {
    /// The dictionary key this accessor fills, upcased -- the directive's own
    /// name for a getter, that name with `=` appended for a setter. An
    /// installer pairs a key with its bind by this name rather than by
    /// position.
    pub(crate) key: Vec<u8>,
    /// The entry point the `REXX` package exports under the procedure this
    /// accessor names, or that procedure where it exports none.
    pub(crate) entry: Result<&'static NativeExternal, Vec<u8>>,
}

/// The accessors an attribute-shaped `EXTERNAL` binds, in the order the oracle
/// creates them: the getter, then the setter.
/// ```text
/// ::attribute at external 'LIBRARY REXX zzz_no_entry'      90.998 rc 166 on "GETzzz_no_entry"
/// ::attribute at external 'LIBRARY REXX'                   90.998 rc 166 on "GETAT"
/// ::attribute at get external 'LIBRARY REXX zzz_no_entry'  90.998 rc 166 on "zzz_no_entry"
/// ::attribute at get external 'LIBRARY REXX'               90.998 rc 166 on "GETAT"
/// ::attribute at get external 'LIBRARY REXX AT'            90.998 rc 166 on "GETAT"
/// ::attribute at set external 'LIBRARY REXX'               90.998 rc 166 on "SETAT"
/// ::attribute file_separator get
///                     external 'LIBRARY REXX FILE_SEPARATOR'
///                                                          90.998 rc 166 on "GETFILE_SEPARATOR"
/// ::attribute file_separator get
///                     external 'LIBRARY REXX file_separator'          rc 0
/// ::attribute at get external 'LIBRARY REXX file_separator'           rc 0
/// ::attribute at set external 'LIBRARY REXX file_separator'           rc 0
/// ::method m attribute external 'LIBRARY REXX'             90.998 rc 166 on "GETM"
/// ```
fn attribute_binds(
    package: Package,
    upper: &[u8],
    style: AttributeStyle,
    spec: &ExternalSpec,
) -> Vec<AttributeBind> {
    attribute_accessors(upper, style, spec)
        .into_iter()
        .map(|(key, procedure)| AttributeBind {
            key,
            entry: package_entry_point(package, &procedure).ok_or(procedure),
        })
        .collect()
}

/// The dictionary key and the procedure name of each accessor, which is the
/// half of [`attribute_binds`] that does not depend on the `REXX` package.
fn attribute_accessors(
    upper: &[u8],
    style: AttributeStyle,
    spec: &ExternalSpec,
) -> Vec<(Vec<u8>, Vec<u8>)> {
    let procedure: Vec<u8> = match &spec.entry {
        Some(entry) => entry.to_vec(),
        None => upper.to_vec(),
    };
    let prefixed = |prefix: &str| [prefix.as_bytes(), &procedure].concat();
    let defaulted = *procedure == *upper;
    match style {
        AttributeStyle::Both => vec![
            (upper.to_vec(), prefixed("GET")),
            (accessor_setter_name(upper), prefixed("SET")),
        ],
        AttributeStyle::Get => vec![(
            upper.to_vec(),
            if defaulted {
                prefixed("GET")
            } else {
                procedure.clone()
            },
        )],
        AttributeStyle::Set => vec![(
            accessor_setter_name(upper),
            if defaulted {
                prefixed("SET")
            } else {
                procedure.clone()
            },
        )],
    }
}

/// [`MethodExternal`] for one `::METHOD`. Half of the boundary between the
/// forms this phase binds and the forms it refuses; [`attribute_external`] is
/// the other half and the library test is shared.
pub(crate) fn method_external(method: &MethodDirective) -> Option<MethodExternal> {
    let spec = method.external.as_ref()?;
    let upper = method.name.to_ascii_uppercase();
    // Before the `ATTRIBUTE` question, because `resolveMethod` loads the
    // library before it looks for either accessor's procedure in it. Measured,
    // oracle: `::method m attribute external 'LIBRARY nosuchlib x'` is
    // `98.903 Unable to load library "nosuchlib"` at rc 158 and not 90.998.
    let Some(package) = Package::named(&spec.library) else {
        let style = method.attribute.then_some(AttributeStyle::Both);
        return Some(other_library(&upper, style, spec));
    };
    if method.attribute {
        return Some(MethodExternal::Attribute(attribute_binds(
            package,
            &upper,
            AttributeStyle::Both,
            spec,
        )));
    }
    let entry = match &spec.entry {
        Some(entry) => entry.to_vec(),
        None => upper,
    };
    Some(MethodExternal::LibraryRexx(
        package_entry_point(package, &entry).ok_or(entry),
    ))
}

/// [`MethodExternal`] for one `::ATTRIBUTE`, whose accessors are the same
/// mechanism [`method_external`] gives a `::METHOD ... ATTRIBUTE`, with the
/// `GET` and `SET` styles [`attribute_binds`] describes on top of it.
pub(crate) fn attribute_external(attribute: &AttributeDirective) -> Option<MethodExternal> {
    let spec = attribute.external.as_ref()?;
    let upper = attribute.name.to_ascii_uppercase();
    let Some(package) = Package::named(&spec.library) else {
        return Some(other_library(&upper, Some(attribute.style), spec));
    };
    Some(MethodExternal::Attribute(attribute_binds(
        package,
        &upper,
        attribute.style,
        spec,
    )))
}

/// [`MethodExternal`] for one `::ROUTINE` naming a shared library, or `None`
/// for `REGISTERED` and for the internal packages, whose routine tables are
/// not the method registry above (`runtime/InternalPackage.cpp:230`) and
/// whose entries [`internal_routine_entry`] names.
///
/// The entry defaults to the routine's own name, which the parser upcases for
/// a symbol and keeps as written for a string -- measured, oracle: `::routine
/// zzz external "LIBRARY rxmath"` is `90.999 Unable to find external routine
/// "ZZZ"` and `::routine 'zq' external "LIBRARY rxmath"` names `"zq"`.
pub(crate) fn routine_external(routine: &RoutineDirective) -> Option<MethodExternal> {
    let spec = routine.external.as_ref()?;
    if spec.registered || Package::named(&spec.library).is_some() {
        return None;
    }
    Some(other_library(&routine.name, None, spec))
}

/// The internal package and the entry point a `::ROUTINE`'s `EXTERNAL
/// "LIBRARY REXX ..."` or `"LIBRARY rexxutil ..."` names, defaulting as
/// [`routine_external`]'s does, or `None` for every other directive.
pub(crate) fn internal_routine_entry(routine: &RoutineDirective) -> Option<(Package, Vec<u8>)> {
    let spec = routine.external.as_ref()?;
    if spec.registered {
        return None;
    }
    let package = Package::named(&spec.library)?;
    Some((
        package,
        spec.entry.as_deref().unwrap_or(&routine.name).to_vec(),
    ))
}

/// [`MethodExternal::OtherLibrary`] for one directive: the accessors of an
/// attribute-shaped `EXTERNAL`, or the single procedure of every other shape.
fn other_library(
    upper: &[u8],
    style: Option<AttributeStyle>,
    spec: &ExternalSpec,
) -> MethodExternal {
    let accessors = match style {
        Some(style) => attribute_accessors(upper, style, spec),
        None => {
            let procedure = spec
                .entry
                .as_ref()
                .map_or_else(|| upper.to_vec(), |entry| entry.to_vec());
            vec![(upper.to_vec(), procedure)]
        }
    };
    MethodExternal::OtherLibrary {
        library: spec.library.to_vec(),
        binds: accessors
            .into_iter()
            .map(|(key, procedure)| LibraryBind { key, procedure })
            .collect(),
    }
}

/// The entry point one dictionary key is bound to, or `None` for a key this
/// `EXTERNAL` does not bind, for a procedure the `REXX` package does not
/// export, and for a directive carrying no `EXTERNAL`.
pub(crate) fn bound_entry(
    external: Option<&MethodExternal>,
    key: &[u8],
) -> Option<&'static NativeExternal> {
    match external? {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().ok().copied(),
        MethodExternal::Attribute(binds) => binds
            .iter()
            .find(|bind| *bind.key == *key)
            .and_then(|bind| bind.entry.as_ref().ok().copied()),
        MethodExternal::OtherLibrary { .. } => None,
    }
}

/// The first procedure an `EXTERNAL` names that the `REXX` package does not
/// export -- the one the oracle reports 90.998 for -- or `None` when every
/// accessor resolved.
pub(crate) fn unresolved_entry(external: Option<&MethodExternal>) -> Option<&[u8]> {
    match external? {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().err().map(Vec::as_slice),
        MethodExternal::Attribute(binds) => binds
            .iter()
            .find_map(|bind| bind.entry.as_ref().err().map(Vec::as_slice)),
        MethodExternal::OtherLibrary { .. } => None,
    }
}

#[cfg(test)]
mod tests;
