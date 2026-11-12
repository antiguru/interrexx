//! `Class`'s own methods: its identity and graph readers, `define`, `subclass`,
//! `new`, `inherit`, the metaclass factory, and a package's `addClass`.

use super::{
    BehaviourId, Body, ClassKind, Cleared, Failure, INIT, InheritRefusal, Interp, Loud, MethodSlot,
    NativeStarted, ObjRef, Object, ObjectMethods, Primitive, Raised, Started, Then, hash,
    required_string_named_argument,
};

mod method_table;

use method_table::method_table_pairs;
pub(super) use method_table::native_define_methods;

/// `Class~baseClass`, bound as a method of `.Class`.
pub(super) fn native_base_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().base_class(class)))
}

/// The class object a receiver whose messages resolve against a class's
/// **class** behaviour is, or the refusal for one that is not a class object.
pub(super) fn class_receiver(interp: &Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    match interp.receiver_kind(receiver) {
        Ok(Primitive::Class(class)) => Ok(class),
        Ok(_) => Err(crate::Raised::receiver_kind("a value that is not a class object").into()),
        Err(kind) => Err(Loud::unknown_receiver(kind).into()),
    }
}

/// The argument a method that requires a class object was given, which refuses
/// an omitted argument with 88.901 and a value that is not a class object with
/// 88.914.
pub(super) fn class_argument(interp: &Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    match interp.heap.is_class(argument) {
        true => Ok(argument),
        false => Err(Raised::argument_not_a_class("class").into()),
    }
}

/// `Class~id`: the name the class was declared with, case unmodified.
pub(super) fn native_id(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let id = interp.class_id_text(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(id)))
}

/// `Class~defaultName`: `The <id> class`.
pub(super) fn native_class_default_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = interp.classes().default_name(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `Class~metaClass`.
pub(super) fn native_metaclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().metaclass(class)))
}

