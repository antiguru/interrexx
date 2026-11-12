//! The golden shape of the interpreter's own library, its provenance, the
//! messages Rust sends to library classes, and the leak guards.
//!
//! Three committed files back these tests, next to `Cargo.toml`:
//!
//! * `library-shape.txt`: one row per class and per method of each class
//!   scope, with the class and method flags, then `.environment`'s and
//!   `.local`'s indexes in order. The replacement of a host class by one of
//!   interrexx's own must leave it unchanged, except for the rows of members
//!   that `build.rs` lists as dropped.
//! * `library-origin.txt`: for the same rows, where each method comes from:
//!   `native` (a Rust method table), `entry:<name>` (a `LIBRARY REXX` entry
//!   point), `host:<program>` (a directive of an embedded program),
//!   `replacement:<file>` (a directive of interrexx's own source, inside a
//!   splice range), or `same-as:<row>` (the method object is the same one an
//!   earlier row holds, which is what a donation leaves).
//! * `library-sends.txt`: every message the Rust side sends in a file that
//!   reaches a library class by name, so that a new send shows in review.
//!
//! **Regenerating** is local: run
//! `REXX_UPDATE_GOLDEN=1 cargo nextest run -p interrexx library_shape`
//! and read the diff of the three files. On a mismatch without that variable
//! each test prints the whole new table.
//!
//! Every row is names and flags. No library source text is read or printed.

use crate::{ExecutableSource, Interp, ObjRef};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn manifest(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(file)
}

/// Compares `table` with the committed `file`, or rewrites the file under
/// `REXX_UPDATE_GOLDEN`.
fn check_golden(file: &str, table: &str) {
    let path = manifest(file);
    if std::env::var_os("REXX_UPDATE_GOLDEN").is_some_and(|value| !value.is_empty()) {
        std::fs::write(&path, table).unwrap_or_else(|e| panic!("writing {file}: {e}"));
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == table,
        "{file} does not match the live interpreter. Regenerate it locally with \
         `REXX_UPDATE_GOLDEN=1 cargo nextest run -p interrexx library_shape` and review the \
         diff. The table now is:\n{table}"
    );
}

/// One bootstrapped interpreter and the objects this module holds across its
/// sends, all rooted for as long as the module runs.
struct Probe {
    interp: Interp,
}

impl Probe {
    fn new() -> Self {
        let mut interp = Interp::new();
        interp
            .bootstrap_library()
            .expect("the library bootstrap runs");
        Probe { interp }
    }

    fn send(&mut self, to: ObjRef, name: &str, args: &[ObjRef]) -> ObjRef {
        let args: Vec<Option<ObjRef>> = args.iter().map(|a| Some(*a)).collect();
        let caller = self.interp.caller();
        let answered = self
            .interp
            .send_message(to, name.as_bytes(), None, &args, caller)
            .unwrap_or_else(|failure| panic!("{name} failed: {failure:?}"))
            .unwrap_or_else(|| panic!("{name} answered nothing"));
        self.interp.roots.activity_mut().push_temp(answered);
        answered
    }

    fn text(&mut self, value: ObjRef) -> String {
        String::from_utf8_lossy(&self.interp.to_text(value)).into_owned()
    }

    fn texts(&mut self, name: &str) -> Vec<String> {
        let directory = self
            .interp
            .dot_variable(name.as_bytes())
            .expect("a directory");
        let indexes = self.send(directory, "ALLINDEXES", &[]);
        let slots = self.interp.array_slots_of(indexes).expect("an array");
        slots
            .into_iter()
            .flatten()
            .map(|index| self.text(index))
            .collect()
    }

    fn array(&self, value: ObjRef) -> Vec<ObjRef> {
        self.interp
            .array_slots_of(value)
            .expect("an array")
            .into_iter()
            .flatten()
            .collect()
    }

    fn class_name(&mut self, class: ObjRef) -> String {
        let id = self.send(class, "ID", &[]);
        self.text(id).to_ascii_uppercase()
    }

    fn flag(&mut self, method: ObjRef, name: &str) -> u8 {
        let answer = self.send(method, name, &[]);
        u8::from(self.text(answer) != "0")
    }

