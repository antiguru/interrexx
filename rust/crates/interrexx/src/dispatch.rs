//! Message dispatch: the object model this crate resolves against, the
//! `resolve`/`invoke` pair, and the one chokepoint every invocation passes.

use std::rc::Rc;

use interrexx_classes::{ClassKind, ClassRegistry, InheritRefusal, MethodId, MethodSlot};
use interrexx_core::{
    BehaviourHandle, BehaviourId, Body, BufferState, Decoded, ObjRef, Object, ObjectMethod,
    ObjectMethods,
};
use interrexx_parse::{Access, Expr, Operator};

use crate::activation::{Activation, MethodIdentity, Replied, ReplyState, TraceEntry, body_of};
use crate::error::{FailureSite, Raised};
use crate::method_rows::MethodRows;
use crate::plan::{BodyKey, Package, ProgramId};
use crate::run::{MAX_ACTIVATION_DEPTH, Started};
use crate::scheduler::Scheduler;
use crate::{Failure, Interp, Loud};

/// The dispatch security seam.
mod seam {
    use super::{Failure, Interp, MethodId, ObjRef};

    /// Evidence that a message send passed the dispatch security seam.
    pub(super) struct Cleared(());

    /// What the seam decided about one send.
    pub(super) enum Clearance {
        /// Run the method.
        Cleared(Cleared),
        /// Run the method, which the security manager was asked about and
        /// did not answer: code it ran may have changed the receiver.
        Checked(Cleared),
        /// The security manager answered the send itself, and the method
        /// never runs.
        Answered(Option<ObjRef>),
    }

    /// **The dispatch chokepoint (D12, site one).** Every method invocation
    /// passes here, whatever kind [`super::Invocable`] finds it to be, and
    /// the security manager's `METHOD` checkpoint is asked here for a method
    /// the send has found to be `PROTECTED`.
    pub(super) fn clear(
        interp: &mut Interp,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        method: MethodId,
    ) -> Result<Clearance, Failure> {
        let cleared = Cleared(());
        // `security_armed` first: a process that never installed a manager
        // pays one flag test per send and no row lookup.
        if interp.security_armed
            && !std::mem::take(&mut interp.unasked_run)
            && interp.method_is_protected(method)
        {
            if let Some(result) = interp.check_protected_method(receiver, name, args)? {
                return Ok(Clearance::Answered(result));
            }
            return Ok(Clearance::Checked(cleared));
        }
        Ok(Clearance::Cleared(cleared))
    }
}

use seam::Cleared;

// The `LIBRARY REXX` entry-point registry: the names a
// `::METHOD ... EXTERNAL 'LIBRARY REXX name'` binds to at install. A child of this module rather than a
// sibling, because the table's rows point at `NativeMethod`s, whose
// parameter list names a type only this module can.
pub(crate) mod native;

// `String`'s primitive methods, whose rows are chained into
// `ObjectModel::build` beside `NATIVE_METHODS` rather than merged into it,
// except the rows `NATIVE_METHODS` itself holds.
mod string;
use string::{
    native_length, native_reverse, native_string_make_string, native_string_makearray,
    native_string_sign, native_string_upper,
};

// `Stream`'s `LIBRARY REXX` entry points. A child of this module because a
// body takes the seam's `Cleared`, which cannot be named outside it.
pub(crate) mod stream;

// `.Alarm`'s and `.Ticker`'s `LIBRARY REXX` entry points.
mod time_support;

// `.RexxQueue`'s entry points.
pub(crate) mod queue_methods;

// `EventSemaphore`'s and `MutexSemaphore`'s methods.
mod semaphore;
use semaphore::{
    native_event_is_posted, native_event_post, native_event_reset, native_event_wait,
    native_mutex_acquire, native_mutex_release, native_mutex_uninit,
};

/// `.File`'s entry points, whose Rexx half hands each one an already-qualified
/// path.
mod files;

// `.CommandHandler`'s and `.CommandContext`'s entry points.
mod command_handler;
pub(crate) use command_handler::new_identity;
use files::{native_file_path_separator, native_file_separator};
pub(crate) use string::subclass_text;

// The collection classes' primitive methods, chained the same way.
mod collection;
pub(crate) mod hash;
use hash::{
    native_hash_at, native_hash_bracket_at, native_hash_bracket_put, native_hash_put,
    native_hash_unknown,
};

// `RexxInfo`'s readers, chained the same way.
mod rexx_info;

// `Method`'s and `Routine`'s own readers, chained the same way.
pub(crate) mod executable;

// `Class`'s graph readers and `Object`'s three, chained the same way.
mod introspection;

// `RexxContext`'s and `StackFrame`'s readers, chained the same way.
pub(crate) mod context;

// `Package`'s readers and its four writes, chained the same way.
mod package;
use package::{native_package_local, native_package_name};

// Running a procedure of a loaded shared library, and the `Host` the boundary
// reaches this interpreter through.
pub(crate) mod library;
pub(crate) mod regexp_methods;

// Array slot and subscript arithmetic, and `Array`'s primitive methods.
mod array;
use array::{
    IndexUse, MAX_FIXED_ARRAY_SIZE, array_dimensions, array_position, array_slots,
    array_slots_owned, native_array_at, native_array_at_for, native_array_dimension,
    native_array_items, native_array_make_string, native_array_new, native_array_of,
    native_array_put, native_array_size, positive_index, unsigned_index,
};

// `MutableBuffer`'s primitive methods, and the argument and byte-search helpers
// the other primitive methods share, chained the same way.
mod buffer;
use buffer::{
    abbrev_arguments, array_of_texts, backward_search, backward_search_arguments, bit_arguments,
    case_shift_arguments, changestr_arguments, compare_arguments, compare_to_arguments,
    conversion_length_argument, copies_argument, count_method_argument, datatype_option_argument,
    delete_arguments, delword_arguments, ends_with, equals_argument, forward_search,
    forward_search_arguments, insert_arguments, match_region_arguments, match_region_over,
    native_mutable_buffer_new, overlay_arguments, pad_arguments, replace_at_bytes, replace_at_plan,
    space_arguments, starts_with, strip_arguments, substr_arguments, translate_arguments,
    verify_arguments, wordpos_arguments,
};

// The method-argument parsers the primitive methods share.
mod method_arguments;
mod method_entry;
use method_arguments::{
    named_string_argument, option_method_argument, optional_length_argument,
    optional_position_argument, pad_method_argument, refuse_method_argument,
    required_position_argument, string_method_argument, usize_or_refuse, whole_method_argument,
};

// `Class`'s own methods: its readers, the mutators, the class factory.
mod class_protocol;
use class_protocol::{
    SourceTaker, begin_init, class_argument, class_receiver, compile_method_source,
    compile_method_source_in, compile_routine_source, is_enhanced_instance, method_name_argument,
    method_source_lines, native_annotation, native_annotations, native_base_class,
    native_class_copy, native_class_default_name, native_class_inherit, native_define,
    native_define_class_method, native_define_methods, native_delete, native_enhanced, native_id,
    native_inherit_instance_methods, native_is_subclass_of, native_metaclass, native_method,
    native_mixin_class_factory, native_new, native_new_class, native_package,
    native_package_add_class, native_package_add_public_class, native_scope, native_subclass,
    native_superclass, native_superclasses, native_uninherit, new_instance, scoped_method,
};
pub(crate) use class_protocol::{builds_in_place, charge_class, new_subclass};

// The native constructors, and `Pointer`'s and `WeakReference`'s methods,
// chained the same way.
mod construct;
use construct::{
    native_capacity_init, native_directory_new, native_executable_new, native_hash_collection_new,
    native_load_external, native_message_new, native_new_file, native_stem_new, native_string_new,
    native_unsupported_new, native_weak_reference_new, pointer_address,
};

// `Object`'s own methods, `run`/`send`/`start`, and the `Message` readers.
mod object_protocol;
use object_protocol::{
    ArrayArgument, array_argument, array_argument_object, decode_message_name, hash_value,
    invoke_executable, native_class, native_copy, native_default_name, native_has_method,
    native_hash_code, native_identity_hash, native_is_a, native_is_nil, native_message_arguments,
    native_message_completed, native_message_error_condition, native_message_halt,
    native_message_has_error, native_message_has_result, native_message_message_complete,
    native_message_name, native_message_notify, native_message_reply, native_message_reply_with,
    native_message_result, native_message_send, native_message_send_with, native_message_start,
    native_message_start_with, native_message_target, native_message_wait, native_no_op,
    native_object_concat, native_object_concat_blank, native_object_different,
    native_object_identical, native_object_name, native_object_name_set, native_request,
    native_run, native_send, native_send_with, native_set_method, native_start, native_start_with,
    native_string, native_unset_method, operator_argument, string_hash,
};

// The required-string protocol and the string conversion behind it.
mod reqstr;
pub(crate) use reqstr::{DEFAULTNAME, MAKESTRING, OBJECTNAME, STRING};
use reqstr::{required_string_argument, required_string_named_argument};

/// One primitive method's implementation.
type NativeMethod =
    fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// What a resolved [`MethodId`] runs.
enum Invocable {
    Native(NativeEntry),
    Rexx(crate::InstalledMethodBody),
    Generated(crate::GeneratedMethod),
    /// A `::METHOD ... EXTERNAL 'LIBRARY REXX name'` bound at install to a
    /// row of [`native`]'s registry, whose body runs with the method's
    /// resolution and scope, which a primitive's does not.
    External(&'static native::NativeExternal),
    /// A `::METHOD`/`::ATTRIBUTE ... EXTERNAL 'LIBRARY <name>'` bound at
    /// install to a procedure of a loaded shared library.
    Library(crate::LibraryBinding),
}

/// The begin half of a primitive method that runs a Rexx activation or parks
/// the activity, which answers [`NativeStarted::Entered`] with that activation
/// pushed or the park on [`crate::activity::Activity::native_park`].
type NativeBegin =
    fn(&mut Interp, Cleared, ObjRef, &[Option<ObjRef>]) -> Result<NativeStarted, Failure>;

/// A `LIBRARY REXX` entry point that reads or writes the receiver's variables
/// in the method's scope, its fourth parameter, and may park as a
/// [`NativeBegin`] does.
type ScopedBegin =
    fn(&mut Interp, Cleared, ObjRef, ObjRef, &[Option<ObjRef>]) -> Result<NativeStarted, Failure>;

/// What [`Interp::invoke`] needs about one primitive method beyond its code.
#[derive(Copy, Clone)]
struct NativeEntry {
    arity: Arity,
    body: NativeBody,
}

/// A primitive `UNKNOWN`'s work run without a send.
type Unsent = fn(&mut Interp, ObjRef, &[Option<ObjRef>]) -> Result<Option<ObjRef>, Failure>;

/// One primitive method's code.
#[derive(Copy, Clone)]
enum NativeBody {
    Run(NativeMethod),
    Begin(NativeBegin),
    /// A [`NativeBody::Run`] that runs Rexx code under a stack frame of its
    /// own.
    Leveled(NativeMethod),
    /// A [`NativeBody::Leveled`] whose Rexx code is only an argument's own
    /// `==` or `HASHCODE`, so it takes its level only for an argument not of
    /// a base class.
    Compared(NativeMethod),
}

/// The primitives that render their items by `stringValue`, which sends a
/// `DEFAULTNAME` of an item's own.
const RENDERING_METHODS: &[(&str, &str)] = &[("Array", "MAKESTRING"), ("Array", "TOSTRING")];

/// The collection primitives that compare or hash an argument
/// (`equalValue`, `isEqual` and `hash` in `ArrayClass`, `ListContents`,
/// `HashContents`, `StemClass` and `DirectoryClass`), which a `==` or
/// `HASHCODE` of the argument's own runs under. `Table~at` and `~[]` take
/// their level in `hash::hash_at`, whose row `Directory` shares.
const COMPARED_METHODS: &[(&str, &str)] = &[
    ("Array", "INDEX"),
    ("Array", "REMOVEITEM"),
    ("List", "HASITEM"),
    ("List", "INDEX"),
    ("List", "REMOVEITEM"),
    ("Table", "HASINDEX"),
    ("Table", "HASITEM"),
    ("Table", "INDEX"),
    ("Table", "REMOVE"),
    ("Table", "REMOVEITEM"),
    ("Set", "HASITEM"),
    ("Set", "REMOVEITEM"),
    ("Relation", "ALLAT"),
    ("Relation", "ALLINDEX"),
    ("Relation", "ITEMS"),
    ("Relation", "REMOVEALL"),
    ("Relation", "HASITEM"),
    ("Relation", "REMOVEITEM"),
    ("Bag", "HASITEM"),
    ("Bag", "REMOVEITEM"),
    ("Stem", "HASITEM"),
    ("Stem", "INDEX"),
    ("Stem", "REMOVEITEM"),
];

/// What a [`NativeBegin`] left.
pub(crate) enum NativeStarted {
    Ran(Option<ObjRef>),
    /// A Rexx activation is pushed, or the activity's park recorded, and
    /// `Then` is the native's own work once it has ended or woken.
    Entered(Then),
}

/// A primitive method's work after the Rexx activation it entered has ended.
#[derive(Clone, Copy)]
pub(crate) enum Then {
    /// The activation's answer.
    Pass,
    /// This object, whatever the activation answered.
    Answer(ObjRef),
    /// This started message, with the activation's outcome recorded on it.
    Started(ObjRef),
    /// The activation's answer, with its outcome recorded on this message.
    Held(ObjRef),
    /// Nothing, with the activation's outcome recorded on this message.
    Triggered(ObjRef),
}

impl Then {
    /// Whether this records an outcome on `message`.
    fn records(&self, message: ObjRef) -> bool {
        matches!(self, Then::Started(object) | Then::Held(object) | Then::Triggered(object) if *object == message)
    }
}

/// A [`Then`] parked on [`Activity::native_tails`] until the activation whose
/// [`CallTail`] sits at `depth` ends.
///
/// [`Activity::native_tails`]: crate::activity::Activity
/// [`CallTail`]: crate::run::CallTail
pub(crate) struct NativeTail {
    /// `Activity::call_tails`' length with that tail on top.
    depth: usize,
    then: Then,
    /// The method's traceback line for a failure, where a send made it.
    blame: Option<NativeBlame>,
    /// The activation the method entered, which `.context~stackFrames`
    /// lists the method's own frame behind.
    entered: Option<crate::activation::ActivationId>,
}

/// What [`Interp::blame_native_method`] names for one send.
struct NativeBlame {
    name: Box<[u8]>,
    scope: ObjRef,
    receiver: ObjRef,
    args: Box<[Option<ObjRef>]>,
}

impl Then {
    /// Appends the `ObjRef` this holds to `out`.
    pub(crate) fn object_roots(self, out: &mut Vec<ObjRef>) {
        match self {
            Then::Pass => {}
            Then::Answer(object)
            | Then::Started(object)
            | Then::Held(object)
            | Then::Triggered(object) => out.push(object),
        }
    }
}

impl NativeBlame {
    fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.extend([self.scope, self.receiver]);
        out.extend(self.args.iter().flatten().copied());
    }
}

impl NativeTail {
    /// Appends every `ObjRef` this tail holds to `out`.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        self.then.object_roots(out);
        if let Some(blame) = &self.blame {
            blame.object_roots(out);
        }
    }

    /// Whether this records its activation's outcome on `message`.
    pub(crate) fn records(&self, message: ObjRef) -> bool {
        self.then.records(message)
    }
}

impl Interp {
    /// Runs the primitive method `run`, which the dispatch reached as
    /// `resolution` under the message `name`, as a level of its own that
    /// `.context~stackFrames` lists while the Rexx code it calls runs.
    #[cold]
    #[inline(never)]
    fn run_leveled(
        &mut self,
        run: NativeMethod,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let caller = self.running_activation().map(|activation| activation.id);
        self.activity
            .method_levels
            .push(crate::activity::MethodLevel {
                caller,
                name: name.into(),
                resolution: Some(resolution),
                receiver,
                arguments: args.into(),
            });
        let ran = pinned!(
            self,
            crate::pinning::PinKind::native(name),
            run(self, cleared, receiver, args)
        );
        self.activity.method_levels.pop();
        self.serve_posted_halt();
        ran
    }

    /// Runs `body`, which a primitive method `name` of `receiver` runs and
    /// which may reach Rexx code, as that method's own level, which
    /// `.context~stackFrames` lists: the `InternalActivationFrame` every
    /// primitive runs under. The level's method
    /// is found when a frame is built ([`Interp::running_primitive`]).
    #[cold]
    #[inline(never)]
    pub(crate) fn primitive_level<T>(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        body: impl FnOnce(&mut Interp) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let caller = self.running_activation().map(|activation| activation.id);
        self.activity
            .method_levels
            .push(crate::activity::MethodLevel {
                caller,
                name: name.into(),
                resolution: None,
                receiver,
                arguments: args.into(),
            });
        let ran = body(self);
        self.activity.method_levels.pop();
        ran
    }

    /// The primitive method `name` that runs for `receiver`: the first method
    /// of that name in `receiver`'s search order that is a primitive, past
    /// any Rexx override that forwarded to it, or `None` where none is.
    pub(crate) fn running_primitive(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
    ) -> Option<Resolution> {
        let mut found = self.lookup(receiver, name, None)?;
        while self.object_model().natives.get(found.method).is_none() {
            let above = self.super_scope_of(receiver, found.scope)?;
            found = self.lookup(receiver, name, Some(above))?;
        }
        Some(found)
    }

    /// The `StackFrame` of the primitive method that entered the running
    /// activation and its `Method` object (`.nil`
    /// where this crate has none), where one did.
    pub(crate) fn entering_native_frame(&mut self) -> Option<(ObjRef, ObjRef)> {
        let depth = self.activity.call_tails.len();
        let row = self.activity.native_tails.len().checked_sub(1)?;
        if self.activity.native_tails[row].depth != depth {
            return None;
        }
        self.entering_frame_at(row)
    }

    /// The rows of [`Activity::native_tails`](crate::activity::Activity)
    /// whose sent primitive method entered activation `entered`, innermost
    /// first.
    pub(crate) fn entering_tails(
        &self,
        entered: crate::activation::ActivationId,
    ) -> impl Iterator<Item = usize> + '_ {
        self.activity
            .native_tails
            .iter()
            .enumerate()
            .rev()
            .filter(move |(_, tail)| tail.blame.is_some() && tail.entered == Some(entered))
            .map(|(row, _)| row)
    }

    /// The `StackFrame` of the sent primitive method in row `row` of
    /// [`Activity::native_tails`](crate::activity::Activity), and its
    /// `Method` object, or `None` for a row no send made.
    pub(crate) fn entering_frame_at(&mut self, row: usize) -> Option<(ObjRef, ObjRef)> {
        let blame = self.activity.native_tails.get(row)?.blame.as_ref()?;
        let (name, scope, receiver, args) = (
            blame.name.clone(),
            blame.scope,
            blame.receiver,
            blame.args.clone(),
        );
        let scope_name = self.scope_id(scope);
        let trace_text = Raised::compiled_method_line(&name, &scope_name);
        let executable = self.method_executable(scope, &name).ok();
        let frame = context::build_native_level_frame(
            self,
            None,
            &name,
            receiver,
            &args,
            &trace_text,
            executable,
        );
        Some((frame, executable.unwrap_or(ObjRef::NIL)))
    }
}