/// `Class~superClass`: the class's own direct superclass, or `.nil` for
/// `.Object`. It reads the **first** entry of the superclass list and not its
/// last.
pub(super) fn native_superclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(
        interp.classes().superclass(class).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~superClasses`: a **fresh** array of the class's own direct
/// superclasses, so it hands out a copy rather than the class's own list.
pub(super) fn native_superclasses(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let items: Vec<Option<ObjRef>> = interp
        .classes()
        .superclasses(class)
        .iter()
        .map(|class| Some(*class))
        .collect();
    // Every item is a class identity, which lives outside the arena
    // (`interrexx_core::CLASS_SLOT_BASE`), so the allocation below cannot collect
    // one of them out from under this array.
    Ok(Some(
        interp.alloc_charged(BehaviourId::ARRAY, Body::array(items)),
    ))
}

/// `Class~isSubclassOf(class)`: whether the receiver **is** `class` or derives
/// from it, so the receiver counts as compatible with itself.
pub(super) fn native_is_subclass_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(interp, args)?;
    let class = class_receiver(interp, receiver)?;
    let answer = interp.classes().is_a(class, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Class~annotation(name)`, and the same method at `Method`, `Routine` and
/// `Package`: the annotation `name` holds, or `.nil`.
pub(super) fn native_annotation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let argument = required_string_named_argument(interp, argument, "name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let table = interp.annotations_of(receiver)?;
    Ok(Some(
        interp.native_entry(table, &name).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~annotations`, and the same method at `Method`, `Routine` and
/// `Package`: the receiver's own annotation table.
pub(super) fn native_annotations(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.annotations_of(receiver)?))
}

/// `Class~method(name)`: the method object `name` names **in this class's own
/// instance dictionary**, and 97.1 for anything else.
pub(super) fn native_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let class = class_receiver(interp, receiver)?;
    match interp
        .classes()
        .own_instance_slot(class, &String::from_utf8_lossy(&name))
    {
        None | Some(MethodSlot::Withdrawn) => {
            let target = interp.class_default_name(class).to_vec();
            Err(Raised::no_method(&target, &name).into())
        }
        Some(MethodSlot::Hidden) => Ok(Some(ObjRef::NIL)),
        Some(MethodSlot::Defined { scope, method }) => {
            let record = crate::ExecutableRecord {
                source: interp.installed_executable_source(method),
                installed: Some(method),
                routine: None,
                owns_installed: false,
            };
            Ok(Some(
                interp.method_object(class, false, &name, scope, record)?,
            ))
        }
    }
}

/// `Method~scope`: the class the method object was defined at, `.nil` for a
/// method object no class has taken -- [`Interp::method_scope`] carries the
/// measurements.
pub(super) fn native_scope(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.method_scope(receiver)?))
}

/// The `REXX_DEFINED` refusal every class mutator opens with, one per mutator.
fn rexx_defined_lock(interp: &mut Interp, class: ObjRef) -> Result<(), Failure> {
    // **Open while the interpreter's own library runs**, which is the state the
    // library image is built in: its prologue is a run of `~inherit` clauses
    // against exactly the classes this flag guards, and the `REXX_DEFINED` mark
    // is set on them at image-save time rather than before. This crate flags
    // each library class as `Interp::install_class` creates it, so the bypass
    // is not a convenience: it is what lets the prologue's own `~inherit`
    // clauses run at all. See `Interp::library_bootstrap` for what else the
    // same flag opens and for why no program can be inside it.
    if !interp.library_bootstrap && interp.classes().is_rexx_defined(class) {
        return Err(Raised::rexx_defined_class().into());
    }
    Ok(())
}

/// Who asks [`method_source_lines`] for a source, which decides what a value
/// that is neither a string nor an array raises.
#[derive(Copy, Clone)]
pub(super) enum SourceTaker {
    /// The class-creation arguments of `Package~new` take those two alone:
    /// 93.961.
    Executable,
    /// `Method~new` takes a `Method` object too: 93.974.
    Method,
    /// `Package~load`'s array argument is 88.913.
    Package,
}

/// The physical lines a method source is compiled from, or the raise for a
/// value that is not one -- `processExecutableSource`.
pub(super) fn method_source_lines(
    interp: &mut Interp,
    source: ObjRef,
    position: &'static str,
    taker: SourceTaker,
) -> Result<Vec<Vec<u8>>, Failure> {
    // `Package~loadPackage` splits a string at its line ends, as the
    // string's `makeArray` does, so one string can hold several lines and the
    // null string none (`r[class.package.loadpackage]`); every other taker
    // reads a string as one line.
    if !matches!(taker, SourceTaker::Package) && is_source_line(interp, source) {
        return Ok(vec![interp.to_text(source).to_vec()]);
    }
    // `requestArray`, then `makeString` for a value that is not a primitive.
    // For example, `.method~new('m', x)` takes `.environment` as its index
    // array, a `List` or `Queue` of lines, a stem's tails, and an instance
    // whose `MAKEARRAY` or `MAKESTRING` answers the lines.
    let array = if interp.array_slots_of(source).is_some() {
        Some(source)
    } else {
        let converted = interp.request_array_value(source)?;
        // An instance's `REQUEST('ARRAY')` that answers an object that is
        // neither an array nor `.nil` is refused here, and no
        // `REQUEST('STRING')` follows it (Deviation 115).
        if let Some(answer) = converted
            && answer != ObjRef::NIL
            && interp.array_slots_of(answer).is_none()
            && !interp.is_base_class(source)
        {
            return Err(Raised::argument_not_single_dimensional(position).into());
        }
        converted.filter(|array| interp.array_slots_of(*array).is_some())
    };
    if let Some(array) = array {
        interp.roots.activity_mut().push_temp(array);
        let slots = interp.array_slots_of(array).unwrap_or_default();
        let last_item = slots
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |at| at + 1);
        let mut lines = Vec::with_capacity(last_item);
        for slot in &slots[..last_item] {
            let Some(item) = *slot else {
                return Err(Raised::method_source_not_all_strings(position).into());
            };
            lines.push(array_item_line(interp, item, position)?);
        }
        return Ok(lines);
    }
    // `arrayArgument` has no `makeString` step.
    if !matches!(taker, SourceTaker::Package) && !interp.is_base_class(source) {
        let wanted = interp.text(b"STRING");
        interp.roots.activity_mut().push_temp(wanted);
        let caller = interp.caller();
        let answered = interp.send_message(source, b"REQUEST", None, &[Some(wanted)], caller)?;
        // Any answer but `.nil` is the line, as its string value: a
        // `MAKESTRING` answering an `Array` compiles the line `an Array`.
        if let Some(line) = answered.filter(|line| *line != ObjRef::NIL) {
            return Ok(vec![interp.string_value_text(line)?]);
        }
    }
    Err(match taker {
        SourceTaker::Executable => Raised::source_not_string_or_array(position),
        SourceTaker::Method => Raised::not_a_method_or_source(position),
        SourceTaker::Package => Raised::argument_not_single_dimensional(position),
    }
    .into())
}

/// The line one item of a source array gives: a string is its own line, and
/// any other item is sent `REQUEST('STRING')`, whose answer is the line when
/// it is a string -- a `MutableBuffer` and an instance with a `MAKESTRING`
/// answer one. An item that gives no string, `.nil` among them, is the
/// 93.952 a hole is.
fn array_item_line(
    interp: &mut Interp,
    item: ObjRef,
    position: &'static str,
) -> Result<Vec<u8>, Failure> {
    if is_source_line(interp, item) {
        return Ok(interp.to_text(item).to_vec());
    }
    if item != ObjRef::NIL {
        let wanted = interp.text(b"STRING");
        interp.roots.activity_mut().push_temp(wanted);
        let caller = interp.caller();
        let answered = interp.send_message(item, b"REQUEST", None, &[Some(wanted)], caller)?;
        if let Some(line) = answered.filter(|line| is_source_line(interp, *line)) {
            return Ok(interp.to_text(line).to_vec());
        }
    }
    Err(Raised::method_source_not_all_strings(position).into())
}

/// Whether one value is a source line: a string for the whole source, and its
/// string conversion for an array's items.
fn is_source_line(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.receiver_kind(value),
        Ok(Primitive::String | Primitive::SmallInt)
    )
}

/// `lines` parsed as a program, or restored where they are an encoded image, or
/// the `SYNTAX` condition they raise, positioned within them (for example `1`
/// for `.Routine~new('T', 'return arg(1')` on line 2 of its caller).
fn parse_source_lines(
    interp: &mut Interp,
    name: &[u8],
    lines: &[&[u8]],
) -> Result<interrexx_parse::Program, Failure> {
    let source = match crate::compiled::lines_source(lines, name) {
        Some(restored) => restored?,
        None => interrexx_parse::ProgramSource::from_lines(lines),
    };
    interrexx_parse::program_from(source)
        .map_err(|rejected| interp.raise_parse_failure(&rejected, Some(name)))
}

/// The package an executable compiled from `parsed` resolves names through
/// after its own: `parent`, or the caller's where it is `None`, and none for
/// a restored image, which inherits no context (measured, a restored routine
/// calling one its caller defines raises 43.1).
fn executable_parent(
    interp: &mut Interp,
    parsed: &interrexx_parse::Program,
    parent: Option<ObjRef>,
) -> Result<Option<ObjRef>, Failure> {
    if !parsed.source.is_traceable() {
        return Ok(None);
    }
    match parent {
        Some(parent) => Ok(Some(parent)),
        None => interp
            .running_program()
            .map(|program| interp.package_object(crate::plan::Package::Program(program)))
            .transpose(),
    }
}

/// The compiling arm of `Method~new`: the `Method` object a source text
/// becomes, carrying no scope, for a caller that is about to install it.
pub(super) fn compile_method_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
    taker: SourceTaker,
) -> Result<ObjRef, Failure> {
    compile_method_source_in(interp, name, source, position, taker, |_| Ok(None))
}

/// [`compile_method_source`] whose directives install into a package whose
/// parent `parent` resolves, or the caller's where it answers `None`. The
/// source is checked before `parent` runs, so a bad source and a bad context
/// together raise the source's fault (`r[class.method.new.order]`).
pub(super) fn compile_method_source_in(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
    taker: SourceTaker,
    parent: impl FnOnce(&mut Interp) -> Result<Option<ObjRef>, Failure>,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position, taker)?;
    let parent = parent(interp)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = parse_source_lines(interp, name, &borrowed)?;
    method_from_program(interp, name, parsed, parent)
}

/// The `Method` a parsed or restored program becomes, carrying no scope.
fn method_from_program(
    interp: &mut Interp,
    name: &[u8],
    parsed: interrexx_parse::Program,
    parent: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let parent = executable_parent(interp, &parsed, parent)?;
    method_in_package(interp, name, parsed, parent)
}

/// The `Method` a parsed program becomes, its package's parent `parent`
/// and none where it is `None`.
pub(crate) fn method_in_package(
    interp: &mut Interp,
    name: &[u8],
    parsed: interrexx_parse::Program,
    parent: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    // The method generation installs them as routine generation does; see
    // [`compile_routine_source`].
    if !parsed.directives.is_empty() {
        return interp.install_executable(parsed, name, false, parent, false, |interp, id| {
            interp.compiled_method_names.insert(id, name.into());
        });
    }
    let method_class = interp.method_class();
    let object = interp.native_instance(method_class);
    // Every `Method` object this crate builds carries an annotation table, so
    // that `~annotations` and `~annotation` answer from the object rather than
    // from a way back to a directive. A compiled method has no directive and
    // its table starts empty, so `.k~define("m", 'return 1')` then
    // `.k~method("M")~annotation('x')` is `The NIL object`.
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_body(object, name, parsed, parent)?;
    Ok(object)
}

/// The compiling arm of `Routine~new`: [`compile_method_source_in`] answering
/// a `Routine` instead.
pub(super) fn compile_routine_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
    parent: impl FnOnce(&mut Interp) -> Result<Option<ObjRef>, Failure>,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position, SourceTaker::Executable)?;
    let parent = parent(interp)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = parse_source_lines(interp, name, &borrowed)?;
    routine_from_program(interp, name, parsed, parent)
}

/// The `Routine` a parsed or restored program becomes, its directives
/// installed in a package of its own.
fn routine_from_program(
    interp: &mut Interp,
    name: &[u8],
    parsed: interrexx_parse::Program,
    parent: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    // Routine generation installs them into the routine's own package, whose
    // parent is the caller's: `.Routine~new('T', .array~of('::class
    // a1'))~package~classes~hasIndex('A1')` is `1`.
    let parent = executable_parent(interp, &parsed, parent)?;
    routine_in_package(interp, name, parsed, parent)
}

/// [`method_in_package`] answering a `Routine`.
pub(crate) fn routine_in_package(
    interp: &mut Interp,
    name: &[u8],
    parsed: interrexx_parse::Program,
    parent: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    if !parsed.directives.is_empty() {
        return interp.install_executable(parsed, name, true, parent, false, |interp, id| {
            interp.compiled_method_names.insert(id, name.into());
        });
    }
    let routine_class = interp.routine_class();
    let object = interp.native_instance(routine_class);
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_routine(object, name, parsed, parent)?;
    Ok(object)
}

/// [`Interp::scoped_method`] as a `Result`.
pub(super) fn scoped_method(
    interp: &mut Interp,
    method: ObjRef,
    scope: ObjRef,
) -> Result<(ObjRef, interrexx_core::MethodId), Failure> {
    interp
        .scoped_method(method, scope)
        .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))
}

/// The `method name` argument `~define`, `~delete` and `~method` share:
/// required, string-valued, and upcased before it reaches a dictionary.
pub(super) fn method_name_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<u8>, Failure> {
    Ok(method_name_pair(interp, args)?.1)
}

/// [`method_name_argument`] with the spelling the caller wrote kept beside
/// the dictionary key.
fn method_name_pair(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>), Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let written = interp.to_text(argument).to_vec();
    let key = written.to_ascii_uppercase();
    Ok((written, key))
}

/// `Class~define(name, method)`: install one instance method on the receiver.
pub(super) fn native_define(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let (written, name) = method_name_pair(interp, args)?;
    match args.get(1).copied().flatten() {
        None => {
            interp
                .classes()
                .hide_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
            interp.charge_retired_behaviours();
        }
        Some(ObjRef::NIL) => {
            interp
                .classes()
                .withdraw_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
            interp.charge_retired_behaviours();
        }
        Some(source) => {
            let source = if interp.receiver_kind(source) == Ok(Primitive::Method) {
                source
            } else {
                compile_method_source(interp, &written, source, "method", SourceTaker::Method)?
            };
            interp
                .define_method_object(class, &name, source)
                .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))?;
        }
    }
    Ok(None)
}

