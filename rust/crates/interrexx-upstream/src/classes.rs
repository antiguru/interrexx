//! The primitive-class bootstrap table: which classes the interpreter
//! creates and in what order, and each class block's method-defining
//! operations.

use crate::header;

/// The bootstrap source's checklist names classes by their implementation
/// type; the table names them by their block, which is the class's Rexx
/// name. Hand-carried because no pattern in the source relates the two.
const CHECKLIST_TO_BLOCK: &[(&str, &str)] = &[
    ("RexxClass", "Class"),
    ("RexxInteger", "Integer"),
    ("RexxString", "String"),
    ("RexxObject", "Object"),
    ("PointerClass", "Pointer"),
    ("BufferClass", "Buffer"),
    ("ArrayClass", "Array"),
    ("TableClass", "Table"),
    ("IdentityTable", "IdentityTable"),
    ("RelationClass", "Relation"),
    ("StringTable", "StringTable"),
    ("DirectoryClass", "Directory"),
    ("SetClass", "Set"),
    ("BagClass", "Bag"),
    ("ListClass", "List"),
    ("QueueClass", "Queue"),
    ("NumberString", "NumberString"),
    ("MethodClass", "Method"),
    ("RoutineClass", "Routine"),
    ("PackageClass", "Package"),
    ("RexxContext", "RexxContext"),
    ("StemClass", "Stem"),
    ("SupplierClass", "Supplier"),
    ("MessageClass", "Message"),
    ("MutableBuffer", "MutableBuffer"),
    ("WeakReference", "WeakReference"),
    ("StackFrameClass", "StackFrame"),
    ("RexxInfo", "RexxInfo"),
    ("VariableReference", "VariableReference"),
    ("EventSemaphoreClass", "EventSemaphore"),
    ("MutexSemaphoreClass", "MutexSemaphore"),
];

/// How many arguments a native method accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arity {
    /// At most this many; fewer is allowed.
    Max(u32),
    /// A counted argument list: the table states no maximum.
    Any,
}

impl Arity {
    fn render(self) -> String {
        match self {
            Arity::Max(n) => n.to_string(),
            Arity::Any => "any".to_string(),
        }
    }

    fn from_text(text: &str) -> Option<Arity> {
        if text == "any" {
            return Some(Arity::Any);
        }
        text.parse().ok().map(Arity::Max)
    }
}

/// What one operation in a class block does. The spellings are the table's
/// own and match the generated Rust enum in `interrexx-classes`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    AddClassMethod,
    AddInstanceMethod,
    /// An instance method the ordinary private check refuses from a program
    /// context.
    AddPrivateInstanceMethod,
    /// An instance method the security manager is asked about.
    AddProtectedInstanceMethod,
    AddProtectedClassMethod,
    /// Copies another block's instance methods; the operand is that block.
    InheritInstanceMethods,
    RemoveInstanceMethod,
    HideInstanceMethod,
}

const KINDS: &[(Kind, &str)] = &[
    (Kind::AddClassMethod, "AddClassMethod"),
    (Kind::AddInstanceMethod, "AddInstanceMethod"),
    (Kind::AddPrivateInstanceMethod, "AddPrivateInstanceMethod"),
    (
        Kind::AddProtectedInstanceMethod,
        "AddProtectedInstanceMethod",
    ),
    (Kind::AddProtectedClassMethod, "AddProtectedClassMethod"),
    (Kind::InheritInstanceMethods, "InheritInstanceMethods"),
    (Kind::RemoveInstanceMethod, "RemoveInstanceMethod"),
    (Kind::HideInstanceMethod, "HideInstanceMethod"),
];

impl Kind {
    /// The spelling in the table.
    pub fn name(self) -> &'static str {
        KINDS
            .iter()
            .find(|(kind, _)| *kind == self)
            .expect("every kind is listed")
            .1
    }

    /// Whether the operation defines a method on the class side.
    pub fn is_class_side(self) -> bool {
        matches!(self, Kind::AddClassMethod | Kind::AddProtectedClassMethod)
    }

    /// Whether the operation adds a method, and so carries an arity.
    pub fn adds_method(self) -> bool {
        !matches!(
            self,
            Kind::InheritInstanceMethods | Kind::RemoveInstanceMethod | Kind::HideInstanceMethod
        )
    }
}

/// One operation of a class block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Op {
    pub kind: Kind,
    /// The method name, or the donor block for `InheritInstanceMethods`.
    pub name: String,
    /// Present for the operations that add a method.
    pub arity: Option<Arity>,
}

/// One class block, in the bootstrap's file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    /// Closed by the form that registers the class with the kernel only,
    /// never in the public environment.
    pub system_only: bool,
    pub ops: Vec<Op>,
}

/// The whole table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Table {
    /// Block names in the order the interpreter creates the classes.
    pub creation: Vec<String>,
    /// Blocks in file order, which is a dependency order: a donor's block
    /// precedes every block that inherits from it.
    pub blocks: Vec<Block>,
}