/// What a primitive method that parked answers once its activity wakes.
pub(crate) type NativeResume = fn(&mut Interp, ObjRef) -> Result<Option<ObjRef>, Failure>;

/// A send to a guarded method that is not a Rexx body, kept while its
/// activity waits for the guard lock.
pub(crate) struct GuardedSend {
    key: crate::guards::GuardKey,
    invocable: Invocable,
    cleared: Cleared,
    resolution: Resolution,
    receiver: ObjRef,
    name: Box<[u8]>,
    args: Vec<Option<ObjRef>>,
}

impl GuardedSend {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        out.extend(self.args.iter().flatten().copied());
    }
}

/// The send a guard wait kept, run once the lock is granted.
fn resume_guarded_send(interp: &mut Interp, _receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let Some(send) = interp.activity.guarded_send.take() else {
        return Err(Loud::scheduler_inconsistency("a guard wait with no send recorded").into());
    };
    let GuardedSend {
        key,
        invocable,
        cleared,
        resolution,
        receiver,
        name,
        args,
    } = *send;
    let frame = interp.roots.activity_mut().push_frame();
    interp.roots.activity_mut().push_temp(receiver);
    for arg in args.iter().flatten() {
        interp.roots.activity_mut().push_temp(*arg);
    }
    let started = interp.run_other_guarded(
        Some(key),
        invocable,
        cleared,
        resolution,
        receiver,
        &name,
        &args,
    );
    interp.roots.activity_mut().pop_frame(frame);
    match started? {
        Started::Ran(value) => {
            if let Some(value) = value {
                interp.roots.activity_mut().push_temp(value);
            }
            Ok(value)
        }
        Started::Entered => Err(Loud::scheduler_inconsistency(
            "a guarded method that parked again after its guard wait",
        )
        .into()),
    }
}

/// A primitive method's park: why, its continuation, and the work of each
/// primitive method between it and the driver, innermost first.
pub(crate) struct NativePark {
    reason: crate::scheduler::ParkReason,
    resume: NativeResume,
    receiver: ObjRef,
    thens: Vec<(Then, Option<NativeBlame>)>,
}

impl NativePark {
    pub(crate) fn reason(&self) -> crate::scheduler::ParkReason {
        self.reason
    }

    pub(crate) fn set_reason(&mut self, reason: crate::scheduler::ParkReason) {
        self.reason = reason;
    }

    /// Whether a primitive method between the park and the driver records
    /// its outcome on `message`.
    pub(crate) fn records(&self, message: ObjRef) -> bool {
        self.thens.iter().any(|(then, _)| then.records(message))
    }

    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        for (then, blame) in &self.thens {
            then.object_roots(out);
            if let Some(blame) = blame {
                blame.object_roots(out);
            }
        }
    }
}

/// How many arguments an entry admits.
#[derive(Copy, Clone)]
enum Arity {
    /// A numeric count. For a primitive method more than this is refused with
    /// 93.902 before the body is entered, and a shorter list is padded out with
    /// nulls: `'abc'~length(1)` is `0 expected` and `'abc'~hasMethod('a','b')`
    /// is `1 expected`.
    Fixed(usize),
    /// An uncounted entry, which hands the body the whole
    /// argument list and count and refuses nothing here.
    /// The body's own check is what
    /// reports, and its error is not always 93.902 -- measured,
    /// `(1,2)~at(1,2)` is 93.926 `Too many subscripts for array; 1 expected.`
    /// where `.environment~at(1,2)`, whose row is a count, is 93.902.
    Counted,
}

/// The primitive methods every run of this crate implements, as (class id,
/// method name, declared parameter count, implementation).
static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `Array`'s lookup under both of its names, at an uncounted arity in each
    // row -- the same both-names-one-function
    // shape `MAKESTRING`/`TOSTRING` below have, and for the same reason: a
    // second function is where the two could come to disagree. Measured, the
    // two agree on every shape `array_list_expression.rex` and
    // `array_index_refusals.rex` ask.
    ("Array", "[]", Arity::Counted, native_array_at),
    ("Array", "AT", Arity::Counted, native_array_at),
    // `Array`'s store under both of its names, the same shape again. `t[i] = v` sends `t~"[]="(v, i)`, so
    // the value leads the subscript list here too.
    ("Array", "[]=", Arity::Counted, native_array_put),
    ("Array", "PUT", Arity::Counted, native_array_put),
    // `Array~dimension`, whose declared
    // count is 1 and whose argument is optional.
    (
        "Array",
        "DIMENSION",
        Arity::Fixed(1),
        native_array_dimension,
    ),
    ("Array", "ITEMS", Arity::Fixed(0), native_array_items),
    (
        "Array",
        "MAKESTRING",
        Arity::Fixed(2),
        native_array_make_string,
    ),
    // The same function under a second name, bound
    // twice with the same arity. Measured, the
    // two agree on every shape `array_make_string.rex` and
    // `array_make_string_refusals.rex` ask -- the default, `L` with and
    // without a separator, `C`, a lower-case option, an option omitted in
    // place, the empty list, 93.915 and the `C`-with-separator 93.902 -- and
    // the traceback frame names the message that was sent rather than the
    // implementation, so `~toString` reads `Compiled method "TOSTRING" with
    // scope "Array".` A second row here rather than a second function,
    // because a second function is where the two could come to disagree.
    (
        "Array",
        "TOSTRING",
        Arity::Fixed(2),
        native_array_make_string,
    ),
    ("Array", "SIZE", Arity::Fixed(0), native_array_size),
    // `Bag`'s `init`: the creation-size handler, in `Bag`'s own dictionary,
    // the same one `IdentityTable` declares. A donation puts the method in the
    // receiving class's own dictionary, which is what the scope in the
    // traceback says, so each donee needs a row of its own here.
    ("Bag", "INIT", Arity::Fixed(1), native_capacity_init),
    // `Object`'s `init`, bound to `.Class` under this name. It is a no-op by
    // default, there to be overridden as a fill-in. Every class object answers
    // it, because a class's class behaviour merges its metaclass's instance
    // behaviour, and the `::CLASS` directive sends it to every class the
    // package installed.
    ("Class", "ACTIVATE", Arity::Fixed(0), native_no_op),
    // The comparison operators `Class` declares, each at count 1 and each an
    // identity or its negation -- the same identity test `Object`'s rows below
    // carry, bound again here because `Class` names its own and a name resolves to
    // the more specific scope. `Interp::operator_message_receiver` is the send, so
    // these answer the expression `(.Array = .Array)` and the message
    // `.Array~'='(.Array)` alike. For example, `.array = .array` is `1`, `.array =
    // 'The Array class'` is `0` -- identity, not a comparison of renderings.
    ("Class", "=", Arity::Fixed(1), native_object_identical),
    ("Class", "==", Arity::Fixed(1), native_object_identical),
    ("Class", "\\=", Arity::Fixed(1), native_object_different),
    ("Class", "\\==", Arity::Fixed(1), native_object_different),
    ("Class", "<>", Arity::Fixed(1), native_object_different),
    ("Class", "><", Arity::Fixed(1), native_object_different),
    // The same pair is bound at `Method`
    // and `Routine`, and at `Package`; each differs from the others only in
    // which field it
    // reaches for, so `Class`, `Method`, `Routine` and `Package` share one
    // implementation here for the reason `MAKESTRING`/`TOSTRING` share
    // theirs.
    ("Class", "ANNOTATION", Arity::Fixed(1), native_annotation),
    ("Class", "ANNOTATIONS", Arity::Fixed(0), native_annotations),
    ("Class", "BASECLASS", Arity::Fixed(0), native_base_class),
    (
        "Class",
        "DEFAULTNAME",
        Arity::Fixed(0),
        native_class_default_name,
    ),
    // The five mutators, each of which is protected except `Inherit`, and each of
    // which opens with the same refusal for a class defined by the system.
    // `Class~copy` is a refusal and not a copy: it overrides `Object`'s row with one
    // that raises. For example, `.K~copy` is `93.970 COPY method is not supported for
    // object The K class.`
    ("Class", "COPY", Arity::Fixed(0), native_class_copy),
    ("Class", "DEFINE", Arity::Fixed(2), native_define),
    (
        "Class",
        "DEFINEMETHODS",
        Arity::Fixed(1),
        native_define_methods,
    ),
    ("Class", "DELETE", Arity::Fixed(1), native_delete),
    ("Class", "ENHANCED", Arity::Counted, native_enhanced),
    ("Class", "HASHCODE", Arity::Fixed(0), native_hash_code),
    ("Class", "ID", Arity::Fixed(0), native_id),
    ("Class", "INHERIT", Arity::Fixed(2), native_class_inherit),
    (
        "Class",
        "ISSUBCLASSOF",
        Arity::Fixed(1),
        native_is_subclass_of,
    ),
    ("Class", "METACLASS", Arity::Fixed(0), native_metaclass),
    ("Class", "METHOD", Arity::Fixed(1), native_method),
    // `Class~mixinClass` and `Class~subclass`, the class factory a program reaches by
    // message. Neither carries the refusal for system-defined classes that the mutators
    // above open with: `subclass` does not test the flag, and what it builds does not
    // carry it either: `.object~subclass("k")~inherit(.object~mixinclass("mx"))`.
    (
        "Class",
        "MIXINCLASS",
        Arity::Fixed(3),
        native_mixin_class_factory,
    ),
    ("Class", "PACKAGE", Arity::Fixed(0), native_package),
    ("Class", "SUBCLASS", Arity::Fixed(3), native_subclass),
    ("Class", "SUPERCLASS", Arity::Fixed(0), native_superclass),
    (
        "Class",
        "SUPERCLASSES",
        Arity::Fixed(0),
        native_superclasses,
    ),
    ("Class", "UNINHERIT", Arity::Fixed(1), native_uninherit),
    // `Directory`'s own row: the `init` it inherits from the hash
    // collections, since it declares none of its own.
    ("Directory", "INIT", Arity::Fixed(1), native_capacity_init),
    // The hash collections' lookup under both of its names and
    // their store, donated to `Directory` out of
    // `StringTable`, which takes them from `IdentityTable`'s own rows. The donation puts
    // them in `Directory`'s **own** dictionary, which is what the scope in the
    // traceback says: measured, `.environment~at()` reports `Compiled method
    // "AT" with scope "Directory".`, not `IdentityTable`.
    ("Directory", "[]", Arity::Fixed(1), native_hash_bracket_at),
    // The store under its second name, donated from
    // `IdentityTable` and so the same function with
    ("Directory", "[]=", Arity::Fixed(2), native_hash_bracket_put),
    ("Directory", "AT", Arity::Fixed(1), native_hash_at),
    ("Directory", "PUT", Arity::Fixed(2), native_hash_put),
    // The string-keyed `unknown`, `StringTable`'s own row
    // and donated to `Directory` by the same donation.
    // is reached by sending its name.
    ("Directory", "UNKNOWN", Arity::Fixed(2), native_hash_unknown),
    (
        "EventSemaphore",
        "ISPOSTED",
        Arity::Fixed(0),
        native_event_is_posted,
    ),
    ("EventSemaphore", "POST", Arity::Fixed(0), native_event_post),
    (
        "EventSemaphore",
        "RESET",
        Arity::Fixed(0),
        native_event_reset,
    ),
    // `EventSemaphore~uninit`, which leaves
    // the post as it was: measured, a posted semaphore still answers
    ("EventSemaphore", "UNINIT", Arity::Fixed(0), native_no_op),
    // The creation-size `init` at the class that declares it.
    (
        "IdentityTable",
        "INIT",
        Arity::Fixed(1),
        native_capacity_init,
    ),
    (
        "Message",
        "ARGUMENTS",
        Arity::Fixed(0),
        native_message_arguments,
    ),
    (
        "Message",
        "COMPLETED",
        Arity::Fixed(0),
        native_message_completed,
    ),
    (
        "Message",
        "ERRORCONDITION",
        Arity::Fixed(0),
        native_message_error_condition,
    ),
    ("Message", "HALT", Arity::Fixed(1), native_message_halt),
    (
        "Message",
        "HASERROR",
        Arity::Fixed(0),
        native_message_has_error,
    ),
    (
        "Message",
        "HASRESULT",
        Arity::Fixed(0),
        native_message_has_result,
    ),
    (
        "Message",
        "MESSAGENAME",
        Arity::Fixed(0),
        native_message_name,
    ),
    ("Message", "NOTIFY", Arity::Fixed(1), native_message_notify),
    ("Message", "REPLY", Arity::Counted, native_message_reply),
    (
        "Message",
        "REPLYWITH",
        Arity::Fixed(2),
        native_message_reply_with,
    ),
    ("Message", "START", Arity::Counted, native_message_start),
    (
        "Message",
        "STARTWITH",
        Arity::Fixed(2),
        native_message_start_with,
    ),
    ("Message", "TARGET", Arity::Fixed(0), native_message_target),
    ("Method", "ANNOTATION", Arity::Fixed(1), native_annotation),
    ("Method", "ANNOTATIONS", Arity::Fixed(0), native_annotations),
    // `Method~scope`. `Routine` and
    // `Package` carry no such row: `Scope` is bound at `Method`
    // alone, and the scope is a field of a method rather than of any
    // executable.
    ("Method", "SCOPE", Arity::Fixed(0), native_scope),
    (
        "MutexSemaphore",
        "RELEASE",
        Arity::Fixed(0),
        native_mutex_release,
    ),
    (
        "MutexSemaphore",
        "UNINIT",
        Arity::Fixed(0),
        native_mutex_uninit,
    ),
    // Each declared at count 1. These are what an operator applied to an
    // instance resolves against when its class defines none of its own;
    // `Interp::operator_message_receiver` is the send, and the names are
    // `Operator::spelling`'s, which `corpus/lang/operator_methods.rex` checks
    // against the reference run.
    ("Object", "=", Arity::Fixed(1), native_object_identical),
    ("Object", "==", Arity::Fixed(1), native_object_identical),
    ("Object", "\\=", Arity::Fixed(1), native_object_different),
    ("Object", "\\==", Arity::Fixed(1), native_object_different),
    ("Object", "<>", Arity::Fixed(1), native_object_different),
    ("Object", "><", Arity::Fixed(1), native_object_different),
    ("Object", "||", Arity::Fixed(1), native_object_concat),
    ("Object", "", Arity::Fixed(1), native_object_concat),
    ("Object", " ", Arity::Fixed(1), native_object_concat_blank),
    ("Object", "CLASS", Arity::Fixed(0), native_class),
    ("Object", "COPY", Arity::Fixed(0), native_copy),
    (
        "Object",
        "DEFAULTNAME",
        Arity::Fixed(0),
        native_default_name,
    ),
    ("Object", "HASMETHOD", Arity::Fixed(1), native_has_method),
    ("Object", "HASHCODE", Arity::Fixed(0), native_hash_code),
    (
        "Object",
        "IDENTITYHASH",
        Arity::Fixed(0),
        native_identity_hash,
    ),
    // `Object`'s `init` under its own name. The
    // same function `ACTIVATE` above names, and a second row rather than a
    // second implementation, because a second implementation is where the two
    // could come to disagree. Class creation sends this to every class
    // it builds, so a `::CLASS` declaring no
    // `::METHOD init CLASS` of its own reaches this entry.
    ("Object", "INIT", Arity::Fixed(0), native_no_op),
    ("Object", "ISA", Arity::Fixed(1), native_is_a),
    ("Object", "ISNIL", Arity::Fixed(0), native_is_nil),
    ("Object", "OBJECTNAME", Arity::Fixed(0), native_object_name),
    (
        "Object",
        "OBJECTNAME=",
        Arity::Fixed(1),
        native_object_name_set,
    ),
    ("Object", "REQUEST", Arity::Fixed(1), native_request),
    // `Object~run`. Private, so a program context is refused by
    // the private check before this entry is reached, and restricted
    // besides -- [`check_restricted_method`].
    ("Object", "RUN", Arity::Counted, native_run),
    // `Object~setMethod` and its partner. Private, which is where the refusal a
    // program context meets comes from; `interrexx_classes::native_classes` files
    // the access scope off the same rows.
    ("Object", "SETMETHOD", Arity::Fixed(3), native_set_method),
    ("Object", "STRING", Arity::Fixed(0), native_string),
    (
        "Object",
        "UNSETMETHOD",
        Arity::Fixed(1),
        native_unset_method,
    ),
    (
        "Package",
        "ADDCLASS",
        Arity::Fixed(2),
        native_package_add_class,
    ),
    (
        "Package",
        "ADDPUBLICCLASS",
        Arity::Fixed(2),
        native_package_add_public_class,
    ),
    ("Package", "ANNOTATION", Arity::Fixed(1), native_annotation),
    (
        "Package",
        "ANNOTATIONS",
        Arity::Fixed(0),
        native_annotations,
    ),
    ("Package", "LOCAL", Arity::Fixed(0), native_package_local),
    ("Package", "NAME", Arity::Fixed(0), native_package_name),
    // `Relation`'s `init`: the creation-size handler, donated from
    // `IdentityTable`.
    ("Relation", "INIT", Arity::Fixed(1), native_capacity_init),
    ("Routine", "ANNOTATION", Arity::Fixed(1), native_annotation),
    (
        "Routine",
        "ANNOTATIONS",
        Arity::Fixed(0),
        native_annotations,
    ),
    // The same donation at `Set`.
    ("Set", "INIT", Arity::Fixed(1), native_capacity_init),
    ("String", "LENGTH", Arity::Fixed(0), native_length),
    (
        "String",
        "MAKEARRAY",
        Arity::Fixed(1),
        native_string_makearray,
    ),
    (
        "String",
        "MAKESTRING",
        Arity::Fixed(0),
        native_string_make_string,
    ),
    ("String", "REVERSE", Arity::Fixed(0), native_reverse),
    // Declared at count 0.
    ("String", "SIGN", Arity::Fixed(0), native_string_sign),
    ("String", "UPPER", Arity::Fixed(2), native_string_upper),
    // `.methods`, `.routines` and `.resources`. The same functions the
    // `Directory` rows above name, because the code is the same, reached
    // through the same donation: `StringTable` takes `[]`, `At` and `Put` from
    // `IdentityTable` and declares `Unknown` itself, and `Directory` then
    // takes that whole set from `StringTable`.
    ("StringTable", "[]", Arity::Fixed(1), native_hash_bracket_at),
    (
        "StringTable",
        "[]=",
        Arity::Fixed(2),
        native_hash_bracket_put,
    ),
    ("StringTable", "AT", Arity::Fixed(1), native_hash_at),
    // The creation-size `init`, which `~new` sends and which validates the
    // initial-size argument, in `StringTable`'s own dictionary, donated from
    // `IdentityTable`.
    ("StringTable", "INIT", Arity::Fixed(1), native_capacity_init),
    ("StringTable", "PUT", Arity::Fixed(2), native_hash_put),
    (
        "StringTable",
        "UNKNOWN",
        Arity::Fixed(2),
        native_hash_unknown,
    ),
    // `Supplier~init`, which is where
    // `.Supplier~new`'s refusal comes from: the two arrays are required and
    // the allocation above it is not what raises.
    // The same donation at `Table`.
    ("Table", "INIT", Arity::Fixed(1), native_capacity_init),
    // `VariableReference`'s own rows, the
    // whole of what its class defines beyond `Object`'s.
    (
        "VariableReference",
        "NAME",
        Arity::Fixed(0),
        native_reference_name,
    ),
    (
        "VariableReference",
        "VALUE",
        Arity::Fixed(0),
        native_reference_value,
    ),
    (
        "VariableReference",
        "VALUE=",
        Arity::Fixed(1),
        native_reference_value_set,
    ),
    (
        "VariableReference",
        "UNKNOWN",
        Arity::Fixed(2),
        native_reference_unknown,
    ),
    (
        "VariableReference",
        "REQUEST",
        Arity::Fixed(1),
        native_reference_request,
    ),
];