/// `Class~subclass(id, metaclass, classMethods)`: build a class derived from
/// the receiver and answer it. It forwards to the same construction a
/// `::CLASS` directive performs, with no package where the directive passes
/// its own.
pub(super) fn native_subclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Regular)
}

/// `Class~mixinClass(id, metaclass, classMethods)`: the same factory, marking
/// what it builds a mixin.
pub(super) fn native_mixin_class_factory(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Mixin)
}

/// `Class~new(id, ...)`: a copy of the receiver that answers the receiver's
/// instance methods, derives its instances from `Object` alone and is in no
/// subclass list, sent `INIT` with the arguments after the id. For example,
/// `.class~new('FooBar')` answers `FooBar Object 0 The REXX Package` for its
/// id, superclass, membership of `.object~subclasses` and package, and its own
/// `~new` is 97.1.
pub(super) fn native_new_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    let name = required_class_id(interp, args)?;
    let metaclass = if interp.is_primitive_class(class) {
        interp.object_model().metaclass
    } else {
        class
    };
    let root = interp.object_model().object;
    let id = interp.mint_class();
    interp.classes().define_cloned_class(
        id,
        &String::from_utf8_lossy(&name),
        class,
        root,
        metaclass,
    );
    charge_class(interp, id);
    interp.copy_class_package(class, id);
    interp.classes().check_uninit(id);
    interp.flag_class_uninit(id);
    begin_init(interp, id, &args[1..])
}

