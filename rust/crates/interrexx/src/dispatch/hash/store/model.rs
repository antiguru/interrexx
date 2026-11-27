//! A model test of the hash store: random sequences of operations on a
//! `Table`, `Relation`, `Set`, `Bag` or `StringTable`, run as a program,
//! must answer what a list of `(index, item)` pairs in order answers.
//!
//! The model is the order rule of Deviation 33 (`r[class.table.deviation.33]`,
//! `r[class.relation.deviation.33]`) with nothing of the store's layout:
//! entries are listed in the order they were first put, a `put` that
//! replaces the item of an index already there keeps its place, an entry
//! removed and put again comes last, and where several entries qualify the
//! oldest is the one answered or removed.
//!
//! Each sequence is one program. Its indexes mix what the store must treat as
//! one index although it may hold them differently: a number written as a
//! literal, built by arithmetic or by concatenation, a decimal, a string
//! longer than an inline one, and objects of a class whose `hashCode`
//! collides, in part (one of 64 codes) or nearly fully (one of 3), so chains
//! grow long. A fresh index object is made for every use, so only `==` can
//! find an entry. Phases that mostly put and phases that mostly remove take
//! the store across its growth and compaction thresholds in both directions,
//! and copies taken along the way must read later as they did when taken.

use interrexx_cast::{CastFrom, CastTruncate};

/// splitmix64: a small generator whose sequences are fixed by their seed, so
/// a failing seed replays.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        // The remainder is below `n`, so truncating it to a `usize` keeps it whole.
        usize::cast_truncate(self.next() % u64::cast_from(n))
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
}

/// The collection a sequence runs on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Table,
    Relation,
    Set,
    Bag,
    StringTable,
}

impl Kind {
    const ALL: [Kind; 5] = [
        Kind::Table,
        Kind::Relation,
        Kind::Set,
        Kind::Bag,
        Kind::StringTable,
    ];

    fn class(self) -> &'static str {
        match self {
            Kind::Table => ".table",
            Kind::Relation => ".relation",
            Kind::Set => ".set",
            Kind::Bag => ".bag",
            Kind::StringTable => ".stringtable",
        }
    }

    /// Whether a `put` adds another entry under an index already there.
    fn multi(self) -> bool {
        matches!(self, Kind::Relation | Kind::Bag)
    }

    /// Whether the item is the index, `put` taking one argument.
    fn index_only(self) -> bool {
        matches!(self, Kind::Set | Kind::Bag)
    }
}

/// The kinds of index a sequence mixes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Family {
    /// A whole number below [`FAMILY`].
    Whole,
    /// Eight digits: longer than an inline string.
    Long,
    /// A decimal, `n.5`.
    Decimal,
    /// A word, `kn`.
    Word,
    /// A `key` object, whose string value is `Kn`.
    Key,
}

/// An index: the `n`th of its family. Two indexes are equal exactly when
/// their strings are, since no two families share a string, and a key object
/// is never `==` a string either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Index(Family, usize);

impl Index {
    fn render(self) -> String {
        let Index(family, n) = self;
        match family {
            Family::Whole => n.to_string(),
            Family::Long => (10_000_000 + n).to_string(),
            Family::Decimal => format!("{n}.5"),
            Family::Word => format!("k{n}"),
            Family::Key => format!("K{n}"),
        }
    }

    /// An expression that makes this index, in one of the forms it can
    /// take: a literal, arithmetic or concatenation, so that one index
    /// reaches the store in each of the ways a value can be held.
    fn expression(self, rng: &mut Rng) -> String {
        let Index(family, n) = self;
        let form = rng.below(3);
        match (family, form) {
            (Family::Whole, 0) => format!("{n}"),
            (Family::Whole, 1) => format!("'{n}'"),
            (Family::Whole, _) => {
                let part = rng.below(n + 1);
                format!("({part} + {})", n - part)
            }
            (Family::Long, 0) => format!("'{}'", 10_000_000 + n),
            (Family::Long, 1) => format!("(10000000 + {n})"),
            (Family::Long, _) => format!("('1000' || {n:04})"),
            (Family::Decimal, 0) => format!("'{n}.5'"),
            (Family::Decimal, 1) => format!("({n} + 0.5)"),
            (Family::Decimal, _) => format!("({n}.5 * 1)"),
            (Family::Word, 0) => format!("'k{n}'"),
            (Family::Word, _) => format!("('k' || {n})"),
            (Family::Key, _) => format!(".key~new({n})"),
        }
    }
}

/// The number of indexes of each family.
const FAMILY: usize = 300;