/// The primitive methods that send a message the receiver may answer with a
/// Rexx body, or that park the activity, entered by their begin halves.
static RESUMABLE_METHODS: &[(&str, &str, Arity, NativeBegin)] = &[
    // `Message~completed`, bound under both names.
    (
        "Message",
        "MESSAGECOMPLETE",
        Arity::Fixed(1),
        native_message_message_complete,
    ),
    (
        "Message",
        "TRIGGERED",
        Arity::Fixed(1),
        native_message_message_complete,
    ),
    ("Message", "RESULT", Arity::Fixed(0), native_message_result),
    ("Message", "SEND", Arity::Counted, native_message_send),
    (
        "Message",
        "SENDWITH",
        Arity::Fixed(2),
        native_message_send_with,
    ),
    ("Object", "SEND", Arity::Counted, native_send),
    ("Object", "SENDWITH", Arity::Fixed(2), native_send_with),
    ("Object", "START", Arity::Counted, native_start),
    ("Object", "STARTWITH", Arity::Fixed(2), native_start_with),
    ("Message", "WAIT", Arity::Fixed(0), native_message_wait),
    ("EventSemaphore", "WAIT", Arity::Fixed(1), native_event_wait),
    (
        "MutexSemaphore",
        "ACQUIRE",
        Arity::Fixed(1),
        native_mutex_acquire,
    ),
];

/// [`RESUMABLE_METHODS`] for the class dictionary: every `NEW` that sends
/// `INIT`.
static RESUMABLE_CLASS_METHODS: &[(&str, &str, Arity, NativeBegin)] = &[
    // `Object`'s `new`, reached by every class whose own class
    // behaviour declares no `NEW` of its own, and the classes below whose
    // allocation builds a body this crate does not model.
    ("Object", "NEW", Arity::Counted, native_new),
    ("EventSemaphore", "NEW", Arity::Counted, native_new),
    ("List", "NEW", Arity::Counted, native_new),
    ("MutexSemaphore", "NEW", Arity::Counted, native_new),
    ("Queue", "NEW", Arity::Counted, native_new),
    ("Supplier", "NEW", Arity::Counted, native_new),
    // `Array`'s `new`. A row of its own for the reason
    // `StringTable`'s below is one: the answer is a body this crate builds
    // rather than an instance.
    ("Array", "NEW", Arity::Counted, native_array_new),
    // `StringTable`'s and `Directory`'s `new`. Rows of their own rather than
    // `Object`'s, because each allocates a hash body rather than an instance.
    (
        "StringTable",
        "NEW",
        Arity::Counted,
        native_hash_collection_new,
    ),
    ("Directory", "NEW", Arity::Counted, native_directory_new),
    // `Method`'s and `Routine`'s `new`, and the
    // two `newFile` rows beside them: a compiled executable, then `INIT`.
    ("Method", "NEW", Arity::Counted, native_executable_new),
    ("Routine", "NEW", Arity::Counted, native_executable_new),
    ("Method", "NEWFILE", Arity::Fixed(2), native_new_file),
    ("Routine", "NEWFILE", Arity::Fixed(2), native_new_file),
    // Every class whose own `new` is an allocation followed by
    // the standard initialisation and nothing else.
    ("Bag", "NEW", Arity::Counted, hash::native_hash_new),
    // `Table`'s and `IdentityTable`'s `new` take an optional
    // initial capacity, and it is observable: it decides the bucket count,
    // which decides the order every iteration answers in.
    (
        "IdentityTable",
        "NEW",
        Arity::Counted,
        hash::native_hash_new,
    ),
    ("Relation", "NEW", Arity::Counted, hash::native_hash_new),
    ("Set", "NEW", Arity::Counted, hash::native_hash_new),
    ("Table", "NEW", Arity::Counted, hash::native_hash_new),
    // `.Class`'s `new`, the one class method `.Class` adds.
    ("Class", "NEW", Arity::Counted, native_new_class),
    ("Message", "NEW", Arity::Counted, native_message_new),
    (
        "WeakReference",
        "NEW",
        Arity::Counted,
        native_weak_reference_new,
    ),
    // `String`'s `new`. The one row here whose answer is a value rather
    // than an instance.
    ("String", "NEW", Arity::Counted, native_string_new),
    // `Stem`'s `new`. A row of its own because the answer is the
    // `Body::Stem` a bare stem read also produces, not an instance.
    ("Stem", "NEW", Arity::Counted, native_stem_new),
    // `MutableBuffer`'s `new`. A row of its own because the arguments are
    // checked before the allocation rather than by `INIT`.
    (
        "MutableBuffer",
        "NEW",
        Arity::Counted,
        native_mutable_buffer_new,
    ),
];

/// The primitive methods bound to a class's **class** dictionary rather than
/// its instance one.
static NATIVE_CLASS_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `Array~of`. The same body as `Array~new`'s, filled from the
    // arguments rather than sized from them.
    ("Array", "OF", Arity::Counted, native_array_of),
    // `Queue~of` and `List~of`. One body: each
    // creates a collection of the receiver's own class and appends the
    // arguments to it.
    (
        "Queue",
        "OF",
        Arity::Counted,
        collection::native_collection_of,
    ),
    (
        "List",
        "OF",
        Arity::Counted,
        collection::native_collection_of,
    ),
    // `Bag~of`, which unlike
    // `Set~of` keeps the duplicates.
    ("Bag", "OF", Arity::Counted, hash::native_bag_of),
    // `Set~of`: `Set`'s own, unlike
    // the mapped classes whose `of` is `MapCollection~OF` in Rexx.
    ("Set", "OF", Arity::Counted, hash::native_set_of),
    (
        "Method",
        "LOADEXTERNALMETHOD",
        Arity::Fixed(2),
        native_load_external,
    ),
    (
        "Routine",
        "LOADEXTERNALROUTINE",
        Arity::Fixed(2),
        native_load_external,
    ),
    // The classes whose `new` is the refusal and nothing else, because
    // their instances come only from native code or from the `>name`
    // operator.
    ("Buffer", "NEW", Arity::Counted, native_unsupported_new),
    ("Pointer", "NEW", Arity::Counted, native_unsupported_new),
    ("RexxInfo", "NEW", Arity::Counted, native_unsupported_new),
    (
        "VariableReference",
        "NEW",
        Arity::Counted,
        native_unsupported_new,
    ),
];

/// The two methods put on `.Class` for the image build, which are
/// deleted before the image is saved.
static SETUP_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Class",
        "DEFINECLASSMETHOD",
        Arity::Fixed(2),
        native_define_class_method,
    ),
    (
        "Class",
        "INHERITINSTANCEMETHODS",
        Arity::Fixed(1),
        native_inherit_instance_methods,
    ),
];

/// `~defaultName` for an array -- the article rule
/// applied to `.Array`'s id.
pub(crate) const ARRAY_DEFAULT_NAME: &[u8] = b"an Array";

/// The message the search order's last step sends.
const UNKNOWN: &[u8] = b"UNKNOWN";

/// The message name a `~run` body is entered under.
const UNNAMED_METHOD: &[u8] = b"*UNNAMED*";

/// The entry a `Message` object keeps the value its send answered under.
const MESSAGE_RESULT: &[u8] = b"RESULT";

/// The entry a `Message` object keeps the condition object its send failed
/// with under.
pub(crate) const MESSAGE_CONDITION: &[u8] = b"ERRORCONDITION";

/// The entries a `Message~new` object keeps its unsent message in:
/// the `receiver`, `message`, `startscope` and `args`.
const MESSAGE_TARGET: &[u8] = b"TARGET";
const MESSAGE_NAME: &[u8] = b"MESSAGENAME";
const MESSAGE_SCOPE: &[u8] = b"SCOPE";
const MESSAGE_ARGUMENTS: &[u8] = b"ARGUMENTS";

/// The entry a `Message` object keeps the `Array` of objects `~notify` named
/// under -- its interested parties.
const MESSAGE_PARTIES: &[u8] = b"NOTIFY";

/// The message a class construction sends the class it just built.
/// Upper case for [`UNKNOWN`]'s reason.
pub(crate) const INIT: &[u8] = b"INIT";

/// The message `SAY` sends whatever `.OUTPUT` holds, whose reply is dropped.
/// Upper case for [`UNKNOWN`]'s reason.
pub(crate) const SAY: &[u8] = b"SAY";

/// The message a trace line and an error report line send whatever
/// `.TRACEOUTPUT` holds. Upper case for [`UNKNOWN`]'s reason.
pub(crate) const LINEOUT: &[u8] = b"LINEOUT";

/// The message a line read with no stream name sends whatever `.INPUT` holds.
/// Measured, `PARSE PULL` sends this too rather than a `PULL` of its own.
/// Upper case for [`UNKNOWN`]'s reason.
pub(crate) const LINEIN: &[u8] = b"LINEIN";

/// The message a finalizer delivery sends. Upper case for
/// [`UNKNOWN`]'s reason.
const UNINIT: &[u8] = b"UNINIT";

/// How many times the termination sweep runs, from the number of times shutdown
/// runs the pending finalizers. See
/// [`Interp::run_termination_uninits`](Interp::run_termination_uninits).
const SWEEPS: usize = 2;

/// The bytes a retired behaviour is charged per method entry, an estimate:
/// measured, 80000 `define`s of one class with nothing freeing them peaked at
/// 628364 kB.
const RETIRED_ENTRY_BYTES: usize = 128;

/// The message the last install pass sends every class the package built.
pub(crate) const ACTIVATE: &[u8] = b"ACTIVATE";

/// The class model this crate dispatches against, and the implementations
/// its `MethodId`s name.
pub(crate) struct ObjectModel {
    classes: ClassRegistry,
    natives: MethodRows<NativeEntry>,
    /// The classes a value this crate builds answers to, and `.Class`,
    /// resolved once at bootstrap. These are handles into `classes` and not a
    /// second model of it, and holding them buys two things: a send does not
    /// look its receiver's class up by name (which upcases and allocates) on
    /// every message, and neither a send nor a `::CLASS` install can be
    /// redirected by a program that declares a class of one of these names.
    string: ObjRef,
    object: ObjRef,
    metaclass: ObjRef,
    array: ObjRef,
    package: ObjRef,
    method: ObjRef,
    routine: ObjRef,
    directory: ObjRef,
    string_table: ObjRef,
    /// The scope a mapped collection's store is bound under.
    table: ObjRef,
    context: ObjRef,
    /// The class the native class set registers with the system and whose
    /// *instance* `.RexxInfo` answers -- see [`Primitive::RexxInfo`].
    /// Reached through `ClassRegistry::system_lookup`, since no `.NAME`
    /// resolves to it.
    rexx_info: ObjRef,
    /// The class `~start` and `~startWith` answer an instance of.
    message: ObjRef,
    /// The class a `>name` term answers an instance of.
    variable_reference: ObjRef,
    /// The class a `Body::Stem` answers, whether a bare stem read produced it
    /// or `.Stem~new` did.
    stem: ObjRef,
    /// The scope [`construct::WEAK_REFERENT`] is bound in, so that a subclass's instance
    /// keeps its referent where `WeakReference~value` looks for it.
    weak_reference: ObjRef,
    /// The class `.Context~stackFrames` builds its elements from. Reached
    /// here rather than by name at every build, exactly as `context` beside
    /// it is.
    stack_frame: ObjRef,
    /// [`hash::STORE_HOLDERS`], resolved.
    store_holders: Vec<ObjRef>,
    /// `.nil`'s own behaviour: `Object`'s with `COPY`, `START` and
    /// `OBJECTNAME=` hidden.
    nil_behaviour: interrexx_classes::BehaviourHandle,
}

impl ObjectModel {
    /// The hash-collection class a `DO OVER` target may be, for a caller
    /// asking whether an object is one.
    pub(crate) fn iterable_collection_class(&self) -> ObjRef {
        self.string_table
    }

    /// The native class set, plus the lookup from each implemented
    /// method's minted identity to its code.
    fn bootstrap(mint: &mut dyn FnMut() -> ObjRef) -> ObjectModel {
        ObjectModel::build(interrexx_classes::native_classes(mint), &[])
    }

    /// [`ObjectModel::bootstrap`] with the two setup-only methods present --
    /// the state the interpreter's own library runs in, and the one
    /// `Interp::bootstrap_library` closes with
    /// `interrexx_classes::remove_setup_methods`.
    pub(crate) fn bootstrap_for_library(mint: &mut dyn FnMut() -> ObjRef) -> ObjectModel {
        ObjectModel::build(
            interrexx_classes::native_classes_for_bootstrap(mint),
            SETUP_METHODS,
        )
    }

    fn build(
        mut classes: interrexx_classes::ClassRegistry,
        extra: &[(&str, &str, Arity, NativeMethod)],
    ) -> ObjectModel {
        let mut natives = MethodRows::new();
        for (class_id, method_name, arity, body) in NATIVE_METHODS
            .iter()
            .chain(buffer::NATIVE_METHODS)
            .chain(construct::NATIVE_METHODS)
            .chain(string::NATIVE_METHODS)
            .chain(hash::NATIVE_METHODS)
            .chain(hash::relation::NATIVE_METHODS)
            .chain(hash::stem::NATIVE_METHODS)
            .chain(collection::list::NATIVE_METHODS)
            .chain(collection::queue::NATIVE_METHODS)
            .chain(array::sort::NATIVE_METHODS)
            .chain(array::surface::NATIVE_METHODS)
            .chain(collection::supplier::NATIVE_METHODS)
            .chain(rexx_info::NATIVE_METHODS)
            .chain(executable::NATIVE_METHODS)
            .chain(introspection::NATIVE_METHODS)
            .chain(context::NATIVE_METHODS)
            .chain(package::NATIVE_METHODS)
            .chain(extra)
            .map(|(class_id, method_name, arity, run)| {
                (class_id, method_name, arity, NativeBody::Run(*run))
            })
            .chain(array::sort::LEVELED_METHODS.iter().map(
                |(class_id, method_name, arity, run)| {
                    (class_id, method_name, arity, NativeBody::Leveled(*run))
                },
            ))
            .chain(
                RESUMABLE_METHODS
                    .iter()
                    .chain(executable::RESUMABLE_METHODS)
                    .map(|(class_id, method_name, arity, begin)| {
                        (class_id, method_name, arity, NativeBody::Begin(*begin))
                    }),
            )
        {
            // **The kernel directory as well as the environment one**, since
            // `RexxInfo` is registered only in the former: `ClassRegistry`'s
            // two tables are disjoint, so the fallback cannot reach a
            // different class than the row names, and a name in neither still
            // panics.
            let class = classes
                .lookup(class_id)
                .or_else(|| classes.system_lookup(class_id))
                .unwrap_or_else(|| {
                    panic!("NATIVE_METHODS names class {class_id:?}, which is not in the registry")
                });
            let (_, method) = classes
                .lookup_instance_method(class, method_name)
                .unwrap_or_else(|| {
                    panic!(
                        "NATIVE_METHODS names {class_id}~{method_name}, which that class's \
                         behaviour does not answer"
                    )
                });
            natives.insert(
                method,
                NativeEntry {
                    arity: *arity,
                    body,
                },
            );
        }
        for (rows, leveled) in [(COMPARED_METHODS, false), (RENDERING_METHODS, true)] {
            for (class_id, method_name) in rows {
                let class = classes
                    .lookup(class_id)
                    .or_else(|| classes.system_lookup(class_id))
                    .unwrap_or_else(|| panic!("{class_id:?} is not in the registry"));
                let (_, method) = classes
                    .lookup_instance_method(class, method_name)
                    .unwrap_or_else(|| panic!("{class_id}~{method_name} is not answered"));
                let entry = natives
                    .get_mut(method)
                    .unwrap_or_else(|| panic!("{class_id}~{method_name} is not a primitive"));
                if let NativeBody::Run(run) = entry.body {
                    entry.body = match leveled {
                        true => NativeBody::Leveled(run),
                        false => NativeBody::Compared(run),
                    };
                }
            }
        }
        for (class_id, method_name, arity, body) in NATIVE_CLASS_METHODS
            .iter()
            .chain(context::NATIVE_CLASS_METHODS)
            .chain(package::NATIVE_CLASS_METHODS)
            .map(|(class_id, method_name, arity, run)| {
                (class_id, method_name, arity, NativeBody::Run(*run))
            })
            .chain(
                RESUMABLE_CLASS_METHODS
                    .iter()
                    .chain(package::RESUMABLE_CLASS_METHODS)
                    .map(|(class_id, method_name, arity, begin)| {
                        (class_id, method_name, arity, NativeBody::Begin(*begin))
                    }),
            )
        {
            // The kernel directory too, as in the loop above, for `RexxInfo`.
            let class = classes
                .lookup(class_id)
                .or_else(|| classes.system_lookup(class_id))
                .unwrap_or_else(|| {
                    panic!(
                        "NATIVE_CLASS_METHODS names class {class_id:?}, which is not in the \
                         registry"
                    )
                });
            let (_, method) = classes
                .lookup_class_method(class, method_name)
                .unwrap_or_else(|| {
                    panic!(
                        "NATIVE_CLASS_METHODS names {class_id}~{method_name}, which that class's \
                         class behaviour does not answer"
                    )
                });
            natives.insert(
                method,
                NativeEntry {
                    arity: *arity,
                    body,
                },
            );
        }
        let string = classes.lookup("String").expect("String is a native class");
        let object = classes.lookup("Object").expect("Object is a native class");
        let metaclass = classes.lookup("Class").expect("Class is a native class");
        let array = classes.lookup("Array").expect("Array is a native class");
        let package = classes
            .lookup("Package")
            .expect("Package is a native class");
        let method = classes.lookup("Method").expect("Method is a native class");
        let routine = classes
            .lookup("Routine")
            .expect("Routine is a native class");
        let directory = classes
            .lookup("Directory")
            .expect("Directory is a native class");
        let string_table = classes
            .lookup("StringTable")
            .expect("StringTable is a native class");
        let context = classes
            .lookup("RexxContext")
            .expect("RexxContext is a native class");
        let rexx_info = classes
            .system_lookup("RexxInfo")
            .expect("RexxInfo is a native class in the kernel directory");
        let message = classes
            .lookup("Message")
            .expect("Message is a native class");
        let variable_reference = classes
            .lookup("VariableReference")
            .expect("VariableReference is a native class");
        let stem = classes.lookup("Stem").expect("Stem is a native class");
        let weak_reference = classes
            .lookup("WeakReference")
            .expect("WeakReference is a native class");
        let stack_frame = classes
            .lookup("StackFrame")
            .expect("StackFrame is a native class");
        let table = classes.lookup("Table").expect("Table is a native class");
        let store_holders = hash::STORE_HOLDERS
            .iter()
            .map(|name| {
                classes
                    .lookup(name)
                    .unwrap_or_else(|| panic!("{name} is a native class"))
            })
            .collect();
        let nil_behaviour = classes.hiding_behaviour(object, &["COPY", "START", "OBJECTNAME="]);
        natives.seal();
        ObjectModel {
            classes,
            natives,
            string,
            object,
            metaclass,
            array,
            package,
            method,
            routine,
            directory,
            string_table,
            table,
            context,
            rexx_info,
            message,
            variable_reference,
            stem,
            weak_reference,
            stack_frame,
            store_holders,
            nil_behaviour,
        }
    }
}