/// `Object~new`: what every class that declares no `NEW` of its own answers.
pub(super) fn native_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let class = class_receiver(interp, receiver)?;
    let object = new_instance(interp, class)?;
    begin_init(interp, object, args)
}

/// `object`'s `INIT` send with `args`, answering `object` once it has run.
pub(super) fn begin_init(
    interp: &mut Interp,
    object: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<NativeStarted, Failure> {
    let caller = interp.caller();
    Ok(match interp.begin_send(object, INIT, None, args, caller)? {
        Started::Ran(_) => NativeStarted::Ran(Some(object)),
        Started::Entered => NativeStarted::Entered(Then::Answer(object)),
    })
}

/// Object completion up to but not including the `INIT` send: the abstract
/// check, the behaviour, the rooting and the `UNINIT` registration, in that
/// order.
pub(super) fn new_instance(interp: &mut Interp, class: ObjRef) -> Result<ObjRef, Failure> {
    if interp.classes().is_abstract(class) {
        let id = interp.classes().id_string(class).as_bytes().to_vec();
        return Err(Raised::abstract_class(&id).into());
    }
    // The object's behaviour is set from the class here, once. What the class
    // holds later is a different behaviour or the same one rebuilt, and an
    // instance dispatches against the one its class held at construction.
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let object = interp.alloc_with(
        interrexx_core::BehaviourId::OBJECT,
        Body::Instance {
            class,
            behaviour,
            name: None,
            pools: interrexx_core::ScopePools::new(),
            own: None,
            native: None,
        },
    );
    // `ProtectedObject p(newObj)`: the `INIT` send allocates, and nothing else
    // holds the object until it returns.
    interp.roots.activity_mut().push_temp(object);
    // See `Interp::reqstr_armed` for why an instance arms the protocol
    // outright rather than by the name a directive installed.
    interp.reqstr_armed = true;
    if interp.classes().has_uninit(class) {
        interp.heap.set_uninit(object);
    }
    Ok(object)
}

/// Class creation, in its own order.
fn class_factory(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    kind: ClassKind,
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let metaclass = factory_metaclass(interp, class, args)?;
    let name = class_id_argument(interp, metaclass, args)?;
    // A mixin class is one only once its second `init` has run: until then
    // it reads as a regular class, its own base class.
    let id = new_subclass(interp, class, &name, metaclass, ClassKind::Regular)?;
    charge_class(interp, id);
    interp.record_packageless_class(id);
    if let Some(enhancing) = args.get(2).copied().flatten() {
        // The receiver lists the class only once its class methods are in.
        if let Err(failure) = enhance_class_methods(interp, id, enhancing) {
            interp.classes().unlist_subclass(id);
            return Err(failure);
        }
    }
    finish_subclass(interp, id)?;
    if matches!(kind, ClassKind::Mixin) {
        interp.classes().complete_mixin(id);
    }
    Ok(Some(id))
}

/// The body bytes a class is charged toward the next collection, per name its
/// two behaviours resolve, and again per method of its own. Measured with
/// collection off, on `Package~new`s of twenty `::CLASS`es: a subclass of
/// `Object` with no methods of its own weighs about 20 kB, one of `Array`
/// about 31 kB, and each `::METHOD` of its own about 8 kB more.
const CLASS_BYTES_PER_NAME: usize = 200;
const CLASS_BYTES_PER_OWN_METHOD: usize = 8 << 10;

/// Class creation up to its enhancing methods: the class `metaclass`'s `NEW`
/// answers for `name`, made a subclass of `class`, rooted. Where `NEW` is
/// `Class`'s own and the metaclass's instances take `Object`'s `INIT`, the
/// class is built in place, which nothing can tell from the clone `Class~new`
/// makes and the rewiring that follows.
pub(crate) fn new_subclass(
    interp: &mut Interp,
    class: ObjRef,
    name: &[u8],
    metaclass: ObjRef,
    kind: ClassKind,
) -> Result<ObjRef, Failure> {
    let id = if builds_in_place(interp, metaclass) {
        let id = interp.mint_class();
        interp.classes().define_unregistered_class(
            id,
            &String::from_utf8_lossy(name),
            Some(class),
            kind,
            metaclass,
        );
        id
    } else {
        let argument = interp.text(name);
        interp.roots.activity_mut().push_temp(argument);
        let caller = interp.caller();
        let answer = interp.send_message(metaclass, b"NEW", None, &[Some(argument)], caller)?;
        // An answer that is not a class is refused here (oracle defect,
        // `oracle-crashes.txt` entry 8: the reference run casts it to a class
        // unchecked and crashes).
        let Some(id) = answer.filter(|answer| interp.heap.is_class(*answer)) else {
            return Err(Loud::receiver_class(
                "a metaclass whose NEW answers something other than a class",
            )
            .into());
        };
        // A class `Class~new` did not build, or one a subclass already took, is
        // not rewired in place here, as that would damage a class others hold
        // (Deviation 32).
        if !interp.classes().is_cloned(id) {
            return Err(Loud::metaclass_new_reused_class().into());
        }
        interp.roots.activity_mut().push_temp(id);
        interp.classes().reparent(id, class, kind, metaclass);
        id
    };
    Ok(id)
}

/// Charges a class a program built toward the next collection, which its
/// behaviours' weight would otherwise not reach.
pub(crate) fn charge_class(interp: &mut Interp, class: ObjRef) {
    let classes = interp.classes();
    let (names, own) = (
        classes.behaviour_names(class),
        classes.own_method_names(class),
    );
    interp.charge_unheld(names * CLASS_BYTES_PER_NAME + own * CLASS_BYTES_PER_OWN_METHOD);
}

/// Whether `metaclass`'s `NEW` is `Class`'s own and its instances take
/// `Object`'s `INIT`, so the `INIT` `Class~new` sends does nothing.
pub(crate) fn builds_in_place(interp: &mut Interp, metaclass: ObjRef) -> bool {
    let modelled = interp.object_model().metaclass;
    let object = interp.object_model().object;
    let classes = interp.classes();
    classes
        .lookup_class_method(metaclass, "NEW")
        .map(|(scope, _)| scope)
        == Some(modelled)
        && classes.lookup_instance_method(metaclass, "INIT")
            == classes.lookup_instance_method(object, "INIT")
}

/// The tail of class creation, in its order, which is the same sequence
/// `Interp::install_class_at` makes for a directive.
fn finish_subclass(interp: &mut Interp, id: ObjRef) -> Result<(), Failure> {
    interp.classes().check_uninit(id);
    interp.flag_class_uninit(id);
    let caller = interp.caller();
    interp.send_message(id, INIT, None, &[], caller)?;
    interp.classes().refresh_parent_has_uninit(id);
    Ok(())
}

/// The metaclass a class factory builds from: the second argument, or the
/// receiver's own where the send omits it, which has to be a metaclass.
fn factory_metaclass(
    interp: &mut Interp,
    class: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let metaclass = match args.get(1).copied().flatten() {
        None => interp.classes().metaclass(class),
        Some(named) => named,
    };
    let is_class = matches!(interp.receiver_kind(metaclass), Ok(Primitive::Class(_)));
    if !is_class || !interp.classes().is_metaclass(metaclass) {
        let shown = interp.string_value_text(metaclass)?;
        return Err(Raised::bad_metaclass(&shown).into());
    }
    Ok(metaclass)
}

/// The `class id` argument, and the traceback frame `NEW`'s own activation
/// contributes when it refuses.
fn class_id_argument(
    interp: &mut Interp,
    metaclass: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Vec<u8>, Failure> {
    let outcome = required_class_id(interp, args);
    if outcome.is_err() {
        interp.blame_native_method(
            b"NEW",
            "Class",
            metaclass,
            args.get(..1).unwrap_or_default(),
        );
    }
    outcome
}

/// [`class_id_argument`] without the frame, so that every way of failing
/// takes it.
fn required_class_id(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class id").into());
    };
    let argument = required_string_named_argument(interp, argument, "class id")?;
    Ok(interp.to_text(argument).to_vec())
}

/// The third argument: class-side methods the new class is built with, merged
/// into the class method dictionary.
fn enhance_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    enhancing: ObjRef,
) -> Result<(), Failure> {
    let pairs = method_table_pairs(interp, enhancing)?;
    // One frame around the whole walk, for the reason
    // [`Interp::define_method_table`] has one: each entry allocates the copy
    // `newScope` makes, and the temporary rooting that copy carries until
    // `hold_method_object` roots it as a global has to be released somewhere.
    let frame = interp.roots.activity_mut().push_frame();
    let installed = install_enhancing_class_methods(interp, class, &pairs);
    interp.roots.activity_mut().pop_frame(frame);
    installed
}