/// A random index. A key object is left out of a `StringTable`, whose
/// indexes are strings.
fn draw_index(rng: &mut Rng, kind: Kind) -> Index {
    let families: &[Family] = if kind == Kind::StringTable {
        &[Family::Whole, Family::Long, Family::Decimal, Family::Word]
    } else {
        &[
            Family::Whole,
            Family::Long,
            Family::Decimal,
            Family::Word,
            Family::Key,
        ]
    };
    Index(families[rng.below(families.len())], rng.below(FAMILY))
}

/// The entries in order. An index-only collection's item is its index.
#[derive(Clone, Debug, Default)]
struct Model(Vec<(Index, String)>);

impl Model {
    fn first(&self, index: Index) -> Option<usize> {
        self.0.iter().position(|(held, _)| *held == index)
    }

    fn put(&mut self, kind: Kind, index: Index, item: String) {
        match self.first(index) {
            Some(at) if !kind.multi() => self.0[at].1 = item,
            _ => self.0.push((index, item)),
        }
    }

    fn at(&self, index: Index) -> Option<String> {
        self.first(index).map(|at| self.0[at].1.clone())
    }

    fn remove(&mut self, index: Index) -> Option<String> {
        self.first(index).map(|at| self.0.remove(at).1)
    }

    fn remove_item(&mut self, item: &str) -> Option<String> {
        let at = self.0.iter().position(|(_, held)| held == item)?;
        Some(self.0.remove(at).1)
    }

    fn all_at(&self, index: Index) -> Vec<String> {
        self.0
            .iter()
            .filter(|(held, _)| *held == index)
            .map(|(_, item)| item.clone())
            .collect()
    }

    fn remove_all(&mut self, index: Index) -> Vec<String> {
        let all = self.all_at(index);
        self.0.retain(|(held, _)| *held != index);
        all
    }

    /// What the program's `dump` writes.
    fn dump(&self) -> String {
        let mut line = format!("{}:", self.0.len());
        for (index, item) in &self.0 {
            line.push_str(&format!(" {}={item}", index.render()));
        }
        line
    }
}

/// What the program's `show` writes for an answer.
fn shown(answer: Option<String>) -> String {
    answer.unwrap_or_else(|| "-".to_owned())
}

/// What the program's `showall` writes for an array.
fn shown_all(items: &[String]) -> String {
    let mut line = format!("{}:", items.len());
    for item in items {
        line.push(' ');
        line.push_str(item);
    }
    line
}

const SUPPORT: &str = "\
exit 0
::routine show
  use arg o
  if .nil == o then say '-'
  else say o~string
::routine showall
  use arg a
  line = a~items || ':'
  do e over a
    line = line e~string
  end
  say line
::routine dump
  use arg c
  s = c~supplier
  line = c~items || ':'
  do while s~available
    line = line s~index~string || '=' || s~item~string
    s~next
  end
  say line
::class key
::method init
  expose v
  use arg v
::method v
  expose v
  return v
::method string
  expose v
  return 'K' || v