/// What [`Interp::write_object_method`] does to one name in an object's own
/// dictionary, which decides which of [`ObjectMethods`]'s levels it reaches.
#[derive(Copy, Clone, Debug)]
enum ObjectMethodWrite {
    /// `setMethod`, whose `None` is the no-method form: it stores `.nil` and
    /// hides the name rather than removing anything.
    Set(Option<ObjectMethod>),
    /// `unsetMethod`, which reveals whatever is behind the entry it takes.
    Remove,
}

/// Which native class a value answers to -- the classes a value this crate
/// can build belongs to, and no others.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Primitive {
    /// Every string-valued receiver: a literal of any length, and a number.
    /// Measured, `~class~id` for `'abc'`, `12345` and `1.5` is `String` for
    /// all three -- an integer and a number string both
    /// answer `String` for their own class.
    String,
    /// A receiver whose whole value is in the handle's integer tag
    /// ([`Decoded::SmallInt`]) -- **the `SmallInt` behaviour arm**.
    SmallInt,
    /// `.nil`, measured: `.nil~class~id` is `Object`.
    Object,
    /// A `Body::Array`. Measured, `.Array~superClasses~class~id` is `Array`.
    Array,
    /// A `Body::Native` whose class is `.Package` -- what `~package` answers.
    /// Measured, `.Array~package~class~id` is `Package`.
    Package,
    /// A `Body::Native` whose class is `.Method` -- what `Class~method`
    /// answers and what a `.METHODS` entry holds. Measured,
    /// `.K~method("M")~class` is `The Method class`.
    Method,
    /// A `Body::Native` whose class is `.Routine` -- what a `.ROUTINES` entry
    /// holds. Measured, `.routines~r~class` is `The Routine class`.
    Routine,
    /// A `Body::Native` whose class is `.Directory`.
    Directory,
    /// A `Body::Native` whose class is `.StringTable` or a subclass of it.
    StringTable(ObjRef),
    /// A `Body::Native` whose class is `.RexxContext` -- what `.context`
    /// answers. Measured, `.context~class` is `The RexxContext class`.
    Context,
    /// A `Body::Native` whose class is `.StackFrame` -- what
    /// `RexxContext~stackFrames` fills its array with. Measured,
    /// `.context~stackFrames[1]~class~id` is `StackFrame`.
    StackFrame,
    /// A `Body::Native` whose class is the `RexxInfo` one -- the single
    /// pre-built instance `.RexxInfo` answers.
    /// Measured, `.RexxInfo~class~id` is `RexxInfo` and
    /// `.RexxInfo~isA(.Class)` is `0`.
    RexxInfo,
    /// A `Body::Native` whose class is `.Message` -- what `~start` and
    /// `~startWith` answer. For example, `o~start('M', 5)~class~id` is
    /// `Message` and `~string` is `a Message`.
    Message,
    /// A `Body::VarRef` -- the object a `>name` term answers. Measured,
    /// `vr = 5; o = >vr; say o~class~id` is `VariableReference`.
    VariableReference,
    /// A `Body::Stem` -- what a bare stem read answers and what `.Stem~new`
    /// builds. For example, `s. = 'd'; o = s.; say o~class~id` is `Stem`.
    Stem,
    /// The receiver **is** a class object, so its messages resolve against
    /// that class's own class behaviour rather than against any class's
    /// instance behaviour. Measured, `::class K` plus `::method m class`:
    /// `.K~m` runs the body, `.K~hasMethod('M')` is `1` and
    /// `.K~hasMethod('LENGTH')` is `0`.
    Class(ObjRef),
    /// What `~new` builds: an object carrying both the class it belongs to
    /// and the behaviour it was given at construction. Measured,
    /// `::class K` and `o = .K~new`: `o~class~id` is `K` and `o~string` is
    /// `a K`.
    Instance {
        class: ObjRef,
        behaviour: BehaviourHandle,
    },
}

/// The behaviour a receiver's messages resolve against.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Behaviour {
    Instance {
        methods: BehaviourHandle,
        /// The class the dictionary belongs
        /// to, which the copying mutators carry across unchanged.
        owner: ObjRef,
    },
    ClassSide(ObjRef),
}

/// One message term's own parts, as the two call sites already hold them.
#[derive(Copy, Clone)]
pub(crate) struct MessageTerm<'a> {
    pub(crate) target: &'a Expr,
    /// Already upcased by the parser, and already carrying the trailing `=`
    /// for the message-assignment form.
    pub(crate) name: &'a [u8],
    /// The `:scope` override's own expression.
    pub(crate) super_class: Option<&'a Expr>,
    pub(crate) args: &'a [Option<Expr>],
    /// `~~`, which yields the target rather than the method's result.
    pub(crate) cascade: bool,
    /// The message-assignment form's value expression, which is inserted as the
    /// **first** argument ahead of the term's own.
    pub(crate) assigned: Option<&'a Expr>,
}

/// The sending side of a send, which the receiver does not carry.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Caller {
    /// The receiver of the call in progress, from the calling convention.
    receiver: Option<ObjRef>,
    /// The package the sending code was translated in, which is the running
    /// activation's program.
    package: CallerPackage,
}

/// The package a send is written in, as the package check reads it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CallerPackage {
    /// No activation is running. The package check refuses outright for it,
    /// and a resolution asked for
    /// from outside any running program is in it.
    NoActivation,
    /// The running activation's own package, which is the package its clauses
    /// were translated in.
    Package(Package),
}

impl Caller {
    /// The receiver of the call the send is written in.
    pub(crate) fn receiver(self) -> Option<ObjRef> {
        self.receiver
    }

    /// The package the send is written in -- [`Interp::check_package`]'s
    /// input.
    pub(crate) fn package(self) -> CallerPackage {
        self.package
    }
}

/// One method's access scope and protection, as `MethodClass`'s own flags
/// carry them.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct AccessScope {
    /// `PRIVATE` or `PACKAGE`. `Access::Default` or `Access::Public` for a
    /// method whose row exists for `PROTECTED` alone.
    pub(crate) access: Access,
    /// Whether the method is protected, which is what the security-manager seam
    /// asks about.
    pub(crate) protected: bool,
    /// The package the method's own directive was translated in.
    pub(crate) package: Package,
}

/// Why a send's own lookup produced no method to run.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Miss {
    /// The receiver's behaviour answers no method of that name -- 97.1.
    NoMethod,
    /// The method is `PRIVATE` and `checkPrivate` refused this caller --
    /// 97.2.
    Private,
    /// The method is `PACKAGE` and the caller is in another package -- 97.3.
    PackageScope,
}

/// What a resolved message send names: the method, and the class its
/// definition came from.
#[derive(Copy, Clone)]
pub(crate) struct Resolution {
    pub(crate) scope: ObjRef,
    pub(crate) method: MethodId,
}

impl Interp {
    /// The [`Caller`] a send written in the running activation resolves as.
    pub(crate) fn caller(&self) -> Caller {
        Caller {
            receiver: self.activity.call_context.receiver,
            package: match self.running_program() {
                None => CallerPackage::NoActivation,
                Some(program) => CallerPackage::Package(Package::Program(program)),
            },
        }
    }

    /// The object model, built on first use.
    pub(crate) fn object_model(&mut self) -> &mut ObjectModel {
        if self.object_model.is_none() {
            self.build_object_model();
        }
        self.object_model
            .as_mut()
            .expect("the object model was just installed")
    }

    #[cold]
    #[inline(never)]
    fn build_object_model(&mut self) {
        let model = ObjectModel::bootstrap(&mut || self.heap.mint_class());
        self.install_object_model(model);
        self.special_methods.seal();
        self.executable_objects.seal();
    }

    /// Stores a freshly built object model and files the access scopes of
    /// the natives the native class set declares private or protected.
    pub(crate) fn install_object_model(&mut self, model: ObjectModel) {
        debug_assert!(
            self.special_methods.is_empty(),
            "the natives' access scopes are the first rows filed, so nothing may have filed one \
             before the model that mints them exists"
        );
        for &method in model.classes.private_native_methods() {
            self.special_methods.insert(
                method,
                AccessScope {
                    access: Access::Private,
                    protected: false,
                    package: Package::Rexx,
                },
            );
        }
        for &method in model.classes.protected_native_methods() {
            self.special_methods.insert(
                method,
                AccessScope {
                    access: Access::Default,
                    protected: true,
                    package: Package::Rexx,
                },
            );
        }
        self.object_model = Some(model);
    }

    /// The class registry: the one class model in this crate (R9), holding
    /// the native classes and whatever `::CLASS` directives have
    /// installed beside them.
    pub(crate) fn classes(&mut self) -> &mut ClassRegistry {
        &mut self.object_model().classes
    }

    /// Charges the behaviours a mutator just retired toward the next
    /// collection, which is what frees them, and collects if that made one
    /// due: a loop of mutators may allocate nothing else.
    pub(crate) fn charge_retired_behaviours(&mut self) {
        let entries = self.classes().take_retired_entries();
        if entries > 0 {
            self.charge_unheld(entries * RETIRED_ENTRY_BYTES);
            self.collect_if_due();
        }
    }

    /// Whether a live class keeps a behaviour `define` retired.
    pub(crate) fn has_retired_behaviours(&self) -> bool {
        self.object_model
            .as_ref()
            .is_some_and(|model| model.classes.has_retired_behaviours())
    }

    /// Frees the behaviours `define` retired that no live instance holds, and
    /// lets go of the `Method` objects only those behaviours named, unless
    /// an activation is running one.
    #[cold]
    #[inline(never)]
    pub(crate) fn free_unheld_behaviours(&mut self) {
        let Some(model) = self.object_model.as_mut() else {
            return;
        };
        let held = self
            .heap
            .live_objects()
            .filter_map(|object| match object.body {
                Body::Instance { behaviour, .. } => Some(behaviour),
                _ => None,
            });
        let unnamed = model.classes.free_unheld_behaviours(held);
        if unnamed.is_empty() {
            return;
        }
        let mut running = Vec::new();
        self.activity.running_bodies(&mut running);
        self.activities.running_bodies(&mut running);
        for (class, method) in unnamed {
            if self
                .method_bodies
                .get(&method)
                .is_some_and(|body| running.contains(&(body.program, Some(body.directive))))
            {
                continue;
            }
            if let Some(object) = self.executable_objects.get(method).copied() {
                self.class_disowns(class, object);
            }
        }
    }

    /// The rendering of a handle the arena does not hold.
    pub(crate) fn not_in_arena(&self, value: ObjRef) -> &[u8] {
        assert!(self.heap.is_class(value), "a live value");
        self.class_object_name(value)
    }

    /// `~objectName` for a class object, which is [`Interp::class_default_name`]
    /// until `~objectName=` stores one.
    pub(crate) fn class_object_name(&self, class: ObjRef) -> &[u8] {
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .object_name(class)
            .as_bytes()
    }

    /// `~defaultName` for a class object -- `The <id> class` -- borrowed out
    /// of the registry that minted the identity.
    pub(crate) fn class_default_name(&self, class: ObjRef) -> &[u8] {
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .default_name(class)
            .as_bytes()
    }

    /// The id, for any receiver, of the
    /// class whose behaviour answers this object's messages.
    pub(crate) fn is_base_class(&mut self, value: ObjRef) -> bool {
        let Some(class) = self.class_of_value(value) else {
            return true;
        };
        let id = self.classes().id_string(class).to_string();
        self.classes().system_lookup(&id) == Some(class)
    }

    pub(crate) fn receiver_class_id(&mut self, receiver: ObjRef) -> Option<String> {
        let owner = match self.receiver_behaviour(receiver).ok()? {
            Behaviour::Instance { owner, .. } => owner,
            // A class object's messages resolve against its class behaviour,
            // whose owning class is the metaclass in play -- which is what
            // `ClassRegistry::class_of` answers, and what makes
            // `.Array~request("CLASS")` the receiver itself.
            Behaviour::ClassSide(class) => self.classes().class_of(class),
        };
        Some(self.classes().id_string(owner).to_string())
    }

    /// `~id` for a class object -- the name it was declared with, case
    /// unmodified -- and `.NIL` for the `.nil` scope a method `setMethod` or
    /// `run` installed carries, which is how the `>I>` line and traceback name
    /// it: `>I> Method "HM" with scope ".NIL"`.
    pub(crate) fn class_id_text(&self, class: ObjRef) -> &str {
        if class == ObjRef::NIL {
            return ".NIL";
        }
        self.object_model
            .as_ref()
            .expect("a class handle can only have come from the object model")
            .classes
            .id_string(class)
    }

    /// `.Object` and `.Class`: the superclass and metaclass
    /// a class creation defaults to, which is what a bare `::CLASS`
    /// declares.
    pub(crate) fn root_and_metaclass(&mut self) -> (ObjRef, ObjRef) {
        let model = self.object_model();
        (model.object, model.metaclass)
    }

    /// `.String`: the class every string-valued receiver answers to, and the
    /// scope a forwarded operator's traceback frame names -- see
    /// `eval.rs`'s `Interp::is_forwarding_receiver`.
    pub(crate) fn string_class(&mut self) -> ObjRef {
        self.object_model().string
    }

    /// `.Package`: the class of the object `~package` answers, which
    /// `environment/identities.rs` builds one of per package.
    pub(crate) fn package_class(&mut self) -> ObjRef {
        self.object_model().package
    }

    /// `.Method`: the class of the object `Class~method` answers, which
    /// `environment/identities.rs` builds one of per instance dictionary entry a program
    /// asks for.
    pub(crate) fn method_class(&mut self) -> ObjRef {
        self.object_model().method
    }

    /// `.Routine`: the class of the objects `.ROUTINES` holds and
    /// `Routine~new` answers.
    pub(crate) fn routine_class(&mut self) -> ObjRef {
        self.object_model().routine
    }