impl Table {
    /// Derives the table from the bootstrap source text.
    pub fn parse(src: &str) -> Table {
        let checklist = parse_checklist(src);
        assert!(
            checklist.len() >= 20,
            "only {} class creation calls parsed from the bootstrap source: the format changed",
            checklist.len()
        );
        let blocks = parse_blocks(src);
        assert!(
            blocks.len() >= 20,
            "only {} class blocks parsed from the bootstrap source: the format changed",
            blocks.len()
        );
        let creation: Vec<String> = checklist
            .iter()
            .map(|name| {
                CHECKLIST_TO_BLOCK
                    .iter()
                    .find(|(known, _)| known == name)
                    .unwrap_or_else(|| {
                        panic!("created class {name:?} has no entry in CHECKLIST_TO_BLOCK")
                    })
                    .1
                    .to_string()
            })
            .collect();
        for name in &creation {
            assert!(
                blocks.iter().any(|block| &block.name == name),
                "created class {name:?} has no class block"
            );
        }
        Table { creation, blocks }
    }

    /// The table as checked-in text, headed with the source revision.
    pub fn to_tsv(&self, sha: &str) -> String {
        let mut out = header(sha);
        out.push_str(
            "# creation<TAB>block: the interpreter creates the classes in this order.\n\
             # block<TAB>name<TAB>ordinary|system: a class block, in the bootstrap's file order.\n\
             # op<TAB>kind<TAB>name<TAB>arity: an operation of the preceding block.\n\
             # arity is a maximum argument count or `any`, and `-` where the kind adds no method.\n",
        );
        for name in &self.creation {
            out.push_str(&format!("creation\t{name}\n"));
        }
        for block in &self.blocks {
            let closing = if block.system_only {
                "system"
            } else {
                "ordinary"
            };
            out.push_str(&format!("block\t{}\t{closing}\n", block.name));
            for op in &block.ops {
                let arity = op.arity.map_or("-".to_string(), Arity::render);
                out.push_str(&format!("op\t{}\t{}\t{arity}\n", op.kind.name(), op.name));
            }
        }
        out
    }

    /// Reads a table back from its checked-in text.
    pub fn from_tsv(text: &str) -> Table {
        let mut creation = Vec::new();
        let mut blocks: Vec<Block> = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let at = || format!("classes table line {}: {line:?}", index + 1);
            let fields: Vec<&str> = line.split('\t').collect();
            match fields.as_slice() {
                ["creation", name] => creation.push((*name).to_string()),
                ["block", name, closing] => blocks.push(Block {
                    name: (*name).to_string(),
                    system_only: match *closing {
                        "system" => true,
                        "ordinary" => false,
                        _ => panic!("unknown closing form, {}", at()),
                    },
                    ops: Vec::new(),
                }),
                ["op", kind, name, arity] => {
                    let kind = KINDS
                        .iter()
                        .find(|(_, spelling)| spelling == kind)
                        .unwrap_or_else(|| panic!("unknown operation kind, {}", at()))
                        .0;
                    let arity = if *arity == "-" {
                        None
                    } else {
                        Some(
                            Arity::from_text(arity)
                                .unwrap_or_else(|| panic!("unknown arity, {}", at())),
                        )
                    };
                    assert_eq!(
                        arity.is_some(),
                        kind.adds_method(),
                        "arity mismatch, {}",
                        at()
                    );
                    blocks
                        .last_mut()
                        .unwrap_or_else(|| panic!("operation before any block, {}", at()))
                        .ops
                        .push(Op {
                            kind,
                            name: (*name).to_string(),
                            arity,
                        });
                }
                _ => panic!("malformed row, {}", at()),
            }
        }
        Table { creation, blocks }
    }
}

fn parse_checklist(src: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_suffix("::createInstance();") {
            names.push(rest.to_string());
        }
    }
    names
}