::method hashCode
  expose v
  if v // 2 = 0 then return right((v % 2) // 64, 8, '0')
  return 'h' || (v // 3)
::method '=='
  expose v
  use arg other
  if \\other~isA(.key) then return .false
  return v == other~v
";

/// One sequence: the program, and for each line it writes, the line the model
/// expects and the clause that wrote it.
struct Sequence {
    program: String,
    expected: Vec<(String, String)>,
}

/// Builds the sequence of `ops` operations on `kind` under `seed`.
fn build(seed: u64, kind: Kind, ops: usize) -> Sequence {
    let mut rng = Rng(seed);
    let mut program = format!("c = {}~new\nsnaps = .array~new\n", kind.class());
    let mut expected = Vec::new();
    let mut model = Model::default();
    let mut snapshots: Vec<Model> = Vec::new();
    let mut adding = true;
    let mut items = 0usize;
    let mut emit = |program: &mut String, clause: String, lines: Vec<String>| {
        program.push_str(&clause);
        program.push('\n');
        for line in lines {
            expected.push((line, clause.clone()));
        }
    };
    for _ in 0..ops {
        if rng.chance(1) {
            adding = !adding;
        }
        let grow = if adding { 65 } else { 20 };
        let choice = rng.below(100);
        // A put mostly names a new index, so the collection grows; every
        // other operation mostly names one it holds.
        let held = if choice < grow { 20 } else { 60 };
        let index = match model.0.len() {
            count if count > 0 && rng.chance(held) => model.0[rng.below(count)].0,
            _ => draw_index(&mut rng, kind),
        };
        let expression = index.expression(&mut rng);
        let item = if kind.index_only() {
            index.render()
        } else {
            items += 1;
            format!("v{}", items % 40)
        };
        match choice {
            r if r < grow => {
                let clause = if kind.index_only() {
                    format!("c~put({expression})")
                } else {
                    format!("c[{expression}] = '{item}'")
                };
                model.put(kind, index, item);
                emit(&mut program, clause, vec![]);
            }
            r if r < grow + 6 => {
                let answer = shown(model.at(index));
                emit(
                    &mut program,
                    format!("call show c[{expression}]"),
                    vec![answer],
                );
            }
            r if r < grow + 10 => {
                let answer = if model.first(index).is_some() {
                    "1"
                } else {
                    "0"
                };
                emit(
                    &mut program,
                    format!("say c~hasIndex({expression})"),
                    vec![answer.to_owned()],
                );
            }
            r if r < grow + 14 => {
                // An item that may be held, under any index.
                let (wanted, literal) = if kind.index_only() {
                    (index.render(), expression)
                } else {
                    let wanted = format!("v{}", rng.below(40));
                    let literal = format!("'{wanted}'");
                    (wanted, literal)
                };
                let answer = shown(model.remove_item(&wanted));
                emit(
                    &mut program,
                    format!("call show c~removeItem({literal})"),
                    vec![answer],
                );
            }
            r if r < grow + 16 && kind == Kind::Relation => {
                let all = model.all_at(index);
                emit(
                    &mut program,
                    format!("say c~items({expression})"),
                    vec![all.len().to_string()],
                );
            }
            r if r < grow + 18 && kind == Kind::Relation => {
                let all = shown_all(&model.all_at(index));
                emit(
                    &mut program,
                    format!("call showall c~allAt({expression})"),
                    vec![all],
                );
            }
            r if r < grow + 19 && kind == Kind::Relation => {
                let all = shown_all(&model.remove_all(index));
                emit(
                    &mut program,
                    format!("call showall c~removeAll({expression})"),
                    vec![all],
                );
            }
            r if r < grow + 21 => {
                // Six slots, a random one replaced once all are taken, so
                // that copies of grown and compacted collections are kept
                // as well as early ones.
                let at = if snapshots.len() < 6 {
                    snapshots.push(model.clone());
                    snapshots.len()
                } else {
                    let at = rng.below(snapshots.len());
                    snapshots[at] = model.clone();
                    at + 1
                };
                emit(&mut program, format!("snaps[{at}] = c~copy"), vec![]);
            }
            r if r < grow + 22 && rng.chance(5) => {
                model = Model::default();
                emit(&mut program, "c~empty".to_owned(), vec![]);
            }
            r if r < grow + 25 => {
                emit(&mut program, "call dump c".to_owned(), vec![model.dump()]);
            }
            _ => {
                let answer = shown(model.remove(index));
                emit(
                    &mut program,
                    format!("call show c~remove({expression})"),
                    vec![answer],
                );
            }
        }
    }
    emit(&mut program, "call dump c".to_owned(), vec![model.dump()]);
    for (at, snapshot) in snapshots.iter().enumerate() {
        emit(
            &mut program,
            format!("call dump snaps[{}]", at + 1),
            vec![snapshot.dump()],
        );
    }
    program.push_str(SUPPORT);
    Sequence { program, expected }
}

/// Runs one sequence and compares what it writes, line by line, with the
/// model.
fn run(seed: u64, kind: Kind, ops: usize) {
    let sequence = build(seed, kind, ops);
    let outcome = crate::run_program(
        "/model.rex",
        sequence.program.clone().into_bytes(),
        crate::Invocation::none(),
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let stdout = String::from_utf8_lossy(&outcome.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    for (at, (want, clause)) in sequence.expected.iter().enumerate() {
        let got = lines.get(at).copied();
        assert_eq!(
            got,
            Some(want.as_str()),
            "{kind:?}, seed {seed}, line {}, after `{clause}`\nstderr: {stderr}",
            at + 1
        );
    }
    assert_eq!(
        lines.len(),
        sequence.expected.len(),
        "{kind:?}, seed {seed}: extra output\nstderr: {stderr}"
    );
    assert_eq!(
        outcome.exit_code, 0,
        "{kind:?}, seed {seed}\nstderr: {stderr}"
    );
}

/// Random sequences on each kind of collection against the model.
#[test]
fn random_sequences_agree_with_the_model() {
    for seed in 0..24 {
        for kind in Kind::ALL {
            run(seed, kind, 2_500);
        }
    }
}