    /// Which native class a value answers to, or the value's own shape when
    /// this crate builds no class for it.
    fn receiver_kind(&self, receiver: ObjRef) -> Result<Primitive, &'static str> {
        match receiver.decode() {
            Decoded::Nil => Ok(Primitive::Object),
            Decoded::SmallInt(_) => Ok(Primitive::SmallInt),
            Decoded::Text(_) => Ok(Primitive::String),
            Decoded::Heap { .. } => match self.heap.get(receiver) {
                // A handle whose slot is gone. A receiver is rooted by the
                // term that evaluated it, so an expression cannot reach this
                // with a value it just produced; what can is a handle held
                // somewhere the collector's root set does not name, which is
                // a defect in the rooting rather than in the program.
                // Answered rather than panicked for the reason every error
                // path here is: an implementation gap must not become a
                // crash.
                None => Err("a value whose object is no longer live"),
                Some(object) => match &object.body {
                    // The class test folds in here rather than guarding the
                    // arm above: this match already had to fetch the object.
                    Body::Class { .. } => Ok(Primitive::Class(receiver)),
                    Body::Text { .. } | Body::Num { .. } => Ok(Primitive::String),
                    Body::Stem { .. } => Ok(Primitive::Stem),
                    Body::Array { .. } => Ok(Primitive::Array),
                    Body::Instance {
                        class, behaviour, ..
                    } => Ok(Primitive::Instance {
                        class: *class,
                        behaviour: *behaviour,
                    }),
                    Body::WeakRef(_) => Err("a weak reference"),
                    Body::Cell(..) => Err("a variable's cell"),
                    // `>name`'s own object. Its behaviour is the native class's
                    // whole set, hidden operators included, so a name it
                    // does not hold reaches `UNKNOWN` on both sides.
                    Body::VarRef(_) => Ok(Primitive::VariableReference),
                    // The package object `~package` answers. `.Package`'s instance
                    // behaviour here is the native class's whole set, so a name it
                    // does not hold is not a method of `.Package`, and 97.1 is the
                    // right answer rather than a guess.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.package)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Package)
                    }
                    // The `Method` and `Routine` objects `~method` and a package
                    // table put in a program's hands. Each class's instance
                    // behaviour here is the native class's whole set, so a name
                    // it does not hold is not a method of the class -- the
                    // position `.Package` beside them is in.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.method)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Method)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.routine)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Routine)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.directory)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Directory)
                    }
                    // Descent rather than identity: `::class "TraceObject"
                    // subclass StringTable`'s class-side `NEW` forwards to
                    // this one, and the object it builds carries
                    // `TraceObject` as its class.
                    Body::Native(native)
                        if self.object_model.as_ref().is_some_and(|model| {
                            model.classes.is_a(native.class(), model.string_table)
                        }) =>
                    {
                        Ok(Primitive::StringTable(native.class()))
                    }
                    // `.context`. Its class is in the registry, so there is
                    // a behaviour to resolve against, and `RexxContext`'s
                    // instance behaviour here is the native class's whole set --
                    // the position `.Package` above is in, so a name it does
                    // not hold is 97.1 and a name it holds with no row here
                    // is this crate's own gap.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.context)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Context)
                    }
                    // `.RexxInfo`. Its class is in the registry's kernel
                    // directory rather than the environment one, which
                    // changes nothing about the behaviour a send resolves
                    // against -- the position `.Package` above is in.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.rexx_info)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::RexxInfo)
                    }
                    // One element of `RexxContext~stackFrames`, in the same
                    // position `.context` above is: the class is in the
                    // registry with the native class's whole instance set, so a
                    // name it does not hold is 97.1 and a name it holds with
                    // no row here is this crate's own gap.
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.stack_frame)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::StackFrame)
                    }
                    Body::Native(native)
                        if self.object_model.as_ref().map(|model| model.message)
                            == Some(native.class()) =>
                    {
                        Ok(Primitive::Message)
                    }
                    // A `Body::Native` of a class this crate builds no
                    // receiver arm for. Nothing constructs one today; loud
                    // rather than answered, this crate's rule for an internal
                    // inconsistency.
                    Body::Native(_) => Err("one of the interpreter's own objects"),
                },
            },
        }
    }

    /// The behaviour a value's messages resolve against, or the value's own
    /// shape when this crate builds no class for it.
    fn receiver_behaviour(&mut self, receiver: ObjRef) -> Result<Behaviour, &'static str> {
        let kind = self.receiver_kind(receiver)?;
        let model = self.object_model();
        let live = match kind {
            // Both arms answer one behaviour, which is what makes
            // `Primitive::SmallInt` a distinction without a divergence: a small
            // integer has the id `String`, so a small integer's messages
            // resolve against `String`'s instance behaviour exactly as a
            // literal's do. This fold is one of the two sites that would change
            // if that ever stopped being true.
            Primitive::String | Primitive::SmallInt => model.string,
            Primitive::Object => {
                return Ok(Behaviour::Instance {
                    methods: model.nil_behaviour,
                    owner: model.object,
                });
            }
            Primitive::Array => model.array,
            Primitive::Package => model.package,
            Primitive::Method => model.method,
            Primitive::Routine => model.routine,
            Primitive::Directory => model.directory,
            Primitive::StringTable(class) => class,
            Primitive::Context => model.context,
            Primitive::StackFrame => model.stack_frame,
            Primitive::RexxInfo => model.rexx_info,
            Primitive::Message => model.message,
            Primitive::VariableReference => model.variable_reference,
            Primitive::Stem => model.stem,
            Primitive::Class(class) => return Ok(Behaviour::ClassSide(class)),
            Primitive::Instance { class, behaviour } => {
                return Ok(Behaviour::Instance {
                    methods: behaviour,
                    owner: class,
                });
            }
        };
        Ok(Behaviour::Instance {
            methods: self.classes().instance_behaviour_handle(live),
            owner: live,
        })
    }

    /// [`Interp::resolve_in`] with the receiver's behaviour looked up.
    #[cfg(test)]
    pub(crate) fn resolve(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
        let Some(resolution) = self.lookup(receiver, name, start_scope) else {
            return Err(Miss::NoMethod);
        };
        self.admit(resolution, receiver, caller)
    }

    /// **Step one of a send**: which method a name reaches on this
    /// receiver, whose behaviour is `behaviour`, and from which scope, or why
    /// the send has none to run.
    fn resolve_in(
        &mut self,
        receiver: ObjRef,
        behaviour: Behaviour,
        name: &[u8],
        start_scope: Option<ObjRef>,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
        let found = match self.own_lookup(receiver, name, start_scope) {
            Some(found) => found,
            None => self.lookup_behaviour(behaviour, name, start_scope),
        };
        let Some(resolution) = found else {
            return Err(Miss::NoMethod);
        };
        self.admit(resolution, receiver, caller)
    }

    /// `resolution` once its method's access admits a send from `caller`.
    fn admit(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        caller: Caller,
    ) -> Result<Resolution, Miss> {
        match self.access_scope_of(resolution.method) {
            None => Ok(resolution),
            Some(scope) => match scope.access {
                // A method whose row exists for `PROTECTED` alone. The seam
                // asks about that one, not this.
                Access::Default | Access::Public => Ok(resolution),
                // `PRIVATE` and `PACKAGE` are two exclusive arms,
                // and the parser makes a second
                // access keyword 25.902, so no method is in both.
                Access::Private => self
                    .check_private(resolution, receiver, caller)
                    .map(|()| resolution),
                Access::Package => Self::check_package(scope.package, caller).map(|()| resolution),
            },
        }
    }

    /// The ordinary method lookup, or the scope-restricted one for a
    /// `target~name:scope` override, which searches only
    /// the starting scope itself and the scopes folded in ahead of it.
    pub(crate) fn lookup(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Resolution> {
        // **The object's own methods come first**, since an object's own
        // methods sit ahead of its class's.
        // A scope
        // override starts inside the class hierarchy and so never reaches
        // one: a lookup from a named scope searches the hierarchy only, and a
        // one-off's scope is `.nil` or the class's own.
        if let Some(found) = self.own_lookup(receiver, name, start_scope) {
            return found;
        }
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        self.lookup_behaviour(behaviour, name, start_scope)
    }

    /// What `receiver`'s own methods answer for `name` ahead of its
    /// behaviour: `None` where they do not decide it.
    fn own_lookup(
        &self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Option<Resolution>> {
        if start_scope.is_none()
            && let Some(entry) = self.own_method_entry(receiver, name)
        {
            return Some(
                entry.map(|ObjectMethod { method, scope, .. }| Resolution { scope, method }),
            );
        }
        None
    }

    /// [`Interp::lookup`] in `behaviour`, past the receiver's own methods.
    fn lookup_behaviour(
        &mut self,
        behaviour: Behaviour,
        name: &[u8],
        start_scope: Option<ObjRef>,
    ) -> Option<Resolution> {
        // Borrowed rather than owned wherever the name is UTF-8, which every
        // name a program can write is: `from_utf8_lossy` allocates only for
        // the bytes it has to replace.
        // `from_utf8` rather than `from_utf8_lossy`: both scan, but this one
        // has the word-at-a-time ASCII fast path and returns a borrow with no
        // `Cow`. A name that is not UTF-8 answers `None` here, which is the
        // same miss the lossy form reached by building a name with
        // replacement characters that no dictionary holds.
        let name = std::str::from_utf8(name).ok()?;
        let (scope, method) = match (behaviour, start_scope) {
            (Behaviour::Instance { methods, .. }, None) => {
                self.classes().lookup_at(methods, name)?
            }
            (Behaviour::Instance { methods, .. }, Some(start)) => {
                self.classes().lookup_from_scope_at(methods, name, start)?
            }
            (Behaviour::ClassSide(class), None) => {
                self.classes().lookup_class_method(class, name)?
            }
            (Behaviour::ClassSide(class), Some(start)) => self
                .classes()
                .lookup_class_method_from_scope(class, name, start)?,
        };
        Some(Resolution { scope, method })
    }

    /// What `receiver`'s own dictionary answers for `name`: `None` when it
    /// holds no entry under it, `Some(None)` for a name it hides, and
    /// `Some(Some(method))` for one it defines.
    pub(crate) fn own_method_entry(
        &self,
        receiver: ObjRef,
        name: &[u8],
    ) -> Option<Option<ObjectMethod>> {
        if !self.object_methods {
            return None;
        }
        match &self.heap.get(receiver)?.body {
            Body::Instance { own: Some(own), .. } => own.get(name),
            Body::Class { .. } => self.class_own_entry(receiver, name),
            _ => None,
        }
    }

    /// [`Interp::own_method_entry`] for a class object, whose own methods
    /// sit on the object holding its variable pools ([`Interp::pool_owner`]).
    #[cold]
    #[inline(never)]
    fn class_own_entry(&self, class: ObjRef, name: &[u8]) -> Option<Option<ObjectMethod>> {
        let owner = self.class_variables.get(&class)?;
        match &self.heap.get(*owner)?.body {
            Body::Instance { own: Some(own), .. } => own.get(name),
            _ => None,
        }
    }

    /// Installs one entry in `receiver`'s own dictionary, or takes one away
    /// -- `setMethod` and `unsetMethod`.
    fn write_object_method(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        write: ObjectMethodWrite,
    ) -> Result<(), Failure> {
        let holder = match self.receiver_kind(receiver) {
            Ok(Primitive::Instance { .. }) => receiver,
            // For example, a class method's `self~setMethod('m2', 'return 42')`
            // then `.K~m2` answers `42`, and `unsetMethod` from one answers.
            Ok(Primitive::Class(_)) => self.pool_owner(receiver)?,
            Ok(_) => return Err(Loud::object_method("a receiver with no scope of its own").into()),
            Err(kind) => return Err(Loud::unknown_receiver(kind).into()),
        };
        let Some(Object {
            body: Body::Instance { own, .. },
            ..
        }) = self.heap.get_mut(holder)
        else {
            unreachable!("an instance receiver and a class's pool owner are Body::Instance")
        };
        match write {
            ObjectMethodWrite::Remove => {
                if let Some(own) = own {
                    own.remove(name);
                }
            }
            ObjectMethodWrite::Set(entry) => {
                own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
                    .set(name, entry);
                self.object_methods = true;
            }
        }
        self.check_uninit(receiver, name);
        Ok(())
    }

    /// The finalization check, which both
    /// mutators run: an object that answers `UNINIT` after the change is
    /// registered for finalization and one that does not is taken back out.
    fn check_uninit(&mut self, receiver: ObjRef, name: &[u8]) {
        if !name.eq_ignore_ascii_case(UNINIT) {
            return;
        }
        if self.answers_uninit(receiver) {
            self.heap.set_uninit(receiver);
        } else {
            self.heap.clear_uninit_all(&[receiver]);
        }
    }

    /// Whether `scope` was folded into the behaviour this receiver resolves
    /// against.
    pub(crate) fn validate_scope_override(
        &mut self,
        target: ObjRef,
        scope: Option<ObjRef>,
    ) -> Result<(), Failure> {
        let Some(scope) = scope else {
            return Ok(());
        };
        if self.receiver_has_scope(target, scope) {
            return Ok(());
        }
        let named_target = self.string_value_text(target)?;
        let named_scope = self.string_value_text(scope)?;
        Err(Raised::scope_override_not_a_scope(&named_target, &named_scope).into())
    }

    fn receiver_has_scope(&mut self, receiver: ObjRef, scope: ObjRef) -> bool {
        match self.receiver_behaviour(receiver) {
            Ok(Behaviour::Instance { methods, .. }) => {
                self.classes().behaviour_has_scope(methods, scope)
            }
            Ok(Behaviour::ClassSide(class)) => {
                self.classes().class_behaviour_has_scope(class, scope)
            }
            Err(_) => false,
        }
    }

    /// The access scope and protection of a resolved method, for the methods
    /// that have one.
    fn access_scope_of(&self, method: MethodId) -> Option<AccessScope> {
        self.special_methods.get(method).copied()
    }

    /// Whether the method is protected, asked by
    /// the security-manager seam and by nothing else.
    fn method_is_protected(&self, method: MethodId) -> bool {
        self.access_scope_of(method)
            .is_some_and(|scope| scope.protected)
    }

    /// The security manager's protected-method check,
    /// asked for a method the
    /// send has found to be `PROTECTED`.
    ///
    /// `Some(result)` where the manager handled the send, the method then
    /// never running; `None` where it did not, or where no manager is
    /// installed.
    fn check_protected_method(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        if self.effective_security_manager().is_none() {
            return Ok(None);
        }
        let message = self.text(name);
        self.roots.activity_mut().push_temp(message);
        let arguments = self.security_arguments_array(args);
        let entries = [
            (crate::security::key::OBJECT, receiver),
            (crate::security::key::NAME, message),
            (crate::security::key::ARGUMENTS, arguments),
        ];
        let Some(info) = self.security_check(crate::security::message::METHOD, &entries)? else {
            return Ok(None);
        };
        let result = self.security_entry(info, crate::security::key::RESULT)?;
        Ok(Some(result))
    }

    fn check_private(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        caller: Caller,
    ) -> Result<(), Miss> {
        // No calling activation, or one whose frame carries no receiver: a
        // routine or program context, which is refused the same way as
        // no activation at all.
        let Some(sender) = caller.receiver() else {
            return Err(Miss::Private);
        };
        // The sending and receiving object being the same is allowed
        // outright.
        if sender == receiver {
            return Ok(());
        }
        // Another instance of the class that defined the method.
        if let Some(class) = self.class_of_value(sender)
            && self.classes().is_a(class, resolution.scope)
        {
            return Ok(());
        }
        // A class object anywhere in the defining scope's own hierarchy
        // Checked as a class first, because the compatibility test is a
        // method on a class and not on a value.
        if self.is_class_object(sender) && self.classes().is_a(sender, resolution.scope) {
            return Ok(());
        }
        Err(Miss::Private)
    }

    fn check_package(method_package: Package, caller: Caller) -> Result<(), Miss> {
        match caller.package() {
            // No calling activation at all.
            CallerPackage::NoActivation => Err(Miss::PackageScope),
            CallerPackage::Package(package) if package == method_package => Ok(()),
            CallerPackage::Package(_) => Err(Miss::PackageScope),
        }
    }

    /// The class an arbitrary value is an
    /// instance of, or `None` for a value this crate builds no class for.
    pub(crate) fn class_of_value(&mut self, value: ObjRef) -> Option<ObjRef> {
        match self.receiver_behaviour(value).ok()? {
            Behaviour::Instance { owner, .. } => Some(owner),
            Behaviour::ClassSide(class) => Some(self.classes().class_of(class)),
        }
    }

    /// Mints a method identity that runs what `template` runs, with its
    /// access scope and guard: the identity a copy made by `Method~newScope`
    /// is installed under, so the copy's own flags stay its own.
    pub(crate) fn copy_method_identity(&mut self, template: MethodId) -> MethodId {
        let method = self.classes().mint_method_id();
        let natives = &mut self.object_model().natives;
        if let Some(&entry) = natives.get(template) {
            natives.insert(method, entry);
        }
        if let Some(body) = self.method_bodies.get(&template).copied() {
            self.method_bodies.insert(method, body);
        }
        if let Some(generated) = self.generated_methods.get(&template).copied() {
            self.generated_methods.insert(method, generated);
        }
        if let Some(entry) = self.native_externals.get(&template).copied() {
            self.native_externals.insert(method, entry);
        }
        if let Some(binding) = self.library_externals.get(&template).cloned() {
            self.library_externals.insert(method, binding);
        }
        if let Some(program) = self.external_packages.get(&template).copied() {
            self.external_packages.insert(method, program);
        }
        if let Some(code) = self.defined_library_codes.get(&template).copied() {
            self.defined_library_codes.insert(method, code);
        }
        if self.guarded_externals.contains(&template) {
            self.guarded_externals.insert(method);
        }
        if let Some(&row) = self.special_methods.get(template) {
            self.special_methods.insert(method, row);
        }
        method
    }

    /// Whether the primitive table has a row for `method`.
    #[cfg(test)]
    pub(crate) fn has_native_entry(&mut self, method: MethodId) -> bool {
        self.object_model().natives.get(method).is_some()
    }

    /// The extent of the primitive table's indexed and minted halves.
    #[cfg(test)]
    pub(crate) fn native_rows_extent(&mut self) -> (usize, usize) {
        self.object_model().natives.extent()
    }

    /// Drops every row filed under `method`, an identity a swept `Method`
    /// object owned.
    pub(crate) fn forget_method_identity(&mut self, method: MethodId) {
        self.method_bodies.remove(&method);
        self.generated_methods.remove(&method);
        self.native_externals.remove(&method);
        self.library_externals.remove(&method);
        self.external_packages.remove(&method);
        self.defined_library_codes.remove(&method);
        self.guarded_externals.remove(&method);
        self.executable_objects.remove(method);
        self.special_methods.remove(method);
        if let Some(model) = self.object_model.as_mut() {
            model.natives.remove(method);
        }
    }

    /// A method scope's `~id`, [`Interp::class_id_text`]'s rendering: a
    /// primitive `setMethod` or `run` installed under the `.nil` scope is
    /// `Compiled method "HM" with scope ".NIL"`.
    pub(crate) fn scope_id(&mut self, scope: ObjRef) -> String {
        self.object_model();
        self.class_id_text(scope).to_string()
    }

    /// `isOfClassType(Class, value)`: whether the value is a class object.
    pub(crate) fn is_class_object(&self, value: ObjRef) -> bool {
        matches!(self.receiver_kind(value), Ok(Primitive::Class(_)))
    }

    /// The scope a method found at `resolution` sees as `SUPER`, or `None` when
    /// it is the topmost scope in the receiver's behaviour -- the superscope a
    /// method activation carries.
    fn super_scope_for(&mut self, receiver: ObjRef, resolution: Resolution) -> Option<ObjRef> {
        self.super_scope_of(receiver, resolution.scope)
    }

    /// [`Interp::super_scope_for`] for a method whose scope is `scope`.
    pub(crate) fn super_scope_of(&mut self, receiver: ObjRef, scope: ObjRef) -> Option<ObjRef> {
        let behaviour = self.receiver_behaviour(receiver).ok()?;
        self.super_scope_in(behaviour, scope)
    }

    /// [`Interp::super_scope_of`] in `behaviour`, the receiver's. A method of
    /// scope `.nil` (`setMethod`, `Class~enhanced`) sees the behaviour's
    /// owning class.
    fn super_scope_in(&mut self, behaviour: Behaviour, scope: ObjRef) -> Option<ObjRef> {
        match behaviour {
            Behaviour::Instance { owner, .. } if scope == ObjRef::NIL => Some(owner),
            Behaviour::Instance { methods, .. } => self.classes().super_scope_at(methods, scope),
            Behaviour::ClassSide(class) if scope == ObjRef::NIL => {
                Some(self.classes().class_of(class))
            }
            Behaviour::ClassSide(class) => self.classes().class_super_scope(class, scope),
        }
    }

    /// What a resolved [`MethodId`] runs, or the refusal for one no table
    /// below names.
    fn invocable(&mut self, resolution: Resolution, name: &[u8]) -> Result<Invocable, Failure> {
        if let Some(&entry) = self.object_model().natives.get(resolution.method) {
            return Ok(Invocable::Native(entry));
        }
        if let Some(installed) = self.method_bodies.get(&resolution.method).copied() {
            return Ok(Invocable::Rexx(installed));
        }
        // **After the body table and never before it**, so a send to a
        // written body reads exactly the one table it read before generated
        // methods existed -- see `crate::GeneratedMethod`'s own doc for what
        // that ordering is worth on `dispatchclass`.
        if let Some(generated) = self.generated_methods.get(&resolution.method).copied() {
            return Ok(Invocable::Generated(generated));
        }
        // **Last, so the tables above are read exactly as they were before
        // an `EXTERNAL` method could be sent to.** A send that reaches here
        // has already missed every table a send to an ordinary body or a
        // primitive reads, so no path this table is not on pays for it.
        if let Some(entry) = self.native_externals.get(&resolution.method).copied() {
            return Ok(Invocable::External(entry));
        }
        if let Some(binding) = self.library_externals.get(&resolution.method) {
            return Ok(Invocable::Library(binding.clone()));
        }
        // The scope's `~id` is rendered only where it is printed -- a
        // successful send has no use for it, and every send would otherwise
        // pay for the copy.
        let scope = self.scope_id(resolution.scope);
        Err(Loud::unbound_method(name, &scope).into())
    }

    /// **Step two of a send**: run what [`Interp::resolve_in`] found.
    pub(crate) fn invoke(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let started = self.begin_invoke(resolution, None, receiver, name, args)?;
        self.complete_send(started)
    }

    /// [`Interp::invoke`] up to the point a Rexx body it enters would run,
    /// with `behaviour` the receiver's where the caller has it.
    fn begin_invoke(
        &mut self,
        resolution: Resolution,
        behaviour: Option<Behaviour>,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let invocable = self.invocable(resolution, name)?;
        let (cleared, behaviour) = match seam::clear(self, receiver, name, args, resolution.method)?
        {
            seam::Clearance::Cleared(cleared) => (cleared, behaviour),
            seam::Clearance::Checked(cleared) => (cleared, None),
            seam::Clearance::Answered(result) => return Ok(Started::Ran(result)),
        };
        if let Invocable::Rexx(installed) = invocable {
            // **Neither this path nor `Generated` blames the method**, and both
            // are observable. An untrapped `1/0` inside a `::METHOD` body
            // reports the method's own failing clause and then the sending
            // clause, with no `Compiled method` line between them, and
            // [`Interp::finish_call`] seals the method's own level for that,
            // exactly as it does for a call. A generated accessor and an
            // `ABSTRACT` send report the *sending* clause and nothing above it:
            // `.K~a(1)` on `::attribute a class` is `93.902` with the sending
            // clause alone on `stderr`, where `'abc'~length(1)` -- a
            // [`NativeMethod`] taking the same refusal -- carries a `Compiled
            // method "LENGTH"` line. The two differ because an attribute getter
            // and an abstract method raise directly where a primitive method
            // raises from inside an activation of its own.
            self.begin_method(
                cleared, installed, resolution, behaviour, receiver, name, args,
            )?;
            return Ok(Started::Entered);
        }
        let outcome = match invocable {
            // A leveled primitive is run by `run_other`, off this path.
            Invocable::Native(entry) if !matches!(entry.body, NativeBody::Leveled(_)) => {
                let outcome = match (entry.arity, entry.body) {
                    (Arity::Fixed(arity), _) if args.len() > arity => {
                        Err(Raised::too_many_method_arguments(arity).into())
                    }
                    (_, NativeBody::Compared(run)) if self.argument_may_run_rexx(args) => self
                        .run_leveled(run, cleared, resolution, receiver, name, args)
                        .map(NativeStarted::Ran),
                    (
                        _,
                        NativeBody::Run(run) | NativeBody::Leveled(run) | NativeBody::Compared(run),
                    ) => {
                        let ran = pinned!(
                            self,
                            crate::pinning::PinKind::native(name),
                            run(self, cleared, receiver, args)
                        );
                        self.serve_posted_halt();
                        ran.map(NativeStarted::Ran)
                    }
                    (_, NativeBody::Begin(begin)) => begin(self, cleared, receiver, args),
                };
                match outcome {
                    Ok(NativeStarted::Ran(value)) => Ok(value),
                    Ok(NativeStarted::Entered(then)) => {
                        let blame = NativeBlame {
                            name: name.into(),
                            scope: resolution.scope,
                            receiver,
                            args: args.into(),
                        };
                        match self.activity.native_park.as_mut() {
                            Some(park) => park.thens.push((then, Some(blame))),
                            None => self.push_native_tail(then, Some(blame)),
                        }
                        return Ok(Started::Entered);
                    }
                    Err(failure) => {
                        let scope = self.scope_id(resolution.scope);
                        self.blame_native_method(name, &scope, receiver, args);
                        Err(failure)
                    }
                }
            }
            other => {
                return self.begin_invoke_other(other, cleared, resolution, receiver, name, args);
            }
        };
        outcome.map(Started::Ran)
    }

    /// [`Interp::begin_invoke`] for a method whose body is neither Rexx nor
    /// native, holding its guard lock where it is guarded.
    #[inline(never)]
    fn begin_invoke_other(
        &mut self,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let guard = self.other_guard(&invocable, resolution, receiver);
        if let Some(key) = guard
            && let crate::guards::Reserve::Contended(owner) = self.take_guard(key)
        {
            let send = GuardedSend {
                key,
                invocable,
                cleared,
                resolution,
                receiver,
                name: name.into(),
                args: args.to_vec(),
            };
            return self.wait_for_other_guard(owner, send);
        }
        self.run_other_guarded(guard, invocable, cleared, resolution, receiver, name, args)
    }

    /// The guard lock a send to `invocable` reserves: a generated getter,
    /// setter or `DELEGATE` and an external method, where guarded.
    fn other_guard(
        &self,
        invocable: &Invocable,
        resolution: Resolution,
        receiver: ObjRef,
    ) -> Option<crate::guards::GuardKey> {
        let guarded = match invocable {
            Invocable::Generated(generated) => {
                matches!(
                    generated.kind,
                    crate::GeneratedKind::Getter
                        | crate::GeneratedKind::Setter
                        | crate::GeneratedKind::Delegate
                ) && self
                    .programs
                    .get(generated.program)
                    .and_then(|program| program.directives.get(generated.directive))
                    .is_some_and(|directive| crate::guards::directive_guarded(&directive.kind))
            }
            Invocable::External(_) | Invocable::Library(_) => {
                self.guarded_externals.contains(&resolution.method)
            }
            Invocable::Rexx(_) | Invocable::Native(_) => false,
        };
        let guarded = match invocable {
            Invocable::Generated(_) | Invocable::External(_) | Invocable::Library(_) => {
                self.sends_guarded(resolution.method, guarded)
            }
            Invocable::Rexx(_) | Invocable::Native(_) => false,
        };
        guarded.then_some(crate::guards::GuardKey {
            object: receiver,
            scope: resolution.scope,
        })
    }

    /// `send`'s wait for its guard lock, owned by `owner`: 98.905 where the
    /// waits it would join lead back here, else parked until a release
    /// grants it, then run.
    #[cold]
    #[inline(never)]
    fn wait_for_other_guard(
        &mut self,
        owner: crate::scheduler::ActivityId,
        send: GuardedSend,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        if self.deadlocks(owner) {
            return Err(Raised::deadlock().into());
        }
        let (key, receiver) = (send.key, send.receiver);
        let me = self.running_activity();
        self.activities.guards.enqueue(key, me);
        self.activity.guarded_send = Some(Box::new(send));
        match self.park_native(
            crate::scheduler::ParkReason::Guard(key),
            resume_guarded_send,
            receiver,
        ) {
            Ok(NativeStarted::Ran(value)) => Ok(Started::Ran(value)),
            Ok(NativeStarted::Entered(_)) => Ok(Started::Entered),
            Err(failure) => {
                self.activity.guarded_send = None;
                if self.granted(key) {
                    self.release_guard(key);
                }
                Err(failure)
            }
        }
    }

    /// [`Interp::run_other`] holding `guard`, which it releases: a
    /// `DELEGATE` method's once it has read its target, before the send
    /// (the delegate path), anything else's once it has run.
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and the lock it holds"
    )]
    fn run_other_guarded(
        &mut self,
        guard: Option<crate::guards::GuardKey>,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let Some(key) = guard else {
            return self.run_other(invocable, cleared, resolution, receiver, name, args, false);
        };
        if let Invocable::Generated(generated) = &invocable
            && generated.kind == crate::GeneratedKind::Delegate
        {
            let target = self.delegate_target(*generated, resolution, receiver);
            self.release_guard(key);
            return self
                .send_to_delegate_target(target?, name, args)
                .map(Started::Ran);
        }
        // A library method's own frame holds the lock, which `SetGuardOff`
        // may give up, and releases it where it still holds it. A scoped
        // native method gives it up by taking `scoped_guard`.
        let library = matches!(invocable, Invocable::Library(_));
        let outer = self.activity.scoped_guard.replace(key);
        let outcome = self.run_other(invocable, cleared, resolution, receiver, name, args, true);
        let held = std::mem::replace(&mut self.activity.scoped_guard, outer);
        if !library && held.is_some() {
            self.release_guard(key);
        }
        outcome
    }

    /// The body of a method that is neither Rexx nor native.
    #[expect(
        clippy::too_many_arguments,
        reason = "the send's own parts, and whether it holds its guard lock"
    )]
    fn run_other(
        &mut self,
        invocable: Invocable,
        cleared: Cleared,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        reserved: bool,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let native = !matches!(invocable, Invocable::Generated(_));
        let outcome = match invocable {
            // The implemented arm blames the method for a refusal, because its
            // refusals are those of a native method: `.k~sep(1)` on a class
            // method bound to `file_separator` is `88.922 Too many arguments in
            // invocation; 0 expected.` at rc 168, under a `Compiled method
            // "SEP" with scope "K".` line.
            Invocable::External(entry) => match &entry.body {
                native::ExternalBody::Implemented { arity, run } => {
                    let ran =
                        self.run_external(*arity, resolution, receiver, name, args, |interp| {
                            pinned!(
                                interp,
                                crate::pinning::PinKind::native(name),
                                run(interp, cleared, receiver, args)
                            )
                        });
                    if !self.activity.native_frames_owed.is_empty() {
                        self.settle_native_frames_owed(resolution, receiver, name, args)?;
                    }
                    ran
                }
                native::ExternalBody::Scoped { arity, begin } => {
                    let started =
                        self.run_external(*arity, resolution, receiver, name, args, |interp| {
                            begin(interp, cleared, receiver, resolution.scope, args)
                        })?;
                    match started {
                        NativeStarted::Ran(value) => Ok(value),
                        NativeStarted::Entered(then) => {
                            match self.activity.native_park.as_mut() {
                                Some(park) => park.thens.push((then, None)),
                                None => self.push_native_tail(then, None),
                            }
                            return Ok(Started::Entered);
                        }
                    }
                }
            },
            Invocable::Native(NativeEntry {
                arity,
                body: NativeBody::Leveled(run),
            }) => {
                let ran = match arity {
                    Arity::Fixed(arity) if args.len() > arity => {
                        Err(Raised::too_many_method_arguments(arity).into())
                    }
                    _ => self.run_leveled(run, cleared, resolution, receiver, name, args),
                };
                if ran.is_err() {
                    let scope = self.scope_id(resolution.scope);
                    self.blame_native_method(name, &scope, receiver, args);
                }
                return ran.map(Started::Ran);
            }
            Invocable::Rexx(_) | Invocable::Native(_) => return Err(Loud::missing_body().into()),
            Invocable::Library(binding) => {
                let outcome = match self
                    .run_library_method(&binding, resolution, receiver, name, args, reserved)
                {
                    Ok(Started::Entered) => return Ok(Started::Entered),
                    Ok(Started::Ran(value)) => Ok(value),
                    Err(failure) => Err(failure),
                };
                if outcome.is_err() {
                    let scope = self.scope_id(resolution.scope);
                    let method = resolution.method;
                    let reraised = std::mem::take(&mut self.activity.native_reraise);
                    self.blame_external_method(name, &scope, method, receiver, args, reraised);
                }
                outcome
            }
            Invocable::Generated(generated) => match generated.kind {
                crate::GeneratedKind::Getter => {
                    self.read_attribute(cleared, generated, resolution, receiver, args)
                }
                crate::GeneratedKind::Setter => {
                    self.write_attribute(cleared, generated, resolution, receiver, args)
                }
                crate::GeneratedKind::Abstract => Err(Raised::abstract_method(name).into()),
                crate::GeneratedKind::Constant => {
                    self.read_constant(cleared, generated, receiver, args)
                }
                crate::GeneratedKind::ConstantExpression => {
                    self.run_constant_expression(generated, resolution, receiver, name, args)
                }
                crate::GeneratedKind::Delegate => {
                    self.send_to_delegate(cleared, generated, resolution, receiver, name, args)
                }
            },
        };
        // A native method's return runs the `UNINIT`s a collection readied.
        if native
            && !self.uninit_ready.is_empty()
            && let Ok(value) = outcome
        {
            self.run_uninits_at_return(value)?;
        }
        outcome.map(Started::Ran)
    }

    /// Leads each condition object a `CALL ON` trap queued inside this
    /// `LIBRARY REXX` method with the method's own frame, as
    /// the program information is built with the method's own
    /// activation on top.
    #[cold]
    #[inline(never)]
    fn settle_native_frames_owed(
        &mut self,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<(), Failure> {
        let frame = self.roots.activity_mut().push_frame();
        let owed = std::mem::take(&mut self.activity.native_frames_owed);
        for &object in &owed {
            self.roots.activity_mut().push_temp(object);
        }
        let scope = self.scope_id(resolution.scope);
        let text = Raised::compiled_method_line(name, &scope);
        let executable = context::native_method_executable(self, receiver, name);
        let level =
            context::build_native_level_frame(self, None, name, receiver, args, &text, executable);
        let settled = owed
            .iter()
            .try_for_each(|&object| self.lead_with_native_frame(object, level, &text));
        self.roots.activity_mut().pop_frame(frame);
        settled
    }

    /// A `LIBRARY REXX` entry's `run` after the arity check external
    /// methods make, with the method blamed for a failure.
    fn run_external<T>(
        &mut self,
        arity: Arity,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        run: impl FnOnce(&mut Interp) -> Result<T, Failure>,
    ) -> Result<T, Failure> {
        let outcome = match arity {
            Arity::Fixed(arity) if args.len() > arity => {
                Err(Raised::too_many_external_arguments(arity).into())
            }
            Arity::Fixed(_) | Arity::Counted => run(self),
        };
        if let Err(failure) = &outcome {
            let scope = self.scope_id(resolution.scope);
            let method = resolution.method;
            let reraised = failure.reraised_by_native_call();
            self.blame_external_method(name, &scope, method, receiver, args, reraised);
        }
        outcome
    }

    /// Runs a started send's Rexx body on this Rust stack, and answers the
    /// send's value.
    pub(crate) fn complete_send(
        &mut self,
        started: Started<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        match started {
            Started::Ran(value) => Ok(value),
            Started::Entered => {
                if self.activity.native_park.is_some() {
                    return Err(self.refuse_unrooted_park());
                }
                let ended = self.run_activation();
                self.finish_send(ended)
            }
        }
    }

    /// Answers a park for `reason`, whose continuation is `resume` on
    /// `receiver`. Where a Rust frame pins the activity, the wait runs the
    /// other activities here and the continuation's answer is the native's.
    pub(crate) fn park_native(
        &mut self,
        reason: crate::scheduler::ParkReason,
        resume: NativeResume,
        receiver: ObjRef,
    ) -> Result<NativeStarted, Failure> {
        if self.activity.pin_depth > 0 {
            let frame = self.roots.activity_mut().push_frame();
            self.roots.activity_mut().push_temp(receiver);
            let mut reason = reason;
            let sent = loop {
                if let Some(failure) = self.pinned_wait(reason) {
                    break Err(failure);
                }
                match self.retest_wait(reason) {
                    Some(again) => reason = again,
                    None => break resume(self, receiver),
                }
            };
            self.roots.activity_mut().pop_frame(frame);
            return sent.map(NativeStarted::Ran);
        }
        self.activity.native_park = Some(Box::new(NativePark {
            reason,
            resume,
            receiver,
            thens: Vec::new(),
        }));
        Ok(NativeStarted::Entered(Then::Pass))
    }

    /// A park for `reason` by an instruction its op runs again once the
    /// activity wakes, recorded on the activity for the root driver.
    #[cold]
    #[inline(never)]
    pub(crate) fn park_instruction(&mut self, reason: crate::scheduler::ParkReason) {
        self.activity.native_park = Some(Box::new(NativePark {
            reason,
            resume: |_, _| Ok(None),
            receiver: ObjRef::NIL,
            thens: Vec::new(),
        }));
    }

    /// A park for `reason` by a routine whose answer, once woken, is what
    /// `resume` answers for `receiver`: that answer after a pinned wait, or
    /// `receiver` with the park recorded on the activity.
    pub(crate) fn park_routine_with(
        &mut self,
        reason: crate::scheduler::ParkReason,
        resume: NativeResume,
        receiver: ObjRef,
    ) -> Result<ObjRef, Failure> {
        Ok(match self.park_native(reason, resume, receiver)? {
            NativeStarted::Ran(answer) => answer.unwrap_or(receiver),
            NativeStarted::Entered(_) => receiver,
        })
    }

    /// [`Interp::park_routine_with`] for the body of a routine of a library
    /// this crate runs in Rust (`crate::extensions`). Where nothing but the
    /// routine's own frame pins the activity, the park is recorded for the
    /// root driver and the routine's call answers `Entered` once the body
    /// returns (`run_rust_routine`), so that the other activities run while
    /// this one waits; otherwise the wait is pinned.
    pub(crate) fn park_rust_routine(
        &mut self,
        reason: crate::scheduler::ParkReason,
        resume: NativeResume,
        receiver: ObjRef,
    ) -> Result<ObjRef, Failure> {
        if self.activity.pin_depth == 1 && !self.activity.resuming {
            self.activity.native_park = Some(Box::new(NativePark {
                reason,
                resume,
                receiver,
                thens: Vec::new(),
            }));
            return Ok(receiver);
        }
        self.park_routine_with(reason, resume, receiver)
    }

    /// `park`'s primitive methods, run once their activity has woken: the
    /// continuation, or `failure` in its place, then each method's work after
    /// it.
    pub(crate) fn resume_native_park(
        &mut self,
        park: NativePark,
        failure: Option<Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let NativePark {
            resume,
            receiver,
            thens,
            ..
        } = park;
        let frame = self.roots.activity_mut().push_frame();
        let mut held = Vec::new();
        for (then, blame) in &thens {
            then.object_roots(&mut held);
            if let Some(blame) = blame {
                blame.object_roots(&mut held);
            }
        }
        for object in held {
            self.roots.activity_mut().push_temp(object);
        }
        let mut sent = match failure {
            Some(failure) => {
                self.abandon_native_call(receiver);
                Err(failure)
            }
            None => {
                self.activity.resuming = true;
                let sent = resume(self, receiver);
                self.activity.resuming = false;
                sent
            }
        };
        for (then, blame) in thens {
            if let Ok(Some(value)) = sent {
                self.roots.activity_mut().push_temp(value);
            }
            sent = self.apply_then(then, sent);
            if sent.is_err()
                && let Some(blame) = blame
            {
                let scope = self.scope_id(blame.scope);
                self.blame_native_method(&blame.name, &scope, blame.receiver, &blame.args);
            }
        }
        self.roots.activity_mut().pop_frame(frame);
        sent
    }

    /// A send's value once the Rexx body it entered has ended `ended`: the
    /// sender's level state back, and the work of every primitive method
    /// between the sender and that body.
    pub(crate) fn finish_send(
        &mut self,
        ended: Result<crate::run::Ended, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let depth = self.activity.call_tails.len();
        let mut sent = self.finish_call(ended).map(crate::run::Ended::value);
        if self
            .activity
            .native_tails
            .last()
            .is_some_and(|tail| tail.depth == depth)
        {
            sent = self.finish_native_tails(depth, sent);
        }
        if !self.uninit_ready.is_empty()
            && let Ok(value) = sent
        {
            self.run_uninits_at_return(value)?;
        }
        sent
    }

    /// [`Interp::finish_send`]'s primitive methods, innermost first.
    #[cold]
    #[inline(never)]
    fn finish_native_tails(
        &mut self,
        depth: usize,
        mut sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        let first = self
            .activity
            .native_tails
            .iter()
            .rposition(|tail| tail.depth != depth)
            .map_or(0, |below| below + 1);
        let tails: Vec<NativeTail> = self.activity.native_tails.drain(first..).collect();
        for tail in tails {
            sent = self.apply_then(tail.then, sent);
            if sent.is_err()
                && let Some(blame) = tail.blame
            {
                let scope = self.scope_id(blame.scope);
                self.blame_native_method(&blame.name, &scope, blame.receiver, &blame.args);
            }
        }
        sent
    }

    /// Parks `then` beside the [`crate::run::CallTail`] on top.
    fn push_native_tail(&mut self, then: Then, blame: Option<NativeBlame>) {
        let depth = self.activity.call_tails.len();
        let entered = self.running_activation().map(|activation| activation.id);
        self.activity.native_tails.push(NativeTail {
            depth,
            then,
            blame,
            entered,
        });
    }

    /// A [`NativeBegin`]'s outcome, with the activation it entered run on
    /// this Rust stack: the whole primitive method, for a caller that is not
    /// a send.
    pub(crate) fn complete_native(
        &mut self,
        started: NativeStarted,
    ) -> Result<Option<ObjRef>, Failure> {
        match started {
            NativeStarted::Ran(value) => Ok(value),
            NativeStarted::Entered(then) => {
                if self.activity.native_park.is_some() {
                    return Err(self.refuse_unrooted_park());
                }
                self.push_native_tail(then, None);
                let ended = self.run_activation();
                self.finish_send(ended)
            }
        }
    }

    /// What a primitive method answers once the activation it entered
    /// answered `sent`.
    pub(crate) fn apply_then(
        &mut self,
        then: Then,
        sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Option<ObjRef>, Failure> {
        match then {
            Then::Pass => sent,
            Then::Answer(object) => sent.map(|_| Some(object)),
            Then::Started(message) => object_protocol::record_started(self, message, sent),
            Then::Held(message) => object_protocol::record_held(self, message, sent),
            Then::Triggered(message) => {
                object_protocol::record_held(self, message, sent).map(|_| None)
            }
        }
    }

    /// A generated getter: the value the attribute's variable holds in the
    /// declaring scope's pool on the receiver ([`Interp::pool_value`]).
    fn read_attribute(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if !args.is_empty() {
            return Err(Raised::too_many_method_arguments(0).into());
        }
        let variable = self.accessor_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        Ok(Some(self.pool_value(owner, resolution.scope, &variable)))
    }

    /// A generated setter: assigns the attribute's variable in the declaring
    /// scope's pool on the receiver, and answers nothing.
    fn write_attribute(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if args.len() > 1 {
            return Err(Raised::too_many_method_arguments(1).into());
        }
        let Some(Some(value)) = args.first().copied() else {
            return Err(Raised::missing_method_argument(1).into());
        };
        let variable = self.accessor_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        self.set_pool_value(owner, resolution.scope, &variable, value);
        Ok(None)
    }

    /// A `::CONSTANT` accessor: the value the resolve-constants pass recorded
    /// for the directive, on whichever dictionary side the send resolved
    /// through.
    fn read_constant(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        receiver: ObjRef,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        if !args.is_empty() {
            return Err(Raised::too_many_method_arguments(0).into());
        }
        if let Some(value) = self.constant_value(generated) {
            return Ok(Some(value));
        }
        let declared = self.constant_name(generated)?;
        let target = self.message_target_text(receiver)?;
        let report = Raised::constant_not_initialized(&target, &declared);
        Err(
            if self.trap_for(b"NOMETHOD").is_some_and(|trap| !trap.call) {
                Raised::nomethod(report, &declared).into()
            } else {
                report.into()
            },
        )
    }

    /// A `DELEGATE` method: the message re-sent, under the name it arrived
    /// under and with the arguments it arrived with, to the value of the
    /// delegate variable in the declaring scope's pool on the receiver.
    fn send_to_delegate(
        &mut self,
        _cleared: Cleared,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let target = self.delegate_target(generated, resolution, receiver)?;
        self.send_to_delegate_target(target, name, args)
    }

    /// The value of a `DELEGATE` method's variable, rooted as a temp.
    fn delegate_target(
        &mut self,
        generated: crate::GeneratedMethod,
        resolution: Resolution,
        receiver: ObjRef,
    ) -> Result<ObjRef, Failure> {
        let variable = self.delegate_variable(generated)?;
        let owner = self.pool_owner(receiver)?;
        let target = self.pool_value(owner, resolution.scope, &variable);
        self.roots.activity_mut().push_temp(target);
        Ok(target)
    }

    /// A `DELEGATE` method's send to `target`.
    fn send_to_delegate_target(
        &mut self,
        target: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let caller = self.caller();
        pinned!(
            self,
            crate::pinning::PinKind::Delegate,
            self.send_message(target, name, None, args, caller)
        )
    }

    /// The variable a `DELEGATE` method addresses.
    fn delegate_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        let Some(program) = self.programs.get(generated.program) else {
            return Err(Loud::missing_body().into());
        };
        let Some(directive) = program.directives.get(generated.directive) else {
            return Err(Loud::missing_body().into());
        };
        let Some(symbol) = crate::delegate_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        Ok(program.symbols.name(symbol).as_bytes().into())
    }

    /// The variable a generated accessor addresses.
    fn accessor_variable(&self, generated: crate::GeneratedMethod) -> Result<Box<[u8]>, Failure> {
        // `get` rather than an index, and `None` rather than a panic, for the
        // reason `Interp::begin_method`'s own reads carry.
        let Some(directive) = self
            .programs
            .get(generated.program)
            .and_then(|program| program.directives.get(generated.directive))
        else {
            return Err(Loud::missing_body().into());
        };
        let Some(variable) = crate::accessor_variable(&directive.kind) else {
            return Err(Loud::missing_body().into());
        };
        Ok(variable.into())
    }

    /// [`Interp::finish_call`]'s release of a method activation that has
    /// just been popped, with `saved_context` the sender's convention. A
    /// `REPLY`'s activation answers its value, and moves with its slot frame
    /// to the continuation its split filed, if the split ran.
    pub(crate) fn release_method_activation(
        &mut self,
        mut callee: Box<Activation>,
        saved_context: crate::CallContext,
    ) -> Option<Option<ObjRef>> {
        // Unconditionally, where `Interp::invoke_call` asks `owns_frame`
        // first: a method activation always owns its frame and nothing can
        // change that under it, because the one instruction that swaps a
        // frame in is `PROCEDURE` and `Entry::Method` is 17.1 for it.
        debug_assert!(callee.owns_frame, "a method activation owns its frame");
        let callee_context = std::mem::replace(&mut self.activity.call_context, saved_context);
        let Some(replied) = callee.replied.take() else {
            self.roots.activity_mut().pop_slots(callee.frame);
            if callee.exposed.capacity() > 0 {
                let mut exposed = std::mem::take(&mut callee.exposed);
                exposed.clear();
                self.activity.spare_exposed = exposed;
            }
            // **Back to the pool**, which until now only `Interp::invoke_call`
            // fed. The pool is drained by every push and was filled by the
            // `CALL` path alone, so a program of method sends missed it every
            // time: measured on `bench-programs/dispatch.rex`, 5,000,089
            // misses and no hits, each one a `Box::new` of a 416-byte
            // `Activation` and the free that follows.
            self.recycle_activation(callee);
            return None;
        };
        let Replied {
            value,
            continuation,
        } = *replied;
        // Parked while the rest is filed: the spawner's frame it records is
        // a fresh allocation.
        let parked = self.roots.activity_mut().park(value.into_iter().collect());
        // The body ended before its next clause, so no split ran. The empty
        // rest is still spawned, and it announces itself again.
        let (callee, continuation) = match continuation {
            Some(continuation) => (callee, Some(continuation)),
            None => self.empty_continuation(callee),
        };
        match continuation {
            Some(continuation) => self.spawn_continuation(callee, callee_context, continuation),
            // A rest that could not be filed is dropped with the numbering a
            // spawn would have made.
            None => {
                self.activity_number();
                self.rotate_pooled();
                self.activity.splits_owed -= 1;
                self.roots.activity_mut().pop_slots(callee.frame);
                self.recycle_activation(callee);
            }
        }
        self.roots.activity_mut().release(parked);
        Some(value)
    }

    /// Files `continuation`, the record a `REPLY`'s split left, with
    /// `activation` and its slot frame moved into it. A replier yields only to
    /// a thread already waiting, which the continuation's is not: the sender
    /// yields at its next clause boundary where another activity is ready, and
    /// otherwise keeps running.
    #[cold]
    #[inline(never)]
    fn spawn_continuation(
        &mut self,
        mut activation: Box<Activation>,
        context: crate::CallContext,
        mut continuation: Box<crate::scheduler::Idle>,
    ) {
        let mut anchor = Vec::new();
        activation.object_roots(&mut anchor);
        context.object_roots(&mut anchor);
        continuation.roots.park(anchor);
        activation.frame = self
            .roots
            .activity_mut()
            .move_frame(activation.frame, &mut continuation.roots);
        activation.reply = ReplyState::Issued;
        activation.trace_entry = if activation.trace_entry == TraceEntry::Done {
            TraceEntry::Allowed
        } else {
            TraceEntry::Spent
        };
        activation.first_instruction_pending = false;
        self.guards_go_live(Some(&activation));
        let transfer = self.transfer_on_reply(&mut activation);
        let resumed = &mut continuation.activity;
        if let crate::guards::Transfer::Again(wait) = transfer {
            resumed.guard_waits.push(wait);
        }
        resumed.trace_cache =
            crate::trace::TraceCache::of(activation.trace_mode, activation.flags.debug_pause());
        resumed.running = Some(activation);
        resumed.call_context = context;
        resumed.call_tails.push(crate::run::CallTail::method(
            crate::CallContext::default(),
            self.save_clause_state(),
            0,
            0,
            None,
        ));
        resumed.first = Some(crate::scheduler::First::Reply);
        resumed.root_then = Some(Then::Pass);
        let others_ready = self.any_ready();
        let spawned = self.spawn(continuation);
        if let crate::guards::Transfer::Moved(key) = transfer {
            self.activities.guards.transfer(key, spawned);
        }
        if others_ready {
            self.timer.requests().set(crate::timer::SLICE);
        }
        // The split's own visit served no other request, so the sender's next
        // clause does: it arms the timer for the continuation now ready.
        self.clause_countdown = 1;
    }

    /// Sends `UNINIT` to `object` and answers a loud refusal if one escaped.
    fn run_one_uninit(&mut self, object: ObjRef) -> Option<Loud> {
        if !self.answers_uninit(object) {
            return None;
        }
        // A class is on two lists: the heap's flag, which a collection
        // readies, and `interrexx-classes`' pending list, which the termination
        // sweep drains. Dropping it from the second here is what keeps a
        // collection-driven finalizer from running a second time at exit --
        // measured, `class-uninit-at-driven-collection` printed its line
        // twice without it.
        if self.heap.is_class(object) {
            self.classes().forget_uninit_class(object);
        }
        let caller = self.caller();
        let outcome = pinned!(
            self,
            crate::pinning::PinKind::Uninit,
            self.send_message(object, UNINIT, None, &[], caller)
        );
        self.clear_failure_levels();
        match outcome {
            // A deadline is discarded here with the rest, and `execute`'s own
            // guard is what keeps that from being a silent answer: the flag
            // the check set outlives this match. Every later clause of this
            // run fails the same way, so the bounded sweep above finishes at
            // once rather than running the remaining finalizers.
            // No slice ends under the pin this send holds.
            Ok(_)
            | Err(Failure::Raised(_) | Failure::Exited(_) | Failure::Deadline | Failure::Slice) => {
                None
            }
            Err(Failure::Loud(loud)) => Some(*loud),
        }
    }

    /// Whether `object` still answers `UNINIT`,
    /// resolved the way [`native_has_method`] resolves the `HASMETHOD`
    /// message.
    fn answers_uninit(&mut self, object: ObjRef) -> bool {
        self.answers_message(object, "UNINIT")
    }

    /// Whether `object`'s behaviour answers `name`, which must already be
    /// upper case. The lookup is the one with no scope override,
    /// resolved the way [`native_has_method`] resolves `HASMETHOD`.
    pub(crate) fn answers_message(&mut self, object: ObjRef, name: &str) -> bool {
        if let Some(entry) = self.own_method_entry(object, name.as_bytes()) {
            return entry.is_some();
        }
        let Ok(behaviour) = self.receiver_behaviour(object) else {
            return false;
        };
        let classes = &self.object_model().classes;
        match behaviour {
            Behaviour::Instance { methods, .. } => classes.has_method_at(methods, name),
            Behaviour::ClassSide(class) => classes.class_has_method(class, name),
        }
    }

    /// Runs `UNINIT` on each of `batch`, oldest first, with the whole batch
    /// rooted for the length of the run.
    fn run_uninit_batch(&mut self, batch: Vec<ObjRef>, loud: &mut Vec<Loud>) {
        let parked = self.roots.activity_mut().park(batch.clone());
        for object in batch {
            loud.extend(self.run_one_uninit(object));
        }
        self.roots.activity_mut().release(parked);
    }

    /// Runs the `UNINIT` of every object a collection has readied, oldest first
    /// -- the pending-finalizer run, reached from a collection and so from
    /// `GC('force')`.
    pub(crate) fn run_ready_uninits(&mut self) -> Vec<Loud> {
        if self.processing_uninits {
            return Vec::new();
        }
        self.processing_uninits = true;
        let mut loud = Vec::new();
        let ready = std::mem::take(&mut self.uninit_ready);
        self.heap.clear_uninit_all(&ready);
        self.run_uninit_batch(ready, &mut loud);
        self.processing_uninits = false;
        loud
    }

    /// [`Interp::run_ending_uninits`] for an activation returning `value`,
    /// which stays rooted while they run.
    #[cold]
    #[inline(never)]
    pub(crate) fn run_uninits_at_return(&mut self, value: Option<ObjRef>) -> Result<(), Failure> {
        let parked = self.roots.activity_mut().park(value.into_iter().collect());
        let drained = self.run_ending_uninits();
        self.roots.activity_mut().release(parked);
        drained
    }

    /// The termination sweep: every live object still carrying the flag,
    /// then every class object with a class-side `UNINIT` in the
    /// order [`ClassRegistry::take_uninit_classes_in_sweep_order`] gives,
    /// and then the same again once.
    pub(crate) fn run_termination_uninits(&mut self) -> Vec<Loud> {
        if self.processing_uninits {
            return Vec::new();
        }
        self.processing_uninits = true;
        let mut loud = Vec::new();
        for _ in 0..SWEEPS {
            let flagged = self.heap.take_uninit_flagged();
            self.run_uninit_batch(flagged, &mut loud);
            let classes = self.classes().take_uninit_classes_in_sweep_order();
            for class in classes {
                loud.extend(self.run_one_uninit(class));
            }
        }
        self.uninit_ready.clear();
        self.processing_uninits = false;
        loud
    }

    /// [`Interp::resolve_in`] then [`Interp::invoke`], with
    /// [`Interp::unknown_or_nomethod`] behind them for a name the receiver's
    /// behaviour does not answer.
    pub(crate) fn send_message(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        args: &[Option<ObjRef>],
        caller: Caller,
    ) -> Result<Option<ObjRef>, Failure> {
        let started = self.begin_send(receiver, name, start_scope, args, caller)?;
        self.complete_send(started)
    }

    /// [`Interp::send_message`] for a name a program supplied, such as the one
    /// `unknown` is sent. Method names are held in upper case and the name
    /// is looked up as given, so one with a lower-case letter finds no method
    /// and goes to the receiver's `UNKNOWN`, or raises `SYNTAX 97.1`.
    pub(crate) fn send_message_as_given(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        caller: Caller,
    ) -> Result<Option<ObjRef>, Failure> {
        if !name.iter().any(u8::is_ascii_lowercase) {
            return self.send_message(receiver, name, None, args, caller);
        }
        if let Err(kind) = self.receiver_behaviour(receiver) {
            return Err(Loud::unknown_receiver(kind).into());
        }
        self.unknown_or_nomethod(receiver, name, args, Miss::NoMethod)
    }

    /// The `SYNTAX 97.1` that [`Interp::send_message_as_given`] raises, when
    /// it raises one without having needed its arguments: the name has a
    /// lower-case letter and the receiver answers no `UNKNOWN`.
    pub(crate) fn unanswered_name_as_given(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
    ) -> Option<Failure> {
        if !name.iter().any(u8::is_ascii_lowercase)
            || self.lookup(receiver, UNKNOWN, None).is_some()
        {
            return None;
        }
        Some(self.nomethod(receiver, name, Miss::NoMethod))
    }

    /// [`Interp::send_message`] up to the point a Rexx body it enters would
    /// run.
    pub(crate) fn begin_send(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        start_scope: Option<ObjRef>,
        args: &[Option<ObjRef>],
        caller: Caller,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let behaviour = match self.receiver_behaviour(receiver) {
            Ok(behaviour) => behaviour,
            Err(kind) => return Err(Loud::unknown_receiver(kind).into()),
        };
        match self.resolve_in(receiver, behaviour, name, start_scope, caller) {
            Ok(resolution) => self.begin_invoke(resolution, Some(behaviour), receiver, name, args),
            Err(miss) => self
                .unknown_or_nomethod(receiver, name, args, miss)
                .map(Started::Ran),
        }
    }

    /// **What the documented search order has left after the class chain**:
    /// `UNKNOWN` on the receiver's own behaviour, and the `NOMETHOD`
    /// condition beneath it when the behaviour answers no `UNKNOWN` either.
    #[cold]
    #[inline(never)]
    fn unknown_or_nomethod(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        miss: Miss,
    ) -> Result<Option<ObjRef>, Failure> {
        let Some(resolution) = self.lookup(receiver, UNKNOWN, None) else {
            return Err(self.nomethod(receiver, name, miss));
        };
        // **Each of the forward's arguments is rooted, and only the array's
        // root has a witness.** `alloc_with` collects *before* it allocates,
        // so the array's root is what carries it across the `text` below -- a
        // message name too long for a handle allocates there, and
        // `corpus/lang/message_send_unknown_forward.rex`'s last row is that
        // name. Measured with `collect_stress.rs`'s
        // collect-on-every-allocation and this root removed: that row's own
        // `a~items` then sends to a collected array and
        // `Interp::receiver_kind` refuses it, so the stress run exits 120
        // with `a message send to a value whose object is no longer live`
        // where the plain run prints the row. A message name of seven bytes
        // or fewer leaves the subset green, because `Interp::text` inlines it
        // and nothing allocates between the array and the send.
        let frame = self.roots.activity_mut().push_frame();
        // No arguments make the one-dimension empty array `.array~of()` does.
        let body = Body::Array {
            slots: args.to_vec().into(),
            dimensions: args.is_empty().then(|| Box::from([0].as_slice())),
        };
        let arguments = self.alloc_charged(BehaviourId::ARRAY, body);
        self.roots.activity_mut().push_temp(arguments);
        let missed = self.text(name);
        self.roots.activity_mut().push_temp(missed);
        let unknown_args = [Some(missed), Some(arguments)];
        let forwarded = match self.frameless_unknown(resolution, receiver) {
            Some(run) => pinned!(
                self,
                crate::pinning::PinKind::Unknown,
                run(self, receiver, &unknown_args)
            ),
            None => pinned!(
                self,
                crate::pinning::PinKind::Unknown,
                self.invoke(resolution, receiver, UNKNOWN, &unknown_args)
            ),
        };
        self.roots.activity_mut().pop_frame(frame);
        forwarded
    }

    /// The primitive `UNKNOWN` a send the receiver does not understand runs
    /// without a level of its own: `processUnknown` calls it directly for a
    /// `Stem`, a `VariableReference`, and a `Directory` or `StringTable` of
    /// the base class.
    fn frameless_unknown(&mut self, resolution: Resolution, receiver: ObjRef) -> Option<Unsent> {
        self.object_model().natives.get(resolution.method)?;
        let classes = self.classes();
        let system = |name| {
            classes.lookup(name).or_else(|| classes.system_lookup(name)) == Some(resolution.scope)
        };
        if system("Stem") {
            return Some(hash::stem::stem_unknown);
        }
        if system("VariableReference") {
            return Some(reference_unknown);
        }
        if !(system("Directory") || system("StringTable")) {
            return None;
        }
        (self.class_of_value(receiver) == Some(resolution.scope)).then_some(hash::hash_unknown)
    }

    /// The condition a send raises when the receiver's behaviour answers
    /// neither the message nor `UNKNOWN`, and **which condition that is
    /// depends on what is armed**.
    #[cold]
    #[inline(never)]
    fn nomethod(&mut self, receiver: ObjRef, name: &[u8], miss: Miss) -> Failure {
        let target = match self.message_target_text(receiver) {
            Ok(target) => target,
            Err(failure) => return failure,
        };
        let report = match miss {
            Miss::NoMethod => Raised::no_method(&target, name),
            Miss::Private => Raised::private_method(&target, name),
            Miss::PackageScope => Raised::package_scope_method(&target, name),
        };
        if self.trap_for(b"NOMETHOD").is_some_and(|trap| !trap.call) {
            // The receiver is the condition's `ADDITIONAL` (`reportNomethod`).
            self.activity.pending_additional = Some(receiver);
            Raised::nomethod(report, name).into()
        } else {
            report.into()
        }
    }

    /// Whether an argument of a [`NativeBody::Compared`] primitive may answer
    /// its `==` or `HASHCODE` with Rexx code.
    #[inline]
    fn argument_may_run_rexx(&mut self, args: &[Option<ObjRef>]) -> bool {
        args.iter().flatten().any(|value| self.may_run_rexx(*value))
    }

    /// Whether `value` is not of a base class, so its own methods may be
    /// Rexx code; a string or number answers on the handle alone.
    #[inline]
    pub(crate) fn may_run_rexx(&mut self, value: ObjRef) -> bool {
        match value.decode() {
            Decoded::Nil | Decoded::SmallInt(_) | Decoded::Text(_) => false,
            Decoded::Heap { .. } => {
                !matches!(
                    self.heap.get(value).map(|object| &object.body),
                    Some(Body::Text { .. } | Body::Num { .. })
                ) && !self.is_base_class(value)
            }
        }
    }

    /// Whether `value`'s `name` resolves to a method other than `Object`'s
    /// primitive.
    pub(crate) fn overrides_object_method(&mut self, value: ObjRef, name: &[u8]) -> bool {
        let Some(found) = self.lookup(value, name, None) else {
            return false;
        };
        if self.object_model().natives.get(found.method).is_none() {
            return true;
        }
        self.classes().lookup("Object") != Some(found.scope)
    }

    /// Whether `name` sent to `value` runs a method that is not one of the
    /// interpreter's own primitives.
    pub(crate) fn answers_by_method(&mut self, value: ObjRef, name: &[u8]) -> bool {
        match self.lookup(value, name, None) {
            Some(resolution) => self.object_model().natives.get(resolution.method).is_none(),
            None => false,
        }
    }

    /// The receiver as 97.1 names it: the string value, which is **not** the
    /// string value [`Interp::to_text`] answers for every receiver.
    fn message_target_text(&mut self, receiver: ObjRef) -> Result<Vec<u8>, Failure> {
        self.string_value_text(receiver)
    }

    /// One whole `target~name(...)` term: the receiver, the scope override,
    /// the arguments, the send, and `~~`'s replacement of the result by the
    /// target.
    pub(crate) fn message_term(
        &mut self,
        code: &crate::Code<'_>,
        term: &MessageTerm<'_>,
    ) -> Result<Option<ObjRef>, Failure> {
        let MessageTerm {
            target,
            name,
            super_class,
            args,
            cascade,
            assigned,
        } = *term;
        let receiver = self.eval(code, target)?;
        self.roots.activity_mut().push_temp(receiver);

        let start_scope = match super_class {
            None => None,
            Some(super_class) => {
                // Evaluated for its own trace lines and its own failures before
                // either check below, which is the check order.
                let scope = self.eval(code, super_class)?;
                self.roots.activity_mut().push_temp(scope);
                // The scope must be a class, and must be in the target's
                // behaviour. Both checks run before
                // the arguments are evaluated.
                if !self.heap.is_class(scope) {
                    return Err(Raised::scope_override_not_a_class().into());
                }
                self.validate_scope_override(receiver, Some(scope))?;
                Some(scope)
            }
        };

        let (mut values, mark) = self.take_value_buffer();
        let evaluated = self.evaluate_message_arguments(code, args, assigned, &mut values);
        let caller = self.caller();
        let result = evaluated
            .and_then(|()| self.send_message(receiver, name, start_scope, &values[mark..], caller));
        self.give_value_buffer(values, mark);
        let sent = result?;

        // `~~` replaces the result with the target **before** the send is
        // traced, so `>M>` shows the target for a cascade -- measured,
        // `'abc'~~length` traces `>M>   "LENGTH" => "abc"`.
        // It also gives a cascade a value
        // where the send itself had none, measured: `.K~~m` on a method
        // ending in a bare `return` yields the target at rc 0.
        let value = if cascade { Some(receiver) } else { sent };
        // **The tree path's `>M>` site.** Emitted here rather than from `eval`'s own
        // post-order `trace_intermediate` hook, where every other value
        // prefix lives, because the message-assignment form
        // (`Interp::exec_message`) never evaluates its term as an expression
        // and so never reaches that hook. Emitting from the one function
        // both forms do pass through is what keeps them from disagreeing.
        // The position is the same either way: this is the last thing the
        // term does before returning its value.
        if let Some(value) = value
            && self.trace_mode().intermediates
        {
            self.trace_sent(name, value)?;
        }
        Ok(value)
    }

    /// The argument loop [`Interp::message_term`] runs, split out so the
    /// borrowed value buffer is returned on the failure path too.
    fn evaluate_message_arguments(
        &mut self,
        code: &crate::Code<'_>,
        args: &[Option<Expr>],
        assigned: Option<&Expr>,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        if let Some(expr) = assigned {
            values.push(Some(self.eval_traced_argument(code, expr)?));
        }
        for arg in args {
            match arg {
                // An omitted position traces an empty value line, not no
                // line -- the same rule a call's own argument list follows.
                None => {
                    self.trace_argument(self.activity.clause_state.current_value_indent, b"");
                    values.push(None);
                }
                Some(expr) => values.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        Ok(())
    }

    /// Records the traceback line a failing native method contributes, and
    /// closes the level so the sending clause records its own.
    pub(crate) fn blame_native_method(
        &mut self,
        name: &[u8],
        scope: &str,
        target: ObjRef,
        arguments: &[Option<ObjRef>],
    ) {
        let level = NativeLevel {
            method: true,
            name,
            target,
            arguments,
            executable: None,
            activation: None,
        };
        self.blame_native_level(Raised::compiled_method_line(name, scope), level);
    }

    /// [`Interp::blame_native_method`] for a method an `EXTERNAL` directive
    /// bound, whose level carries the package that directive was written in.
    /// `failure` is what the method answered.
    /// `reraised` is whether the call's failure is the condition it raised
    /// while running.
    pub(crate) fn blame_external_method(
        &mut self,
        name: &[u8],
        scope: &str,
        method: MethodId,
        target: ObjRef,
        arguments: &[Option<ObjRef>],
        reraised: bool,
    ) {
        let reported_on_return = std::mem::take(&mut self.activity.native_reported_on_return);
        let activation = Some(NativeActivationLevel {
            // A `LIBRARY REXX` entry's package is `REXX`.
            package: Some(
                self.external_package_program(method)
                    .map_or(Package::Rexx, Package::Program),
            ),
            reraised,
            reported_on_return,
        });
        // The native activation has given up the receiver before it reports
        // that error, so the level is a routine's: no target, and the
        // `Compiled routine` line.
        if reported_on_return {
            let executable = context::native_method_executable(self, target, name);
            let level = NativeLevel {
                method: false,
                name,
                target: ObjRef::NIL,
                arguments,
                executable,
                activation,
            };
            self.blame_native_level(Raised::compiled_routine_line(name), level);
            return;
        }
        let level = NativeLevel {
            method: true,
            name,
            target,
            arguments,
            executable: None,
            activation,
        };
        self.blame_native_level(Raised::compiled_method_line(name, scope), level);
    }

    /// The same for a routine of an internal package, whose traceback line
    /// names the routine alone and upcased -- measured, `filespec('D')` is
    /// reported for `Filespec` as `"FILESPEC"`. `given` is the `Routine` the
    /// call ran, `.nil` for the one standing for the row.
    pub(crate) fn blame_internal_routine(
        &mut self,
        row: &'static crate::internal_routines::InternalRoutine,
        name: &[u8],
        arguments: &[Option<ObjRef>],
        failure: &Failure,
        given: ObjRef,
    ) {
        if self.activity.failure_site.is_some() {
            return;
        }
        let executable = match given {
            ObjRef::NIL => self.internal_routine_object(crate::internal_routines::index_of(row)),
            given => given,
        };
        let level = NativeLevel {
            method: false,
            name,
            target: ObjRef::NIL,
            arguments,
            executable: Some(executable),
            activation: Some(NativeActivationLevel {
                package: Some(Package::Rexx),
                reraised: failure.reraised_by_native_call(),
                reported_on_return: false,
            }),
        };
        self.blame_native_level(Raised::compiled_routine_line(name), level);
    }

    /// The same for a routine of a loaded library, whose level carries the
    /// package its shared code reports, if any. `ran` and `code` are what the
    /// routine's frame held, from which [`Interp::routine_executable`] answers.
    pub(crate) fn blame_native_routine(
        &mut self,
        name: &[u8],
        program: Option<ProgramId>,
        arguments: &[Option<ObjRef>],
        ran: crate::RoutineObject,
        code: Option<usize>,
    ) {
        let reraised = std::mem::take(&mut self.activity.native_reraise);
        let reported_on_return = std::mem::take(&mut self.activity.native_reported_on_return);
        if self.activity.failure_site.is_some() {
            return;
        }
        let executable = self.routine_executable(ran, code);
        let level = NativeLevel {
            method: false,
            name,
            target: ObjRef::NIL,
            arguments,
            executable,
            activation: Some(NativeActivationLevel {
                package: program.map(Package::Program),
                reraised,
                reported_on_return,
            }),
        };
        self.blame_native_level(Raised::compiled_routine_line(name), level);
    }

    /// Records `text` as a native level's whole traceback line, first call
    /// wins, and closes the level so the sending clause records its own.
    fn blame_native_level(&mut self, text: Vec<u8>, level: NativeLevel<'_>) {
        if self.activity.failure_site.is_some() {
            return;
        }
        let executable = if level.method {
            context::native_method_executable(self, level.target, level.name)
        } else {
            level.executable
        };
        let frame = crate::dispatch::context::build_native_level_frame(
            self,
            (!level.method).then_some(0),
            level.name,
            level.target,
            level.arguments,
            &text,
            executable,
        );
        self.activity.failure_frame = Some(frame);
        match level.activation {
            // The native activation re-raises a SYNTAX condition in the
            // caller, whose `POSITION`
            // and program are the caller's.
            Some(NativeActivationLevel { reraised: true, .. }) => self.reraise_failure_levels(),
            // Anything else keeps the object built where it was raised, and
            // a condition raised here names this frame's package, which has
            // no line.
            Some(NativeActivationLevel {
                package: Some(package),
                reraised: false,
                reported_on_return,
            }) => {
                self.activity.failure_origin.get_or_insert((package, None));
                if reported_on_return {
                    let name = match self.reported_package(package) {
                        Package::Program(id) => self.program_display_name(id).to_vec(),
                        Package::Rexx => crate::LIBRARY_PACKAGE_NAME.to_vec(),
                    };
                    self.activity.failure_site = Some(FailureSite::Packaged { text, name });
                    self.seal_site_level();
                    return;
                }
            }
            Some(_) | None => {}
        }
        self.activity.failure_site = Some(FailureSite::Rendered { text });
        self.seal_site_level();
    }
}

/// What the `StackFrame` of a failing native level is built from; see
/// [`context::build_native_level_frame`].
struct NativeLevel<'a> {
    method: bool,
    name: &'a [u8],
    target: ObjRef,
    arguments: &'a [Option<ObjRef>],
    /// A routine's `Routine` object, where this crate has one.
    executable: Option<ObjRef>,
    /// `None` for an `InternalActivationFrame`, which neither has a package nor
    /// re-raises.
    activation: Option<NativeActivationLevel>,
}

/// A level run as a native-method activation.
struct NativeActivationLevel {
    /// What the level's package query answers.
    package: Option<Package>,
    /// Whether the condition leaves it re-raised: a SYNTAX condition
    /// ([`Failure::reraised_by_native_call`]).
    reraised: bool,
    /// Whether the level reported the error itself once its call had
    /// returned ([`Activity::native_reported_on_return`]), so that its package,
    /// not a caller's line, is what the report's major line names.
    reported_on_return: bool,
}

/// `VariableReference~name`: the referenced variable's own spelling.
fn native_reference_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(
            crate::Raised::receiver_kind("a value that is not a variable reference").into(),
        );
    };
    let name = reference.name.to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `VariableReference~value`: what the referenced variable holds.