    /// Every class the shape covers, by upper-case name: the REXX package's
    /// own table and every library program's table (which can disagree on a
    /// name), then each class `.environment` holds.
    fn classes(&mut self) -> BTreeMap<String, ObjRef> {
        let mut found: BTreeMap<String, ObjRef> = BTreeMap::new();
        let rexx = self.interp.rexx_package_object();
        let mut tables: Vec<Vec<(String, ObjRef)>> = Vec::new();
        let table = |data: &crate::package_state::PackageData| {
            data.classes
                .iter()
                .map(|(name, class)| (String::from_utf8_lossy(name).into_owned(), *class))
                .collect::<Vec<_>>()
        };
        if let Some(data) = self.interp.package_data(rexx) {
            tables.push(table(data));
        }
        for program in self.interp.library_programs.clone() {
            if let Some(data) = self.interp.program_data(program) {
                tables.push(table(data));
            }
        }
        for (name, class) in tables.into_iter().flatten() {
            found.entry(name).or_insert(class);
        }
        for name in self.texts(".ENVIRONMENT") {
            let Ok(value) = self.interp.dot_variable(format!(".{name}").as_bytes()) else {
                continue;
            };
            if value != ObjRef::NIL && self.interp.heap.is_class(value) {
                found.entry(name.to_ascii_uppercase()).or_insert(value);
            }
        }
        found
    }

    /// `1` for a class the REXX package holds publicly, `0` for one it holds
    /// without `PUBLIC`, and `native` for one only `.environment` holds.
    fn is_public(&mut self, name: &str) -> &'static str {
        let rexx = self.interp.rexx_package_object();
        let Some(data) = self.interp.package_data(rexx) else {
            return "native";
        };
        if data.public_classes.contains_key(name.as_bytes()) {
            "1"
        } else if data.classes.contains_key(name.as_bytes()) {
            "0"
        } else {
            "native"
        }
    }

    /// Where `method` comes from, or `None` for a primitive.
    fn origin(&self, method: ObjRef) -> String {
        let Some(record) = self.interp.executable_sources.get(&method).copied() else {
            return "native".into();
        };
        let Some(installed) = record.installed else {
            return "native".into();
        };
        match self.interp.installed_executable_source(installed) {
            ExecutableSource::Directive { program, directive }
            | ExecutableSource::ConstantExpression { program, directive } => {
                let Some(name) = self.interp.library_program_names.get(&program).copied() else {
                    return "outside-library".into();
                };
                let start = self
                    .interp
                    .programs
                    .get(program)
                    .and_then(|parsed| parsed.directives.get(directive))
                    .map(|directive| directive.clause_span.start);
                match start.and_then(|at| spliced_file(interrexx_lib::SPLICES, name, at)) {
                    Some(file) => format!("replacement:{file}"),
                    None => format!(
                        "host:{}",
                        name.rsplit_once('.').map_or(name, |(stem, _)| stem)
                    ),
                }
            }
            ExecutableSource::External { .. } => {
                match self.interp.native_externals.get(&installed) {
                    Some(entry) => format!("entry:{}", entry.name),
                    None => "entry:unbound".into(),
                }
            }
            ExecutableSource::Native => "native".into(),
            other => format!("other:{other:?}"),
        }
    }

    /// The class whose `::CLASS` directive holds the directive `method` came
    /// from: the one a donation took it from.
    fn declaring_class(&self, method: ObjRef) -> Option<String> {
        let installed = self.interp.executable_sources.get(&method)?.installed?;
        let (ExecutableSource::Directive { program, directive }
        | ExecutableSource::ConstantExpression { program, directive }) =
            self.interp.installed_executable_source(installed)
        else {
            return None;
        };
        let parsed = self.interp.programs.get(program)?;
        parsed.directives[..directive.min(parsed.directives.len())]
            .iter()
            .rev()
            .find_map(|d| match &d.kind {
                interrexx_parse::DirectiveKind::Class(class) => {
                    Some(String::from_utf8_lossy(&class.name).to_ascii_uppercase())
                }
                _ => None,
            })
    }

    fn entry_name(&self, method: ObjRef) -> String {
        let installed = self
            .interp
            .executable_sources
            .get(&method)
            .and_then(|record| record.installed);
        installed
            .and_then(|id| self.interp.native_externals.get(&id))
            .map_or_else(|| "-".to_string(), |entry| entry.name.to_string())
    }
}

/// The replacement file whose splice range in `program` holds byte `at`, for a
/// directive whose clause starts there.
fn spliced_file<'a>(
    splices: &'a [interrexx_lib::SpliceRange],
    program: &str,
    at: usize,
) -> Option<&'a str> {
    splices
        .iter()
        .find(|s| s.program == program && (s.start..s.end).contains(&at))
        .map(|s| s.file)
}