/// [`enhance_class_methods`]'s walk, split out so the root frame it runs
/// inside is released on the failure path too.
fn install_enhancing_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    pairs: &[(Box<[u8]>, ObjRef)],
) -> Result<(), Failure> {
    for (name, value) in pairs {
        let value = *value;
        let value = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            // For example, a literal array `('return 1', , 'nop')` in this
            // table is `93.952 Method argument method source is an array and
            // does not contain all string values.`
            compile_method_source(interp, name, value, "method source", SourceTaker::Method)?
        };
        interp
            .define_class_method_object(class, name, value)
            .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))?;
    }
    Ok(())
}

/// `Class~delete(name)`: take one instance method back off the receiver.
pub(super) fn native_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let name = method_name_argument(interp, args)?;
    interp
        .classes()
        .delete_instance_method(class, &String::from_utf8_lossy(&name));
    interp.drop_method_object(class, &name);
    interp.charge_retired_behaviours();
    Ok(None)
}

/// `Class~inherit(mixin, position)`: add a mixin to the receiver's superclass
/// list. The `INHERIT` keyword of a `::CLASS` directive reaches the same
/// function by sending this message ([`Interp::inherit_mixin`]).
pub(super) fn native_class_inherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    // **The position is not type-checked.** It is used in one place, to find
    // the position in the superclass list, and a value that is not in the list
    // gives index 0 and the "has not inherited" error whatever kind of object
    // it was. With `::class K` inheriting `.M1`, `.K~inherit(.M2, "abc")` is
    // `98.945`, `Class "The K class" has not inherited class "abc".` -- the
    // second argument's error and not the first's.
    let position = args.get(1).copied().flatten();
    match interp.classes().inherit_at(class, mixin, position) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// `Class~defineClassMethod(name, method)`: install `method` as a class-side