/// It answers the variable's derived name when it holds
/// nothing.
fn native_reference_value(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(
            crate::Raised::receiver_kind("a value that is not a variable reference").into(),
        );
    };
    match interp.referenced_value(reference) {
        Some(value) => Ok(Some(value)),
        None => {
            let name = reference.name.to_vec();
            Ok(Some(interp.text_built(name)))
        }
    }
}

/// `VariableReference~value=`: writes the referenced variable.
/// An omitted value raises 93.903.
fn native_reference_value_set(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("VALUE").into());
    };
    let Some(reference) = interp.as_variable_reference(receiver) else {
        return Err(
            crate::Raised::receiver_kind("a value that is not a variable reference").into(),
        );
    };
    let home = reference.home.clone();
    let name = reference.name.clone();
    // The stem and simple assignment bits are sorted out here, so a stem
    // reference assigns what the bare `stem. = value` assigns rather than
    // storing the value itself -- a plain value in a stem-named slot is a state
    // nothing else in this crate can produce.
    let value = if crate::run::shape_of(&name) == crate::run::NameShape::Stem {
        interp.stem_assignment_value(&name, value)
    } else {
        value
    };
    match home {
        interrexx_core::VarRefHome::Cell(cell) => interp.roots.set_slot_value(cell, value),
        interrexx_core::VarRefHome::Instance { owner, scope } => {
            interp.set_pool_variable(owner, scope, &name, value);
        }
    }
    Ok(None)
}