/// A method is a replacement's exactly when its clause starts inside a splice
/// range of its own program.
#[test]
fn a_directive_inside_a_splice_range_is_a_replacements() {
    let splices = [interrexx_lib::SpliceRange {
        program: "A.orx",
        class: "C",
        file: "c.rex",
        start: 10,
        end: 20,
        first_line: 1,
    }];
    assert_eq!(spliced_file(&splices, "A.orx", 10), Some("c.rex"));
    assert_eq!(spliced_file(&splices, "A.orx", 19), Some("c.rex"));
    assert_eq!(spliced_file(&splices, "A.orx", 20), None);
    assert_eq!(spliced_file(&splices, "A.orx", 9), None);
    assert_eq!(spliced_file(&splices, "B.orx", 15), None);
}

/// Both tables, built from one interpreter.
fn tables() -> (String, String) {
    let mut probe = Probe::new();
    let frame = probe.interp.roots.activity_mut().push_frame();
    let classes = probe.classes();

    // Identity: every reference to a name that this table holds is one object.
    let mut references: BTreeMap<String, std::collections::HashSet<ObjRef>> = BTreeMap::new();
    for (name, class) in &classes {
        references.entry(name.clone()).or_default().insert(*class);
    }
    let mut superclasses: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut metaclasses: BTreeMap<String, String> = BTreeMap::new();
    for (name, class) in &classes {
        let supers = probe.send(*class, "SUPERCLASSES", &[]);
        let mut names = Vec::new();
        for sup in probe.array(supers) {
            let sup_name = probe.class_name(sup);
            references.entry(sup_name.clone()).or_default().insert(sup);
            names.push(sup_name);
        }
        superclasses.insert(name.clone(), names);
        let meta = probe.send(*class, "METACLASS", &[]);
        let meta_name = probe.class_name(meta);
        references
            .entry(meta_name.clone())
            .or_default()
            .insert(meta);
        metaclasses.insert(name.clone(), meta_name);
    }
    for program in probe.interp.library_programs.clone() {
        if let Some(data) = probe.interp.program_data(program) {
            for (name, class) in data.classes.iter() {
                references
                    .entry(String::from_utf8_lossy(name).into_owned())
                    .or_default()
                    .insert(*class);
            }
        }
    }
    let environment: BTreeSet<String> = probe.texts(".ENVIRONMENT").into_iter().collect();
    for name in &environment {
        if let Ok(value) = probe.interp.dot_variable(format!(".{name}").as_bytes())
            && probe.interp.heap.is_class(value)
        {
            references
                .entry(name.to_ascii_uppercase())
                .or_default()
                .insert(value);
        }
    }

    let mut shape = String::from(
        "# The interrexx library shape. Generated: see library_shape.rs for how to regenerate.\n",
    );
    let mut origin =
        String::from("# Where each library method comes from. Generated: see library_shape.rs.\n");
    let mut first_row: std::collections::HashMap<interrexx_core::MethodId, (String, u8)> =
        std::collections::HashMap::new();
    let mut rows: Vec<(String, Option<interrexx_core::MethodId>, String)> = Vec::new();
    for (name, class) in &classes {
        let mixin = u8::from(probe.interp.classes().is_mixin(*class));
        let abstract_ = u8::from(probe.interp.classes().is_abstract(*class));
        let public = probe.is_public(name);
        let identity = u8::from(references[name].len() == 1);
        let annotations = probe.send(*class, "ANNOTATIONS", &[]);
        let indexes = probe.send(annotations, "ALLINDEXES", &[]);
        let mut notes = Vec::new();
        for index in probe.array(indexes) {
            let key = probe.text(index);
            let value = probe.send(annotations, "AT", &[index]);
            notes.push(format!("{key}={}", probe.text(value)));
        }
        notes.sort();
        let supers = superclasses[name].join(",");
        let notes = if notes.is_empty() {
            "-".to_string()
        } else {
            notes.join(",")
        };
        let _ = writeln!(
            shape,
            "class {name} public={public} mixin={mixin} abstract={abstract_} metaclass={} \
             supers={} annotations={notes} identity={identity}",
            metaclasses[name],
            if supers.is_empty() { "-" } else { &supers },
        );
        for (side, label) in [(false, "instance"), (true, "class")] {
            let names = if side {
                probe.interp.classes().own_class_method_names(*class)
            } else {
                probe.interp.classes().own_instance_method_names(*class)
            };
            for method_name in names {
                let found = if side {
                    probe
                        .interp
                        .classes()
                        .lookup_class_method(*class, &method_name)
                        .filter(|(scope, _)| scope == class)
                } else {
                    match probe
                        .interp
                        .classes()
                        .own_instance_slot(*class, &method_name)
                    {
                        Some(interrexx_classes::MethodSlot::Defined { scope, method }) => {
                            Some((scope, method))
                        }
                        _ => None,
                    }
                };
                let Some((scope, installed)) = found else {
                    let _ = writeln!(shape, "  {label} {method_name} unreachable");
                    rows.push((
                        format!("{name}.{label}.{method_name}"),
                        None,
                        "unreachable".into(),
                    ));
                    continue;
                };
                let record = crate::ExecutableRecord {
                    source: probe.interp.installed_executable_source(installed),
                    installed: Some(installed),
                    routine: None,
                    owns_installed: false,
                };
                let method = probe
                    .interp
                    .method_object(*class, side, method_name.as_bytes(), scope, record)
                    .expect("a Method object");
                probe.interp.roots.activity_mut().push_temp(method);
                let flags = [
                    ("private", "ISPRIVATE"),
                    ("protected", "ISPROTECTED"),
                    ("guarded", "ISGUARDED"),
                    ("abstract", "ISABSTRACT"),
                    ("constant", "ISCONSTANT"),
                    ("attribute", "ISATTRIBUTE"),
                    ("package", "ISPACKAGE"),
                ];
                let mut row = format!("  {label} {method_name}");
                for (text, message) in flags {
                    let value = probe.flag(method, message);
                    let _ = write!(row, " {text}={value}");
                }
                let entry = probe.entry_name(method);
                let _ = writeln!(shape, "{row} entry={entry}");
                let id = probe
                    .interp
                    .executable_sources
                    .get(&method)
                    .and_then(|record| record.installed);
                let key = format!("{name}.{label}.{method_name}");
                // The row a shared method object is attributed to: the class whose
                // directive declares it, then a mixin, then a class that holds it
                // at its own scope.
                let lexical = probe.declaring_class(method).is_some_and(|c| c == *name);
                let owned = u8::from(lexical) * 4 + mixin * 2 + u8::from(scope == *class);
                let where_from = probe.origin(method);
                if let Some(id) = id {
                    let canonical = first_row.entry(id).or_insert_with(|| (key.clone(), owned));
                    if canonical.1 < owned {
                        *canonical = (key.clone(), owned);
                    }
                }
                rows.push((key, id, where_from));
            }
        }
    }
    for (key, id, where_from) in rows {
        let shared = id
            .and_then(|id| first_row.get(&id))
            .filter(|(canonical, _)| *canonical != key);
        match shared {
            Some((canonical, _)) => {
                let _ = writeln!(origin, "{key} same-as:{canonical}");
            }
            None => {
                let _ = writeln!(origin, "{key} {where_from}");
            }
        }
    }
    let _ = writeln!(
        shape,
        "environment: {}",
        probe.texts(".ENVIRONMENT").join(" ")
    );
    let _ = writeln!(shape, "local: {}", probe.texts(".LOCAL").join(" "));
    probe.interp.roots.activity_mut().pop_frame(frame);
    (shape, origin)
}