/// method of the receiver, writing the class behaviour and the class method
/// dictionary from one scope copy.
pub(super) fn native_define_class_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = method_name_argument(interp, args)?;
    let Some(Some(source)) = args.get(1).copied() else {
        return Err(Loud::setup_method("defineClassMethod with no method object").into());
    };
    if interp.receiver_kind(source) != Ok(Primitive::Method) {
        return Err(
            Loud::setup_method("defineClassMethod with a value that is not a method").into(),
        );
    }
    interp
        .define_class_method_object(class, &name, source)
        .ok_or_else(|| Failure::from(Loud::unrecorded_executable()))?;
    Ok(None)
}

/// `Class~inheritInstanceMethods(source)`: copy `source`'s own instance methods
/// into the receiver's dictionary at the receiver's scope, with no superclass
/// edge added -- the "phony inherit".
pub(super) fn native_inherit_instance_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Loud::setup_method("inheritInstanceMethods with no source class").into());
    };
    let Some(source) = interp.heap.is_class(argument).then_some(argument) else {
        return Err(
            Loud::setup_method("inheritInstanceMethods with a value that is not a class").into(),
        );
    };
    let source = class_receiver(interp, source)?;
    interp.classes().inherit_instance_methods(class, source);
    interp.classes().check_uninit(class);
    interp.flag_class_uninit(class);
    Ok(None)
}

