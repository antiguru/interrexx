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
    assert_eq!(
        store.capacity(&interp).expect("capacity"),
        capacity_of(MINIMUM_SLOTS)
    );
    let slots = array_slots(&interp, store.slots).expect("slots").len();
    assert_eq!(slots, MINIMUM_SLOTS);
    assert_eq!(count_in(&store), 1);
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

/// A key whose `==` puts a hundred entries while a `put` of an index the
/// table does not hold is probing, which rebuilds the store under the probe:
/// the new entry goes into the store the table has afterwards, where a lookup
/// finds it, and not into the arrays the rebuild left behind.
#[test]
fn a_put_whose_probe_rebuilt_the_store_lands_in_the_new_one() {
    let out = output_of(
        "t = .table~new\n\
         t[.key~new('a', t)] = 1\n\
         k = .key~new('k', t)\nk~arm\n\
         t[k] = 'x'\n\
         say t[k] t~items t~hasIndex(k)\n\
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
    assert_eq!(out, "x 102 1\n");
}

/// A key whose `==` removes the very entry it is being compared with, adding
/// nothing, and then answers true: the probe must not answer the hole that
/// removal left, or the `put` would write its item into the hole and be lost.
#[test]
fn a_mid_probe_remove_of_the_compared_entry_does_not_lose_the_put() {
    let out = output_of(
        "t = .table~new\n\
         t[.key~new('a', t)] = 1\n\
         t[.key~new('b', t)] = 2\n\
         k = .key~new('b', t)\nk~arm\n\
         t[k] = 'x'\n\
         say t[k] t~items\n\
         ::class key\n\
         ::method init\n  expose v t armed\n  use arg v, t\n  armed = .false\n\
         ::method arm\n  expose armed\n  armed = .true\n\
         ::method v\n  expose v\n  return v\n\
         ::method hashCode\n  return 'same'\n\
         ::method '=='\n  expose v t armed\n  use arg other\n\
         \x20 if \\other~isA(.key) then return .false\n\
         \x20 if armed & other~v == v then do\n    armed = .false\n    t~remove(other)\n  end\n\
         \x20 return v == other~v\n",
    );
    assert_eq!(out, "x 2\n");
}

/// A key whose `==`, met with the second entry of its chain, removes the
/// chain's oldest and answers false: the `put` that follows the probe must
/// join the chain at the entry that is its oldest now, not at the hole, or
/// the new entry would be linked from the hole and lost to every lookup.
#[test]
fn a_put_after_its_probe_removed_the_oldest_of_its_chain_is_found() {
    let out = output_of(
        "t = .table~new\n\
         t[.key~new('a', t)] = 1\n\
         t[.key~new('b', t)] = 2\n\
         k = .key~new('k', t)\nk~arm\n\
         t[k] = 'x'\n\
         t[.key~new('m', t)] = 'y'\n\
         say t[k] t[.key~new('m', t)] t[.key~new('b', t)] t~items t~hasIndex(.key~new('a', t))\n\
         ::class key\n\
         ::method init\n  expose v t armed\n  use arg v, t\n  armed = .false\n\
         ::method arm\n  expose armed\n  armed = .true\n\
         ::method v\n  expose v\n  return v\n\
         ::method hashCode\n  return 'same'\n\
         ::method '=='\n  expose v t armed\n  use arg other\n\
         \x20 if \\other~isA(.key) then return .false\n\
         \x20 if armed & other~v == 'b' then do\n    armed = .false\n    t~remove(.key~new('a', t))\n  end\n\
         \x20 return v == other~v\n",
    );
    assert_eq!(out, "x y 2 3 0\n");
}

/// Distinct indexes whose hashes are alike share one chain: each entry of it
/// is still one entry under its own index.
#[test]
fn distinct_indexes_of_one_hash_are_one_entry_each() {
    let out = output_of(
        "r = .relation~new\n\
         do i = 1 to 12\n  r~put('v'i, .key~new('L'i))\nend\n\
         line = ''\n\
         do i = 1 to 12\n  k = .key~new('L'i)\n  \
         line = line r~items(k)r~allAt(k)~items\nend\n\
         say line\n\
         say r~removeAll(.key~new('L5'))~items r~items\n\
         ::class key\n\
         ::attribute label\n\
         ::method init\n  expose label\n  use arg label\n\
         ::method hashCode\n  return 'same hash'\n\
         ::method '=='\n  expose label\n  use arg other\n  return label == other~label\n",
    );
    assert_eq!(out, " 11 11 11 11 11 11 11 11 11 11 11 11\n1 11\n");
}

/// A search compares each entry it began with at most once, so it ends even
/// when every comparison adds an entry to the collection searched. The run is
/// bounded by a deadline: a search that restarts on every change fails the
/// test instead of hanging it.
#[test]
fn a_search_whose_comparison_always_adds_an_entry_ends() {
    let outcome = crate::run_program(
        "/t.rex",
        b"r = .relation~new\n\
          r~put('a', 'k')\n\
          q = .grow~new(r)\n\
          say 'hasItem' r~hasItem(q) 'calls' q~calls\n\
          q = .grow~new(r)\n\
          say 'removeItem' r~removeItem(q) 'calls' q~calls\n\
          t = .table~new\n\
          t~put('a', 'k')\n\
          q = .grow~new(t)\n\
          say 'table removeItem' t~removeItem(q) 'calls' q~calls\n\
          ::class grow\n\
          ::attribute calls\n\
          ::method init\n  expose c calls\n  use arg c\n  calls = 0\n\
          ::method '=='\n  expose c calls\n  calls += 1\n  c~put('n'calls, 'k'calls)\n  return 0\n"
            .to_vec(),
        crate::Invocation::none().with_deadline(std::time::Duration::from_secs(20)),
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    // Each search began with the entries the collection held before it: one,
    // then two once the first search had added its one entry.
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "hasItem 0 calls 1\nremoveItem The NIL object calls 2\n\
         table removeItem The NIL object calls 1\n"
    );
}

/// [`a_search_whose_comparison_always_adds_an_entry_ends`] for the searches
/// that compare indexes: a `Relation` searched by index, a `Bag`, whose item
/// is its index, and the keyed lookup of `at`, `put` and `remove`. Every index collides
/// and every comparison of the armed index adds a colliding entry, so a
/// search that restarts on a change never ends. A search must compare the one
/// candidate it began with once. A keyed lookup compares the candidates the
/// chain held when a comparison first changed the collection, here the one it
/// began with and the one that comparison added, each once.
#[test]
fn an_index_search_whose_comparison_always_adds_an_entry_ends() {
    let outcome = crate::run_program(
        "/t.rex",
        b"r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'allAt' r~allAt(q)~items q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'items' r~items(q) q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'supplier' r~supplier(q)~available q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'removeAll' r~removeAll(q)~items q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'hasItem' r~hasItem('a', q) q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'removeItem' r~removeItem('a', q) q~calls\n\
          b = fresh(.bag); q = .grow~new(b, 1)\n\
          say 'bag hasItem' b~hasItem(q) q~calls\n\
          b = fresh(.bag); q = .grow~new(b, 1)\n\
          say 'bag removeItem' b~removeItem(q) q~calls\n\
          t = fresh(.table); q = .grow~new(t, 1)\n\
          say 'table at' t[q] q~calls\n\
          t = fresh(.table); q = .grow~new(t, 1)\n\
          t[q] = 'b'\n\
          say 'table put' t~items q~calls\n\
          t = fresh(.table); q = .grow~new(t, 1)\n\
          say 'table remove' t~remove(q) q~calls\n\
          r = fresh(.relation); q = .grow~new(r, 1)\n\
          say 'relation at' r~at(q) q~calls\n\
          exit\n\
          fresh: procedure\n  use arg class\n  c = class~new\n  \
          k = .grow~new(c, 0)\n  if class == .bag then c~put(k)\n  else c~put('a', k)\n  \
          return c\n\
          ::class grow\n\
          ::attribute calls\n\
          ::method init\n  expose c calls armed\n  use arg c, armed\n  calls = 0\n\
          ::method hashCode\n  return 'h'\n\
          ::method '=='\n  expose c calls armed\n  use arg other\n  \
          if \\armed then return 0\n  calls += 1\n  g = .grow~new(c, 0)\n  \
          if c~isA(.bag) then c~put(g)\n  else c~put('n'calls, g)\n  return 0\n"
            .to_vec(),
        crate::Invocation::none().with_deadline(std::time::Duration::from_secs(20)),
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "allAt 0 1\nitems 0 1\nsupplier 0 1\nremoveAll 0 1\nhasItem 0 1\n\
         removeItem The NIL object 1\nbag hasItem 0 1\nbag removeItem The NIL object 1\n\
         table at The NIL object 2\ntable put 4 2\ntable remove The NIL object 2\nrelation at The NIL object 2\n"
    );
}

/// A `Bag`'s `hasItem` and `removeItem` stop at the first equal member: with
/// every member equal, each sends `==` once, so emptying the bag one
/// `removeItem` at a time sends as many as it held, not a count that grows
/// with the square of it.
#[test]
fn a_bag_search_stops_at_the_first_equal_member() {
    let out = output_of(
        ".local~count = 0\n\
         b = .bag~new\n\
         do i = 1 to 200\n  b~put(.key~new)\nend\n\
         .local~count = 0\n\
         say b~hasItem(.key~new) .count\n\
         .local~count = 0\n\
         do while b~removeItem(.key~new) \\= .nil\nend\n\
         say b~items .count\n\
         ::class key\n\
         ::method hashCode\n  return 'same'\n\
         ::method '=='\n  .local~count = .count + 1\n  return .true\n",
    );
    assert_eq!(out, "1 1\n0 200\n");
}

/// A `Bag`'s `hasItem` and `removeItem` decide by the bag's own store: a
/// comparison that writes another collection leaves the bag as it was, so the
/// search goes on to the later members and finds the equal one.
#[test]
fn a_bag_search_whose_comparison_writes_another_collection_finds_its_member() {
    let out = output_of(
        ".local~log = .set~new\n\
         b = .bag~new\n\
         b~put(.key~new('y'))\n\
         b~put(.key~new('x'))\n\
         say 'hasItem' b~hasItem(.key~new('x'))\n\
         r = b~removeItem(.key~new('x'))\n\
         if r~isA(.key) then say 'removeItem' r~label\n\
         else say 'removeItem none'\n\
         say 'items' b~items b~hasItem(.key~new('y'))\n\
         ::class key\n\
         ::attribute label\n\
         ::method init\n  expose label\n  use arg label\n\
         ::method hashCode\n  return 7\n\
         ::method '=='\n  expose label\n  use arg other\n  \
         .log~put(.object~new)\n  return label == other~label\n",
    );
    assert_eq!(out, "hasItem 1\nremoveItem x\nitems 1 1\n");
}

/// A rebuild of a store whose positions are mostly holes leaves room for
/// four times its live entries, so a store whose removals keep pace with its
/// appends is rebuilt less often than once per as many appends as it holds.
#[test]
fn a_rebuild_of_a_store_of_mostly_holes_leaves_four_times_the_room() {
    let mut interp = Interp::new();
    let table = run_returning(
        &mut interp,
        "t = .table~new\n\
         do i = 1 to 100\n  t[i] = i\nend\n\
         do i = 1 to 90\n  t~remove(i)\nend\n\
         return t\n",
    );
    let store = store_of(&mut interp, table).expect("the table has a store");
    let live = count_in(&store);
    assert_eq!(live, 10);
    assert!(store.used - live > live, "the store is mostly holes");
    let rebuilt = rebuild(
        &mut interp,
        table,
        CONTENTS,
        &store,
        regrown(live, store.used),
    )
    .expect("rebuild");
    let capacity = rebuilt.capacity(&interp).expect("capacity");
    assert!(
        capacity >= 4 * live,
        "capacity {capacity} for {live} live entries"
    );
}

/// Both individual and batched removal reclaim holes while retaining the
/// remaining entries and the order of a multi-value chain.
#[test]
fn mass_removal_compacts_without_losing_entries() {
    let remove_one = r"do i = 1 to 299
        if r~remove('drop') \== ('removed payload number 'i) then raise syntax 40.1
    end";
    let remove_all = r"removed = r~removeAll('drop')
    if removed~items \== 299 then raise syntax 40.1
    if removed[1] \== 'removed payload number 1' then raise syntax 40.1
    if removed[299] \== 'removed payload number 299' then raise syntax 40.1";
    for remove in [remove_one, remove_all] {
        let mut interp = Interp::new();
        interp.enable_stress_collect();
        let relation = run_returning(
            &mut interp,
            &format!(
                "r = .relation~new\n\
                 do i = 1 to 299\n  r~put('removed payload number 'i, 'drop')\nend\n\
                 r~put(17, 'keep')\nr~put(18, 'keep')\n\
                 {remove}\n\
                 r~put(19, 'keep')\n\
                 return r\n"
            ),
        );
        let store = store_of(&mut interp, relation).expect("the relation has a store");
        let positions = entries(&interp, &store).expect("live positions");
        assert_eq!(count_in(&store), 3, "{remove}");
        let values: Vec<_> = positions
            .iter()
            .map(|&position| slot_at(&interp, store.items, position).expect("item"))
            .collect();
        assert_eq!(values, [Some(small(17)), Some(small(18)), Some(small(19))]);
        assert!(store.used < MINIMUM_COMPACT_SPAN, "{remove}: {store:?}");
    }
}

#[test]
fn packed_counts_round_trip_at_their_limits() {
    for (used, live) in [
        (0, 0),
        (MAXIMUM_SLOTS - 1, 0),
        (MAXIMUM_SLOTS - 1, MAXIMUM_SLOTS - 1),
    ] {
        let counts = Counts { used, live };
        assert_eq!(Counts::decode(counts.encode()), Some(counts));
    }
}

#[test]
fn an_insertion_counts_entries_removed_by_its_hashcode() {
    let out = output_of(
        "t = .table~new\n\
         t['victim'] = 1\n\
         k = .key~new(t)\n\
         t[k] = 2\n\
         say t~items t[k] t~hasIndex('victim')\n\
         ::class key\n\
         ::method init\n  expose t\n  use arg t\n\
         ::method hashCode\n  expose t\n  t~remove('victim')\n  return 'same'\n",
    );
    assert_eq!(out, "1 2 0\n");
}

/// Removing the oldest entry of a chain hands the chain's tail to the next
/// entry, which heads it now: an entry appended to the chain afterwards
/// follows the last one, and no entry of the chain is lost. The run is
/// bounded by a deadline: a chain linked into a loop fails the test instead
/// of hanging it.
#[test]
fn removing_the_head_of_a_chain_hands_its_tail_on() {
    let outcome = crate::run_program(
        "/t.rex",
        b"r = .relation~new\n\
          do i = 1 to 3\n  r~put(i, 'dup')\nend\n\
          r~remove('dup')\n\
          r~put(4, 'dup')\n\
          say r~allAt('dup')~makeString('l', ' ') r~items('dup') r~items\n"
            .to_vec(),
        crate::Invocation::none().with_deadline(std::time::Duration::from_secs(20)),
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert_eq!(outcome.exit_code, 0, "{stderr}");
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), "2 3 4 3 3\n");
}