/// The library's classes, methods and flags, and where each method comes
/// from, are what the two committed files say.
#[test]
fn library_shape_and_origin_match_the_goldens() {
    let (shape, origin) = tables();
    check_golden("library-shape.txt", &shape);
    check_golden("library-origin.txt", &origin);
}

/// The REXX package holds each class once, and `.environment`, the package
/// table and each program's table name the same object.
#[test]
fn library_shape_is_not_empty() {
    let (shape, origin) = tables();
    assert!(shape.lines().filter(|l| l.starts_with("class ")).count() > 40);
    assert!(origin.lines().count() > 500);
    assert!(shape.contains("environment: "));
    assert!(origin.contains(" host:CoreClasses"));
}

/// Files that reach a library class by name, and the messages they send.
fn rust_sends() -> String {
    let root = manifest("src");
    let mut files = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("src is readable").flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "rs")
                && path.file_name().is_some_and(|n| n != "tests.rs")
                && path.file_name().is_some_and(|n| n != "library_shape.rs")
            {
                files.push(path);
            }
        }
    }
    files.sort();
    let lookups = ["rexx_package_class(", "dot_variable(b\".", "EnvScope::"];
    let mut rows = BTreeSet::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("a source file");
        let live = text
            .split_once("#[cfg(test)]\nmod tests {")
            .map_or(text.as_str(), |(live, _)| live);
        if !lookups.iter().any(|l| live.contains(l)) {
            continue;
        }
        let name = path
            .strip_prefix(&root)
            .expect("under src")
            .to_string_lossy()
            .into_owned();
        let mut function = String::from("-");
        let mut at = 0;
        while at < live.len() {
            let line_end = live[at..].find('\n').map_or(live.len(), |n| at + n);
            let line = live[at..line_end].trim_start();
            if let Some(rest) = line
                .strip_prefix("pub(crate) fn ")
                .or_else(|| line.strip_prefix("pub(super) fn "))
                .or_else(|| line.strip_prefix("pub fn "))
                .or_else(|| line.strip_prefix("fn "))
            {
                function = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
            }
            for call in ["send_message(", "send_message_as_given("] {
                let mut from = at;
                while let Some(found) = live[from..line_end.max(from)].find(call) {
                    let start = from + found + call.len();
                    rows.insert(format!("{name} {function} {}", message_of(&live[start..])));
                    from = start;
                }
            }
            at = line_end + 1;
        }
    }
    let mut out = String::from(
        "# Messages sent from Rust in files that reach a library class by name. Generated: see \
         library_shape.rs.\n",
    );
    for row in rows {
        out.push_str(&row);
        out.push('\n');
    }
    out
}