/// `Class~uninherit(mixin)`: take a mixin back out of the receiver's
/// superclass list.
pub(super) fn native_uninherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    match interp.classes().uninherit(class, mixin) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// The `mixin class` argument `~inherit` and `~uninherit` share: required, and
/// a `MIXINCLASS` class object or 98.942 naming the value.
fn mixin_class_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("mixin class").into());
    };
    class_receiver(interp, argument).map_err(|_| {
        let shown = match interp.string_value_text(argument) {
            Ok(shown) => shown,
            Err(failure) => return failure,
        };
        Failure::from(Raised::inherit_needs_a_mixinclass(&shown))
    })
}

/// One [`InheritRefusal`] as the condition reported in its place, with the
/// substitutions rendered the way the message renders an object:
/// `stringValue()`, which is `~objectName` and so follows a rename.
fn inherit_refusal(
    interp: &mut Interp,
    class: ObjRef,
    mixin: ObjRef,
    refusal: InheritRefusal,
) -> Failure {
    let class_name = match interp.string_value_text(class) {
        Ok(class_name) => class_name,
        Err(failure) => return failure,
    };
    let mixin_name = match interp.string_value_text(mixin) {
        Ok(mixin_name) => mixin_name,
        Err(failure) => return failure,
    };
    match refusal {
        InheritRefusal::NotAMixin => Raised::inherit_needs_a_mixinclass(&mixin_name).into(),
        InheritRefusal::Recursive => Raised::recursive_inherit(&class_name, &mixin_name).into(),
        InheritRefusal::BaseClass(base) => {
            let base_name = match interp.string_value_text(base) {
                Ok(base_name) => base_name,
                Err(failure) => return failure,
            };
            Raised::inherit_base_class(&class_name, &mixin_name, &base_name).into()
        }
        InheritRefusal::NotInherited(other) => {
            let other_name = match interp.string_value_text(other) {
                Ok(other_name) => other_name,
                Err(failure) => return failure,
            };
            Raised::not_inherited(&class_name, &other_name).into()
        }
    }
}

/// `Package~addClass(name, class)` and `Package~addPublicClass(name, class)`,
/// which differ only in the public flag they hand on.
pub(super) fn native_package_add_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, false)
}

/// `Package~addPublicClass` -- see [`native_package_add_class`].
pub(super) fn native_package_add_public_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, true)
}

/// The body both `~addClass` rows share, since a second implementation is
/// where the two could come to disagree.
fn add_installed_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    public: bool,
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(name)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_vec();
    let Some(Some(class)) = args.get(1).copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    if !interp.heap.is_class(class) {
        return Err(Raised::argument_not_a_class("class").into());
    }
    if interp.package_data(receiver).is_none() {
        return Err(crate::Raised::receiver_kind("a value that is not a package").into());
    }
    if interp.is_the_rexx_package(receiver) {
        return Err(Raised::rexx_package_addition().into());
    }
    interp.add_installed_class(receiver, &name, class, public);
    Ok(Some(receiver))
}

