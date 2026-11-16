use std::rc::Rc;

use interrexx_parse::parse_program;

use super::*;
use crate::Activation;
use crate::plan::BodyKey;

/// Runs `source`, directives included, in `interp` and answers what its main
/// program returns. The setup `Interp::run` does, as `run/tests.rs` does it.
fn run_returning(interp: &mut Interp, source: &str) -> ObjRef {
    let program = parse_program(source.as_bytes().to_vec()).expect("test program parses");
    let program = Rc::new(program);
    let program_id = interp.programs.insert(Rc::clone(&program));
    interp
        .install_directives(program_id, &program)
        .expect("directives install");
    let plan = interp.plan_for(
        BodyKey {
            program: program_id,
            directive: None,
        },
        &program.main,
        &program.symbols,
        &program.source,
    );
    let frame = interp.roots.push_slots(plan.len());
    let id = interp.next_activation_id();
    interp.push_activation(Activation::new(
        id,
        Rc::clone(&program),
        program_id,
        plan,
        frame,
    ));
    interp
        .run_activation()
        .expect("test program runs")
        .value()
        .expect("test program returns a value")
}

/// Runs `source` as a program and answers what it wrote.
fn output_of(source: &str) -> String {
    let outcome = crate::run_program(
        "/t.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    String::from_utf8_lossy(&outcome.stdout).into_owned()
}

/// A store's rebuilds re-file entries from their cached codes: filling a table
/// through several rebuilds sends `HASHCODE` once per `put` and never more.
#[test]
fn growth_sends_no_hashcode() {
    let out = output_of(
        ".local~count = 0\n\
         t = .table~new\n\
         do i = 1 to 1000\n  t[.key~new(i)] = i\nend\n\
         say .count t~items\n\
         ::class key\n\
         ::method init\n  expose v\n  use arg v\n\
         ::method hashCode\n  expose v\n  .local~count = .count + 1\n  return v~hashCode\n",
    );
    assert_eq!(out, "1000 1000\n");
}

/// A million put and remove cycles of one key leave the store at its minimum
/// size: a rebuild sizes by the live entries, not by the positions the
/// removed ones used.
#[test]
fn churn_stays_bounded() {
    let mut interp = Interp::new();
    let table = run_returning(
        &mut interp,
        "t = .table~new\n\
         do 1000000\n  t['k'] = 1\n  t~remove('k')\nend\n\
         t['k'] = 2\n\
         return t\n",
    );
    let store = store_of(&mut interp, table).expect("the table has a store");
    assert_eq!(store.capacity, capacity_of(MINIMUM_SLOTS));
    let slots = array_slots(&interp, store.slots).expect("slots").len();
    assert_eq!(slots, MINIMUM_SLOTS);
    assert_eq!(count_in(&interp, &store).expect("count"), 1);
}

/// A key whose `==` puts a hundred entries into the table it is being looked
/// up in, which rebuilds and compacts the store under the probe, still finds
/// its own entry or none, never another key's, and a `put` or `remove` it is
/// the index of acts on the store the table has afterwards.
#[test]
fn a_mid_probe_put_does_not_find_a_wrong_entry() {
    let out = output_of(
        "t = .table~new\n\
         keys = .array~new\n\
         do i = 1 to 6\n  keys[i] = .key~new('k'i, t)\n  t[keys[i]] = 'item'i\nend\n\
         t~remove(keys[1])\nt~remove(keys[2])\nt~remove(keys[3])\n\
         hit = .key~new('k5', t)\nhit~arm\n\
         say t[hit]\n\
         say t~items\n\
         miss = .key~new('k9', t)\nmiss~arm\n\
         say t[miss]\n\
         say t~items\n\
         put = .key~new('k5', t)\nput~arm\n\
         t[put] = 'new'\n\
         say t[keys[5]] t~items\n\
         gone = .key~new('k6', t)\ngone~arm\n\
         say t~remove(gone) t~hasIndex(keys[6]) t~items\n\
         ::class key\n\
         ::method init\n  expose v t armed\n  use arg v, t\n  armed = .false\n\
         ::method arm\n  expose armed\n  armed = .true\n\
         ::method v\n  expose v\n  return v\n\
         ::method hashCode\n  return 'same'\n\
         ::method '=='\n  expose v t armed\n  use arg other\n\
         \x20 if armed then do\n    armed = .false\n    n = t~items\n    \
         do i = n to n + 99\n      t[i] = i\n    end\n  end\n\
         \x20 if \\other~isA(.key) then return .false\n\
         \x20 return v == other~v\n",
    );
    assert_eq!(
        out,
        "item5\n103\nThe NIL object\n203\nnew 303\nitem6 0 402\n"
    );
}