/// The second argument of the call whose text starts at `after`, spaces
/// squashed.
fn message_of(after: &str) -> String {
    let mut depth = 0usize;
    let mut commas = 0;
    let mut token = String::new();
    for c in after.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' if depth == 0 => break,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                commas += 1;
                if commas == 2 {
                    break;
                }
                continue;
            }
            _ => {}
        }
        if commas == 1 && !c.is_whitespace() {
            token.push(c);
        }
    }
    let token = token.trim_start_matches('&');
    if token.is_empty() {
        "<unread>".into()
    } else {
        token.to_string()
    }
}

/// A Rust send to a library class shows up in the committed list, so a new one
/// is reviewed against the contract the library keeps for it.
#[test]
fn library_sends_match_the_golden() {
    check_golden("library-sends.txt", &rust_sends());
}

/// Trimmed lines of the embedded programs long enough to be distinctive.
fn library_source_lines() -> BTreeSet<String> {
    let mut lines = BTreeSet::new();
    for program in interrexx_lib::PROGRAMS {
        for line in String::from_utf8_lossy(program.source).lines() {
            let line = line.trim();
            if line.len() >= 14 {
                lines.insert(line.to_string());
            }
        }
    }
    lines
}

/// The library source lines that `output` contains.
fn leaked(output: &str, user_program: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for (n, line) in library_source_lines().iter().enumerate() {
        if !user_program.contains(line.as_str()) && output.contains(line.as_str()) {
            found.push(n);
        }
    }
    found
}