/// `Class~package`: the package the class was defined in.
pub(super) fn native_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.package_object_for(class)?))
}

/// `Class~copy`: the refusal put over [`super::native_copy`]; its whole body is
/// the raise.
pub(super) fn native_class_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let target = interp.string_value_text(receiver)?;
    Err(Raised::copy_not_supported(&target).into())
}

/// `Class~enhanced(methods, ...)`: an instance of the receiver carrying
/// methods of its own: `NEW` sent to a subclass whose instance behaviour holds
/// the methods at scope `.nil`, and the object it answers then reporting the
/// receiver as its class.
pub(super) fn native_enhanced(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    // The refusals are different errors and are checked in this order:
    // `.K~enhanced` and `.K~enhanced()` are both `93.901 Not enough arguments
    // for method; 1 expected.` at rc 163, the second because a trailing
    // omission is dropped from the count, while `.K~enhanced(, 'x')` is `88.901
    // Missing argument; argument methods is required.` at rc 168.
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    let Some(table) = args[0] else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    let metaclass = interp.classes().metaclass(class);
    let dummy = new_subclass(
        interp,
        class,
        b"Enhanced Subclass",
        metaclass,
        ClassKind::Regular,
    )?;
    charge_class(interp, dummy);
    interp.record_packageless_class(dummy);
    finish_subclass(interp, dummy)?;
    let pairs = method_table_pairs(interp, table)?;
    let frame = interp.roots.activity_mut().push_frame();
    let installed = install_enhancing_methods(interp, dummy, &pairs);
    interp.roots.activity_mut().pop_frame(frame);
    installed?;
    interp.classes().check_uninit(dummy);
    let caller = interp.caller();
    let answer = interp
        .send_message(dummy, b"NEW", None, &args[1..], caller)?
        .ok_or_else(|| Failure::from(Raised::no_result(b"NEW")))?;
    interp.roots.activity_mut().push_temp(answer);
    mark_enhanced(interp, answer, dummy, class);
    Ok(Some(answer))
}

/// The owning class and the enhanced mark are set on what the dummy's `NEW`
/// answered, where the dummy built it. An answer built from anywhere else is
/// handed back untouched, where writing into it would change whatever behaviour
/// that object shares with every other of its class (Deviation 32).
fn mark_enhanced(interp: &mut Interp, answer: ObjRef, dummy: ObjRef, class: ObjRef) {
    if interp.heap.is_class(answer) {
        if interp.classes().class_of(answer) == dummy {
            interp.classes().set_owning_class(answer, class);
            interp.class_owns(answer, dummy);
        }
        return;
    }
    if let Some(Object {
        body: Body::Instance {
            class: held, own, ..
        },
        ..
    }) = interp.heap.get_mut(answer)
        && *held == dummy
    {
        *held = class;
        own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
            .mark_enhanced_by(dummy);
    }
}

/// Whether `Class~enhanced` built `value`, which
/// [`crate::environment::default_object_name`] is not given because the
/// answer takes no article.
pub(super) fn is_enhanced_instance(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.heap.get(value).map(|object| &object.body),
        Some(Body::Instance { own: Some(own), .. }) if own.is_enhanced_instance()
    )
}

/// [`native_enhanced`]'s methods into `dummy`'s own instance dictionary,
/// split out so the root frame it runs inside is released on the failure
/// path too, as in [`install_enhancing_class_methods`].
fn install_enhancing_methods(
    interp: &mut Interp,
    dummy: ObjRef,
    pairs: &[(Box<[u8]>, ObjRef)],
) -> Result<(), Failure> {
    let mut methods = Vec::with_capacity(pairs.len());
    let mut objects = Vec::with_capacity(pairs.len());
    for (name, value) in pairs {
        let value = *value;
        let name_text = String::from_utf8_lossy(name).to_ascii_uppercase();
        // A `.nil` item hides the name.
        if value == ObjRef::NIL {
            methods.push((name_text, None));
            continue;
        }
        let source = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            compile_method_source(interp, name, value, "method source", SourceTaker::Method)?
        };
        // `createMethodDictionary`'s `newMethodObject`, which takes a copy of
        // a method object some class already holds.
        let (executable, method) = scoped_method(interp, source, ObjRef::NIL)?;
        interp.roots.activity_mut().push_temp(executable);
        methods.push((name_text, Some(method)));
        objects.push((name.to_ascii_uppercase(), executable));
    }
    interp
        .classes()
        .enhance_instance_methods(dummy, ObjRef::NIL, &methods);
    for (name, executable) in objects {
        interp.hold_method_object(dummy, false, &name, executable);
    }
    Ok(())
}
