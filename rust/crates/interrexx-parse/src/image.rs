//! A parsed program as bytes, and back: what lets the interpreter's own
//! library, parsed when the interpreter is built, start without being parsed
//! again.
//!
//! An image holds everything a [`Program`] holds except its text, which the
//! reader is handed and checks against the image: the image is refused
//! unless it was written by this build's format from that same text. A
//! refused image is `None`, never a partial program, and the caller parses
//! the text instead.

use std::collections::BTreeMap;
use std::ops::Range;

use crate::Program;
use crate::ast::*;
use crate::selector::{Selector, SelectorTable};
use crate::source::{ProgramSource, SourceKind};
use crate::token::{Operator, SymbolId, SymbolTable};

/// What an image starts with.
const MAGIC: &[u8; 8] = b"RXPARSED";

/// The layout this module writes and reads. A change to what any type below
/// writes changes it.
const FORMAT: u32 = 2;

/// `program`, which was parsed from its own source text, as an image.
pub fn program_image(program: &Program) -> Vec<u8> {
    let mut out = Writer(Vec::new());
    out.0.extend_from_slice(MAGIC);
    out.0.extend_from_slice(&FORMAT.to_le_bytes());
    let (text, ..) = program.source.parts();
    out.0.extend_from_slice(&fingerprint(text).to_le_bytes());
    out.usize(text.len());
    program.source.put(&mut out);
    program.symbols.put(&mut out);
    program.main.put(&mut out);
    program.directives.put(&mut out);
    out.0
}

/// The program `image` holds, given the text it was parsed from, or `None`
/// where `image` is not this format's image of `text`.
pub fn program_from_image(image: &[u8], text: &[u8]) -> Option<Program> {
    read_program(image, None, text)
}

/// [`program_from_image`] for an image built into the interpreter together with
/// `text`, by the one build step that wrote both: each code body is left encoded
/// until it is first read (see [`Body`]), and only a debug build hashes `text` to
/// check the image is its image, which is a quarter of what an empty program's
/// startup spends reading the image. The format and the text's length are checked
/// either way.
pub fn program_from_static_image(image: &'static [u8], text: &[u8]) -> Option<Program> {
    read_program(image, Some(image), text)
}

fn read_program(image: &[u8], whole: Option<&'static [u8]>, text: &[u8]) -> Option<Program> {
    let mut input = Reader {
        bytes: image,
        whole,
        selectors: SelectorTable::new(),
        text: None,
    };
    let built_in = whole.is_some() && !cfg!(debug_assertions);
    if input.take(MAGIC.len())? != MAGIC
        || u32::from_le_bytes(input.take(4)?.try_into().ok()?) != FORMAT
    {
        return None;
    }
    let fingerprinted = u64::from_le_bytes(input.take(8)?.try_into().ok()?);
    if (!built_in && fingerprinted != fingerprint(text)) || input.usize()? != text.len() {
        return None;
    }
    input.text = Some(text.to_vec());
    let program = Program {
        source: Image::get(&mut input)?,
        symbols: Image::get(&mut input)?,
        main: Image::get(&mut input)?,
        directives: Image::get(&mut input)?,
    };
    input.bytes.is_empty().then_some(program)
}

/// The code body a [`Body`] left encoded holds.
pub(crate) fn decode_body(encoded: &[u8]) -> Option<CodeBody> {
    let mut input = Reader {
        bytes: encoded,
        whole: None,
        selectors: SelectorTable::new(),
        text: None,
    };
    let body = CodeBody::get(&mut input)?;
    input.bytes.is_empty().then_some(body)
}

/// A 64-bit FNV-1a hash of `text`, which ties an image to the text it was
/// parsed from.
fn fingerprint(text: &[u8]) -> u64 {
    text.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

struct Writer(Vec<u8>);

impl Writer {
    fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    /// LEB128.
    fn usize(&mut self, mut value: usize) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.0.push(byte);
                return;
            }
            self.0.push(byte | 0x80);
        }
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.usize(bytes.len());
        self.0.extend_from_slice(bytes);
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    /// The whole image, where it lives as long as the program: `bytes` is
    /// then its tail, and a body is left encoded as a slice of it.
    whole: Option<&'static [u8]>,
    /// Interns every selector read, as the parser does.
    selectors: SelectorTable,
    /// The source text, until the program's source takes it.
    text: Option<Vec<u8>>,
}

impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        if count > self.bytes.len() {
            return None;
        }
        let (taken, rest) = self.bytes.split_at(count);
        self.bytes = rest;
        Some(taken)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn usize(&mut self) -> Option<usize> {
        let mut value = 0usize;
        let mut shift = 0;
        loop {
            let byte = self.u8()?;
            if shift >= usize::BITS {
                return None;
            }
            value |= usize::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Some(value);
            }
            shift += 7;
        }
    }

    fn bytes(&mut self) -> Option<&'a [u8]> {
        let count = self.usize()?;
        self.take(count)
    }
}

/// A value an image holds.
trait Image: Sized {
    fn put(&self, out: &mut Writer);
    fn get(input: &mut Reader<'_>) -> Option<Self>;
}

impl Image for bool {
    fn put(&self, out: &mut Writer) {
        out.u8(u8::from(*self));
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        match input.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

impl Image for usize {
    fn put(&self, out: &mut Writer) {
        out.usize(*self);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        input.usize()
    }
}

impl Image for i64 {
    fn put(&self, out: &mut Writer) {
        out.0.extend_from_slice(&self.to_le_bytes());
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        Some(i64::from_le_bytes(input.take(8)?.try_into().ok()?))
    }
}

impl Image for Range<usize> {
    fn put(&self, out: &mut Writer) {
        out.usize(self.start);
        out.usize(self.end);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        Some(input.usize()?..input.usize()?)
    }
}

impl Image for Box<[u8]> {
    fn put(&self, out: &mut Writer) {
        out.bytes(self);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        Some(Box::from(input.bytes()?))
    }
}

impl<T: Image> Image for Option<T> {
    fn put(&self, out: &mut Writer) {
        match self {
            None => out.u8(0),
            Some(value) => {
                out.u8(1);
                value.put(out);
            }
        }
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        match input.u8()? {
            0 => Some(None),
            1 => Some(Some(T::get(input)?)),
            _ => None,
        }
    }
}

impl<T: Image> Image for Box<T> {
    fn put(&self, out: &mut Writer) {
        (**self).put(out);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        Some(Box::new(T::get(input)?))
    }
}

impl<T: Image> Image for Vec<T> {
    fn put(&self, out: &mut Writer) {
        out.usize(self.len());
        for item in self {
            item.put(out);
        }
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let count = input.usize()?;
        // Bounded by what is left, so a damaged count cannot ask for more
        // than the image could hold: every item takes a byte at least.
        if count > input.bytes.len() {
            return None;
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(T::get(input)?);
        }
        Some(items)
    }
}

impl Image for BTreeMap<Box<[u8]>, usize> {
    fn put(&self, out: &mut Writer) {
        out.usize(self.len());
        for (key, value) in self {
            key.put(out);
            value.put(out);
        }
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let count = input.usize()?;
        let mut map = BTreeMap::new();
        for _ in 0..count {
            let key = Image::get(input)?;
            map.insert(key, Image::get(input)?);
        }
        (map.len() == count).then_some(map)
    }
}

impl Image for SymbolId {
    fn put(&self, out: &mut Writer) {
        out.usize(self.index());
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        SymbolId::from_index(input.usize()?)
    }
}

impl Image for Selector {
    fn put(&self, out: &mut Writer) {
        out.bytes(self.bytes());
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let name = input.bytes()?;
        Some(input.selectors.intern(name))
    }
}

/// A body's image, behind its length, so a reader can leave it encoded.
impl Image for Body {
    fn put(&self, out: &mut Writer) {
        let mut body = Writer(Vec::new());
        CodeBody::put(self, &mut body);
        out.bytes(&body.0);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let encoded = input.bytes()?;
        match input.whole {
            Some(whole) => {
                let end = whole.len() - input.bytes.len();
                Some(Body::encoded(&whole[end - encoded.len()..end]))
            }
            None => Some(Body::new(decode_body(encoded)?)),
        }
    }
}

/// The names in id order; read back, each is interned again, which gives it
/// the id it had.
impl Image for SymbolTable {
    fn put(&self, out: &mut Writer) {
        out.usize(self.len());
        for index in 0..self.len() {
            let id = SymbolId::from_index(index).expect("an id the table issued");
            out.bytes(self.name(id).as_bytes());
        }
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let count = input.usize()?;
        let mut table = SymbolTable::default();
        for index in 0..count {
            let name = std::str::from_utf8(input.bytes()?).ok()?;
            if table.intern(name).index() != index {
                return None;
            }
        }
        Some(table)
    }
}

/// Everything but the text, which the reader holds.
impl Image for ProgramSource {
    fn put(&self, out: &mut Writer) {
        let (_, lines, kind, interpret_line, traceable, image_name) = self.parts();
        out.usize(lines.len());
        for &(start, end) in lines {
            out.usize(start);
            out.usize(end);
        }
        kind.put(out);
        out.usize(interpret_line);
        traceable.put(out);
        image_name.map(Box::<[u8]>::from).put(out);
    }

    fn get(input: &mut Reader<'_>) -> Option<Self> {
        let count = input.usize()?;
        if count > input.bytes.len() {
            return None;
        }
        let mut lines = Vec::with_capacity(count);
        for _ in 0..count {
            lines.push((input.usize()?, input.usize()?));
        }
        let kind = Image::get(input)?;
        let interpret_line = input.usize()?;
        let traceable = Image::get(input)?;
        let image_name = Image::get(input)?;
        let text = input.text.take()?;
        Some(ProgramSource::from_parts(
            text,
            lines,
            kind,
            interpret_line,
            traceable,
            image_name,
        ))
    }
}

/// A struct, field by field. The pattern names every field, so a field added
/// to the struct and not here does not compile.
macro_rules! image_struct {
    ($ty:ident { $($field:ident),* $(,)? }) => {
        impl Image for $ty {
            fn put(&self, out: &mut Writer) {
                let $ty { $($field),* } = self;
                $($field.put(out);)*
            }

            fn get(input: &mut Reader<'_>) -> Option<Self> {
                Some($ty { $($field: Image::get(input)?),* })
            }
        }
    };
}

/// An enum, as a tag byte and then the variant's fields. The `match` names
/// every variant, so a variant added to the enum and not here does not
/// compile.
macro_rules! image_enum {
    ($ty:ident {
        $($tag:literal => $variant:ident
            $(( $($item:ident),* ))?
            $({ $($field:ident),* })?
        ),* $(,)?
    }) => {
        impl Image for $ty {
            fn put(&self, out: &mut Writer) {
                match self {
                    $($ty::$variant $(( $($item),* ))? $({ $($field),* })? => {
                        out.u8($tag);
                        $($($item.put(out);)*)?
                        $($($field.put(out);)*)?
                    })*
                }
            }

            fn get(input: &mut Reader<'_>) -> Option<Self> {
                Some(match input.u8()? {
                    $($tag => $ty::$variant
                        $(( $({
                            let $item = Image::get(input)?;
                            $item
                        }),* ))?
                        $({ $($field: Image::get(input)?),* })?,)*
                    _ => return None,
                })
            }
        }
    };
}

image_enum!(SourceKind {
    0 => Program,
    1 => Interpret,
    2 => Lines,
});

image_enum!(Operator {
    0 => Plus,
    1 => Subtract,
    2 => Multiply,
    3 => Divide,
    4 => IntDiv,
    5 => Remainder,
    6 => Power,
    7 => Abuttal,
    8 => Concatenate,
    9 => Blank,
    10 => Equal,
    11 => BackslashEqual,
    12 => GreaterThan,
    13 => BackslashGreaterThan,
    14 => LessThan,
    15 => BackslashLessThan,
    16 => GreaterThanEqual,
    17 => LessThanEqual,
    18 => StrictEqual,
    19 => StrictBackslashEqual,
    20 => StrictGreaterThan,
    21 => StrictBackslashGreaterThan,
    22 => StrictLessThan,
    23 => StrictBackslashLessThan,
    24 => StrictGreaterThanEqual,
    25 => StrictLessThanEqual,
    26 => LessThanGreaterThan,
    27 => GreaterThanLessThan,
    28 => And,
    29 => Or,
    30 => Xor,
    31 => Backslash,
});

image_struct!(Expr { kind, span });

image_enum!(PrefixOp {
    0 => Plus,
    1 => Minus,
    2 => Not,
});

image_enum!(CallTarget {
    0 => Symbol(id),
    1 => Literal(bytes),
});

image_enum!(ExprKind {
    0 => Literal(bytes),
    1 => Constant(id),
    2 => Variable(id),
    3 => Stem(id),
    4 => Compound(id),
    5 => DotVariable(id),
    6 => Prefix { op, operand },
    7 => Binary { op, left, right },
    8 => Call { target, args },
    9 => QualifiedCall { namespace, name, args },
    10 => ClassResolver { namespace, name },
    11 => Message { target, name, super_class, args, cascade },
    12 => List(items),
    13 => Logical(items),
    14 => VariableReference(inner),
});

image_struct!(CodeBody {
    instructions,
    labels
});

image_enum!(EndStyle {
    0 => Do,
    1 => LabeledDo,
    2 => Loop,
    3 => Select,
    4 => Otherwise,
    5 => LabeledOtherwise,
});

image_struct!(EndTarget { block, style });

image_struct!(Instruction { kind, clause_span });

image_enum!(InstructionKind {
    0 => Assignment { target, value },
    1 => Label { name },
    2 => Message { term, value },
    3 => Command { expression },
    4 => Do(body),
    5 => Loop(body),
    6 => If { condition, false_target },
    7 => Then,
    8 => Else { then_exit },
    9 => Select { label, case, whens, otherwise, end },
    10 => When { condition, false_target, exit },
    11 => WhenCase { values, false_target, exit },
    12 => Otherwise,
    13 => Leave { name },
    14 => Iterate { name },
    15 => End { name, closes },
    16 => Drop { variables },
    17 => Expose { variables },
    18 => Parse(parse),
    19 => Arg(parse),
    20 => Pull(parse),
    21 => Push { expression },
    22 => Queue { expression },
    23 => Say { expression },
    24 => Call(call),
    25 => Return { expression },
    26 => Procedure { variables },
    27 => Signal(signal),
    28 => Exit { expression },
    29 => Interpret { expression },
    30 => Guard(guard),
    31 => Reply { expression },
    32 => Forward(forward),
    33 => Raise(raise),
    34 => Use(use_),
    35 => Numeric { setting, expression },
    36 => Address(address),
    37 => Trace(trace),
    38 => Options { expression },
    39 => Nop,
});

image_enum!(VariableRef {
    0 => Direct(id),
    1 => Indirect(id),
});

image_struct!(Loop {
    label,
    counter,
    kind,
    conditional,
    end
});

image_enum!(LoopKind {
    0 => Simple,
    1 => Forever,
    2 => Count(count),
    3 => Controlled(controlled),
    4 => Over { control, target, for_count },
    5 => With { index, item, target, for_count },
});

image_struct!(Controlled {
    control,
    initial,
    to,
    by,
    for_count,
    order
});

image_enum!(ControlExpr {
    0 => To,
    1 => By,
    2 => For,
});

image_struct!(LoopConditional { until, condition });

image_struct!(Parse {
    source,
    upper,
    lower,
    caseless,
    template
});

image_enum!(ParseSource {
    0 => Arg,
    1 => LineIn,
    2 => Pull,
    3 => Source,
    4 => Version,
    5 => Var(id),
    6 => Value(value),
});

image_struct!(ParseTrigger {
    kind,
    value,
    targets
});

image_enum!(TriggerKind {
    0 => End,
    1 => Plus,
    2 => Minus,
    3 => Absolute,
    4 => MinusLength,
    5 => PlusLength,
    6 => String,
    7 => Mixed,
});

image_enum!(Call {
    0 => Named { name, literal, args },
    1 => Dynamic { target, args },
    2 => Qualified { namespace, name, args },
    3 => Trap(trap),
});

image_enum!(Signal {
    0 => Label(name),
    1 => Value(value),
    2 => Trap(trap),
});

image_struct!(ConditionTrap {
    on,
    condition,
    label
});

image_struct!(Guard {
    on,
    condition,
    watched
});

image_struct!(Forward {
    to,
    message,
    class,
    arguments,
    array,
    continue_
});

image_struct!(Raise {
    condition,
    propagate,
    rc,
    description,
    additional,
    array,
    result
});

image_struct!(RaiseResult { exit, value });

image_enum!(Use {
    0 => Arg { strict, allow_optionals, targets },
    1 => Local { variables },
});

image_struct!(UseTarget {
    target,
    default,
    alias
});

image_struct!(Address {
    environment,
    dynamic,
    command,
    io
});

image_struct!(AddressIo {
    input,
    output,
    error,
    output_option,
    error_option
});

image_enum!(Redirection {
    0 => Default,
    1 => Normal,
    2 => Stem(id),
    3 => Stream(expression),
    4 => Using(expression),
});

image_enum!(OutputOption {
    0 => Default,
    1 => Replace,
    2 => Append,
});

image_enum!(NumericSetting {
    0 => Digits,
    1 => Fuzz,
    2 => FormDefault,
    3 => FormScientific,
    4 => FormEngineering,
    5 => FormValue,
});

image_enum!(Trace {
    0 => Default,
    1 => Setting(setting),
    2 => Skip(count),
    3 => Value(value),
});

image_struct!(Directive { kind, clause_span });

image_enum!(DirectiveKind {
    0 => Annotate(annotate),
    1 => Attribute(attribute),
    2 => Class(class),
    3 => Constant(constant),
    4 => Method(method),
    5 => Options(options),
    6 => Requires(requires),
    7 => Resource(resource),
    8 => Routine(routine),
});

image_enum!(Access {
    0 => Default,
    1 => Private,
    2 => Public,
    3 => Package,
});

image_enum!(Protection {
    0 => Default,
    1 => Protected,
    2 => Unprotected,
});

image_enum!(GuardOption {
    0 => Default,
    1 => Guarded,
    2 => Unguarded,
});

image_struct!(ExternalSpec {
    registered,
    library,
    entry
});

image_struct!(ClassDirective {
    name,
    access,
    abstract_,
    subclass,
    mixin,
    metaclass,
    inherit
});

image_struct!(ClassRef { namespace, name });

image_struct!(MethodDirective {
    name,
    class_method,
    attribute,
    abstract_,
    access,
    protection,
    guard,
    external,
    delegate,
    body
});

image_struct!(AttributeDirective {
    name,
    style,
    class_method,
    abstract_,
    access,
    protection,
    guard,
    external,
    delegate,
    body
});

image_enum!(AttributeStyle {
    0 => Both,
    1 => Get,
    2 => Set,
});

image_struct!(ConstantDirective { name, value });

image_enum!(ConstantValue {
    0 => Name,
    1 => Text(text),
    2 => Expression(expression),
});

image_struct!(Annotate {
    target,
    annotations
});

image_enum!(AnnotationTarget {
    0 => Package,
    1 => Class(name),
    2 => Routine(name),
    3 => Method(name),
    4 => Attribute(name),
    5 => Constant(name),
});

image_struct!(Annotation { name, value });

image_enum!(PackageOption {
    0 => Digits(digits),
    1 => Fuzz(fuzz),
    2 => Form(form),
    3 => Trace(setting),
    4 => Condition { which, syntax },
    5 => Prolog(on),
    6 => NumericInherit(on),
});

image_enum!(OptionsForm {
    0 => Scientific,
    1 => Engineering,
});

image_enum!(ConditionOption {
    0 => All,
    1 => Error,
    2 => Failure,
    3 => LostDigits,
    4 => NoString,
    5 => NotReady,
    6 => NoValue,
});

image_struct!(Requires {
    name,
    library,
    namespace
});

image_struct!(Resource {
    name,
    end_marker,
    lines
});

image_struct!(RoutineDirective {
    name,
    access,
    external,
    body
});

#[cfg(test)]
mod tests;