/// A library frame names its method and scope and says no source is
/// available, whether the error is caught or reaches the top level, and
/// `Method~source` of a library method holds no lines. No library source text
/// appears in anything a program can read about a failure inside the library.
#[test]
fn a_failure_inside_the_library_renders_no_library_source() {
    let caught = "\
say 'S>' .bag~method('PUTALL')~source~items
say 'S>' .timespan~method('DAYS')~source~items
signal on syntax name trap
x = .timespan~fromDays('a')
exit
trap:
c = condition('O')
do f over c~stackFrames
  say 'F>' f~traceLine
end
do l over c~traceback; say 'TB>' l; end
say 'D>' c~message c~errorText c~position c~program
";
    let uncaught = "x = .timespan~fromDays('a')\n";
    let mut output = String::new();
    for program in [caught, uncaught] {
        let outcome = crate::run_program(
            "/w/leak.rex",
            program.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        output.push_str(&String::from_utf8_lossy(&outcome.stdout));
        output.push_str(&String::from_utf8_lossy(&outcome.stderr));
    }
    // The run is a failure that crossed library frames, or the check below
    // has nothing to read.
    assert!(
        output.contains("(no source available)"),
        "no library frame was rendered:\n{output}"
    );
    assert!(
        output.contains("S> 0"),
        "a library method held source:\n{output}"
    );
    let user = format!("{caught}{uncaught}");
    assert!(
        leaked(&output, &user).is_empty(),
        "library source text reached the output:\n{output}"
    );
}

/// The check above sees a library line when one is there.
#[test]
fn the_leak_check_finds_a_planted_library_line() {
    let lines = library_source_lines();
    assert!(lines.len() > 500, "the library source set is empty");
    let planted = lines.iter().next().expect("a line").clone();
    assert!(!leaked(&format!("x\n{planted}\n"), "").is_empty());
    assert!(leaked("nothing of the library here", "").is_empty());
}

/// The first-run report: which embedded program declares which class, the
/// access of every host member, whether the spec page of its class names it,
/// each program's `::OPTIONS`, and how native classes' methods are provided.
/// Names and flags only. Run with
/// `cargo nextest run -p interrexx first_run_report --run-ignored only --no-capture`.
#[test]
#[ignore = "prints a report"]
fn first_run_report() {
    use interrexx_parse::{DirectiveKind, Program};
    let probe = Probe::new();
    let spec = manifest("../../../spec/src/classes");
    let mut page_cache: BTreeMap<String, String> = BTreeMap::new();
    let mut with_methods: BTreeSet<String> = BTreeSet::new();
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    for program in probe.interp.library_programs.clone() {
        let host = probe.interp.library_program_names[&program].to_string();
        let parsed: &Program = probe.interp.programs.get(program).expect("a program");
        let mut options = Vec::new();
        let mut class = String::new();
        for directive in &parsed.directives {
            match &directive.kind {
                DirectiveKind::Options(list) => options.push(format!("{list:?}")),
                DirectiveKind::Class(c) => {
                    class = String::from_utf8_lossy(&c.name).to_ascii_uppercase();
                    declared.insert(class.clone(), host.clone());
                }
                DirectiveKind::Method(m) => {
                    let flags = format!(
                        "{:?}/{:?}/{:?}{}{}",
                        m.access,
                        m.protection,
                        m.guard,
                        if m.class_method { "/classside" } else { "" },
                        if m.external.is_some() {
                            "/external"
                        } else {
                            ""
                        },
                    );
                    member(&mut page_cache, &spec, &host, &class, &m.name, &flags);
                    with_methods.insert(class.clone());
                }
                DirectiveKind::Attribute(a) => {
                    let flags = format!(
                        "{:?}/{:?}/{:?}/attribute{}",
                        a.access,
                        a.protection,
                        a.guard,
                        if a.class_method { "/classside" } else { "" },
                    );
                    member(&mut page_cache, &spec, &host, &class, &a.name, &flags);
                    with_methods.insert(class.clone());
                }
                DirectiveKind::Constant(k) => {
                    member(&mut page_cache, &spec, &host, &class, &k.name, "constant");
                    with_methods.insert(class.clone());
                }
                _ => {}
            }
        }
        let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
        for directive in &parsed.directives {
            let kind = format!("{:?}", directive.kind);
            let kind = kind.split(['(', ' ', '{']).next().unwrap_or("").to_string();
            *kinds.entry(kind).or_default() += 1;
        }
        println!(
            "OPTIONS {host}: [{}] directive kinds: {kinds:?}",
            options.join(" ")
        );
    }
    for (class, host) in &declared {
        println!(
            "DECLARES {host} {class}{}",
            if with_methods.contains(class) {
                ""
            } else {
                " (no members)"
            }
        );
    }
    let (_, origin) = tables();
    let mut native: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for line in origin.lines().filter(|l| !l.starts_with('#')) {
        let Some((key, from)) = line.split_once(' ') else {
            continue;
        };
        let class = key.split('.').next().unwrap_or("");
        if declared.contains_key(class) {
            continue;
        }
        let counts = native.entry(class.to_string()).or_default();
        if from.starts_with("same-as") {
            counts[0] += 1;
        } else if from == "native" {
            counts[1] += 1;
        } else {
            counts[2] += 1;
        }
    }
    for (class, [donated, own, other]) in native {
        println!("NATIVE {class}: donated={donated} native={own} host-or-entry={other}");
    }
}

fn member(
    pages: &mut BTreeMap<String, String>,
    spec: &Path,
    host: &str,
    class: &str,
    name: &[u8],
    flags: &str,
) {
    let name = String::from_utf8_lossy(name).to_ascii_uppercase();
    let page = pages.entry(class.to_string()).or_insert_with(|| {
        std::fs::read_to_string(spec.join(format!("{}.md", class.to_ascii_lowercase())))
            .unwrap_or_default()
            .to_ascii_uppercase()
    });
    let named = page.contains(&name);
    println!(
        "MEMBER {host} {class}.{name} {flags} spec={}",
        if page.is_empty() {
            "nopage"
        } else if named {
            "named"
        } else {
            "unnamed"
        }
    );
}