/// `VariableReference~unknown(name, arguments)`: forwards to the referenced
/// value, which is what makes every message
/// the class does not define read as one to the variable's value.
fn native_reference_unknown(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    reference_unknown(interp, receiver, args)
}

/// [`native_reference_unknown`]'s work, which `processUnknown` also runs without a send.
fn reference_unknown(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(message)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let message = required_string_argument(interp, message, 1)?;
    let name = interp.to_text(message).to_vec();
    let Some(Some(arguments)) = args.get(1).copied() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    if name.iter().any(u8::is_ascii_lowercase) {
        let referenced = referenced_receiver(interp, receiver)?;
        if let Some(refusal) = interp.unanswered_name_as_given(referenced, &name) {
            return Err(refusal);
        }
    }
    let forwarded = array_argument(interp, arguments, ArrayArgument::Positional)?;
    let referenced = referenced_receiver(interp, receiver)?;
    let caller = interp.caller();
    interp.send_message_as_given(referenced, &name, &forwarded, caller)
}

/// `VariableReference~request(class)`: forwards to the referenced value,
/// which handles none of them itself.
fn native_reference_request(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let referenced = referenced_receiver(interp, receiver)?;
    let caller = interp.caller();
    // A request with no argument still sends the value one omitted argument.
    let omitted = [None];
    let args = if args.is_empty() { &omitted[..] } else { args };
    interp.send_message(referenced, b"REQUEST", None, args, caller)
}

/// The value a reference receiver names, materialised as an object so that a
/// message can be sent to it: the value its chain of references ends at, or
/// an unset variable's name. A cyclic reference is 11.1, the error unbounded
/// recursion answers.
fn referenced_receiver(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    match interp.referent(receiver) {
        crate::value::Referent::Value(referenced) => Ok(referenced),
        crate::value::Referent::Unset(last) => interp
            .referenced_object(last.unwrap_or(receiver))
            .ok_or_else(|| {
                crate::Raised::receiver_kind("a value that is not a variable reference").into()
            }),
        crate::value::Referent::Cycle => Err(crate::error::Raised::insufficient_stack().into()),
    }
}

#[cfg(test)]
mod tests;