/// Extract the class blocks and, for each, the operation kind and method
/// name of every line, following the table of definition macros the source
/// declares. Lines this does not recognize (comments, braces, completion
/// calls, ...) are simply not operations.
fn parse_blocks(src: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut current: Option<Block> = None;

    for raw_line in src.lines() {
        let line = raw_line.trim();
        if let Some(name) = macro_arg(line, "StartClassDefinition") {
            assert!(
                current.is_none(),
                "nested StartClassDefinition({name}) before a matching End"
            );
            current = Some(Block {
                name,
                system_only: false,
                ops: Vec::new(),
            });
            continue;
        }
        // Which of the two closing forms ran is the whole of the
        // registration difference: the ordinary one puts the class into
        // `.environment` under its own name, and the special one puts it
        // into the kernel directory no `.NAME` reaches. Read off the source
        // rather than listed by hand, so a block that changes which form
        // closes it changes the registry with it.
        let closing = macro_arg(line, "EndClassDefinition")
            .map(|name| (name, false))
            .or_else(|| macro_arg(line, "EndSpecialClassDefinition").map(|name| (name, true)));
        if let Some((name, system_only)) = closing {
            let mut block = current
                .take()
                .unwrap_or_else(|| panic!("End...Definition({name}) with no matching Start"));
            assert_eq!(
                block.name, name,
                "StartClassDefinition({}) closed by a block named {name}",
                block.name
            );
            block.system_only = system_only;
            result.push(block);
            continue;
        }
        let Some(block) = current.as_mut() else {
            continue;
        };
        let op = add_method(line);
        if let Some((kind, (name, arity))) = op {
            block.ops.push(Op {
                kind,
                name,
                arity: Some(arity),
            });
        } else if let Some(name) = macro_arg(line, "InheritInstanceMethods") {
            block.ops.push(plain(Kind::InheritInstanceMethods, name));
        } else if let Some(name) = macro_string_arg(line, "RemoveMethod") {
            block.ops.push(plain(Kind::RemoveInstanceMethod, name.0));
        } else if let Some(name) = macro_string_arg(line, "HideMethod") {
            block.ops.push(plain(Kind::HideInstanceMethod, name.0));
        }
    }
    assert!(current.is_none(), "unterminated StartClassDefinition block");
    result
}

/// The method-adding macro on `line`, if any, with its operands.
fn add_method(line: &str) -> Option<(Kind, (String, Arity))> {
    let protected_class = method_operands(line, "AddClassProtectedMethod");
    if let Some(operands) = protected_class {
        return Some((Kind::AddProtectedClassMethod, operands));
    }
    let class = [
        "AddClassMethod",
        "AddClassPrivateMethod",
        "AddClassUnguardedMethod",
    ]
    .iter()
    .find_map(|name| method_operands(line, name));
    if let Some(operands) = class {
        return Some((Kind::AddClassMethod, operands));
    }
    if let Some(operands) = method_operands(line, "AddPrivateMethod") {
        return Some((Kind::AddPrivateInstanceMethod, operands));
    }
    if let Some(operands) = method_operands(line, "AddProtectedMethod") {
        return Some((Kind::AddProtectedInstanceMethod, operands));
    }
    ["AddUnguardedMethod", "AddMethod"]
        .iter()
        .find_map(|name| method_operands(line, name))
        .map(|operands| (Kind::AddInstanceMethod, operands))
}

fn plain(kind: Kind, name: String) -> Op {
    Op {
        kind,
        name,
        arity: None,
    }
}

/// `Macro(bareword)` -> `"bareword"`, for macros whose argument is an
/// identifier rather than a quoted string (the class-block openers and
/// closers and the instance-method donations).
fn macro_arg(line: &str, macro_name: &str) -> Option<String> {
    let rest = line.strip_prefix(macro_name)?;
    let rest = rest.strip_prefix('(')?;
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

/// `Macro("quoted", ...)` -> the unescaped content of the first quoted
/// argument and the text after its closing quote -- every method-adding,
/// removing or hiding invocation's method name. The only C escape the source
/// uses in these strings is a doubled backslash standing for one literal
/// backslash (`"\\="` is the two-character method name `\=`); this unescapes
/// exactly that and nothing else, since nothing else occurs here.
fn macro_string_arg<'a>(line: &'a str, macro_name: &str) -> Option<(String, &'a str)> {
    let rest = line.strip_prefix(macro_name)?;
    let rest = rest.strip_prefix('(')?;
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut result = String::new();
    let mut chars = rest.char_indices();
    loop {
        match chars.next()? {
            (end, '"') => return Some((result, &rest[end + 1..])),
            (_, '\\') => result.push(chars.next()?.1),
            (_, c) => result.push(c),
        }
    }
}

/// `Macro("name", entry, arity)` -> the name and the arity operand. The
/// entry operand names an implementation and is not part of the table.
fn method_operands(line: &str, macro_name: &str) -> Option<(String, Arity)> {
    let (name, after) = macro_string_arg(line, macro_name)?;
    let operands = after.strip_prefix(',');
    let close = operands.and_then(|rest| rest.rfind(')'));
    let (Some(operands), Some(close)) = (operands, close) else {
        panic!("{macro_name} without entry and arity operands in {line:?}");
    };
    let mut parts = operands[..close].splitn(2, ',');
    let _entry = parts.next();
    let Some(arity) = parts.next() else {
        panic!("{macro_name} without an arity operand in {line:?}");
    };
    let arity = arity.trim();
    let arity = if arity == "A_COUNT" {
        Arity::Any
    } else {
        Arity::Max(
            arity
                .parse()
                .unwrap_or_else(|_| panic!("unrecognised arity operand {arity:?} in {line:?}")),
        )
    };
    Some((name, arity))
}

#[cfg(test)]
mod tests;