/// The slots of `collection`'s store that file an entry.
fn filed_slots(interp: &mut Interp, collection: ObjRef) -> Vec<usize> {
    let store = store_of(interp, collection).expect("the collection has a store");
    let slots = array_slots(interp, store.slots).expect("slots");
    (0..slots.len())
        .filter_map(|slot| match mark_in(slots, slot) {
            Mark::Entry(position) => Some(position),
            _ => None,
        })
        .collect()
}

/// Duplicates of one index are one chain behind the oldest of them: however
/// many a `Relation` or `Bag` holds under one index, they take one slot, and
/// that slot files the oldest live one, through removals of the oldest and
/// through growth.
#[test]
fn duplicates_of_one_index_take_one_slot() {
    for (class, put, other) in [
        ("relation", "r~put(i, 'dup')", "r~put(0, 'other')"),
        ("bag", "r~put('dup')", "r~put('other')"),
    ] {
        let mut interp = Interp::new();
        let collection = run_returning(
            &mut interp,
            &format!(
                "r = .{class}~new\n\
                 do i = 1 to 100\n  {put}\nend\n\
                 {other}\n\
                 r~remove('dup')\n\
                 r~remove('dup')\n\
                 return r\n"
            ),
        );
        // Positions 0 and 1 are the two removed; the 100 puts grew the store,
        // which kept every entry in its order.
        let mut filed = filed_slots(&mut interp, collection);
        filed.sort_unstable();
        assert_eq!(filed, [2, 100], "{class}");
    }
}

/// A slot whose chain loses its last entry is marked deleted, so the slot
/// index names only live entries, and the next new chain whose probe passes
/// it takes it. Nothing a program can see depends on this, since a probe
/// passes over a slot naming a hole as it does over a deleted one, but a
/// slot left naming the hole would never be reused before a rebuild, and
/// every probe that passes it would take a step more.
#[test]
fn a_chain_that_loses_its_last_entry_frees_its_slot() {
    let mut interp = Interp::new();
    let emptied = run_returning(
        &mut interp,
        "t = .table~new\nt['a'] = 1\nt~remove('a')\nreturn t\n",
    );
    assert_eq!(filed_slots(&mut interp, emptied), Vec::<usize>::new());

    let mut interp = Interp::new();
    let refilled = run_returning(
        &mut interp,
        "t = .table~new\nt['a'] = 1\nt~remove('a')\nt['a'] = 2\nreturn t\n",
    );
    // The same index probes the same slots, so it takes the freed one.
    assert_eq!(filed_slots(&mut interp, refilled), [1]);
}
