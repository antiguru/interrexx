//! `ClassGraph`: the class-object cascade over `MethodDict`.

use crate::method_dict::{MethodDict, MethodSlot};
use interrexx_core::MethodId;
use interrexx_core::NameMap;
use interrexx_core::{BehaviourHandle, ObjRef};
use std::collections::{BTreeSet, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

/// The source of every [`ClassGraph::epoch`] value in the process, so that two
/// graphs never share one: a stamp taken from one graph cannot match another.
static NEXT_EPOCH: AtomicU64 = AtomicU64::new(1);

/// A stamp of a graph's dispatch-visible state: see [`ClassGraph::epoch`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Epoch(u64);

impl Epoch {
    fn fresh() -> Epoch {
        Epoch(NEXT_EPOCH.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for Epoch {
    fn default() -> Epoch {
        Epoch::fresh()
    }
}

/// Whether a class was declared with `SUBCLASS`/plain inheritance or with
/// `MIXINCLASS`. The only difference this makes is `base_class`: a
/// `Mixin`'s is inherited from the class named after `MIXINCLASS`,
/// a `Regular`
/// class's is itself.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ClassKind {
    Regular,
    Mixin,
}

/// Why [`ClassGraph::inherit`] refused to build the edge, each variant named
/// for the `SYNTAX` condition `~inherit` reports in its place.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum InheritRefusal {
    /// `SYNTAX` 98.942: the target is not a
    /// `MIXINCLASS`, so nothing may inherit it.
    NotAMixin,
    /// `SYNTAX` 98.944: one of
    /// the two classes is already an ancestor of the other, in whichever
    /// direction.
    Recursive,
    /// `SYNTAX` 98.943: the inheriting class does not already have the mixin's
    /// base class in scope. The base class travels with the refusal because the
    /// message names it.
    BaseClass(ObjRef),
    /// `SYNTAX` 98.945: a class named as a position or as an `~uninherit`
    /// target is not one this class inherits. The class travels with the
    /// refusal because the message names it, and it is the *named* class rather
    /// than the mixin: `.K~inherit(.M1, .M2)` reports `has not inherited class
    /// "The M2 class"`.
    NotInherited(ObjRef),
}

/// Which of a class's two behaviours an internal cascade operation
/// targets.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Side {
    Instance,
    Class,
}

/// One flattened behaviour, plus a monotonic version bumped by every write to
/// its dictionary, a handle's reuse included: the dictionary of a freed
/// handle is cleared, which is a write.
struct Behaviour {
    dict: MethodDict,
    version: u64,
}

struct ClassDef {
    /// `Regular` or `Mixin`. `inherit`'s mixin check reads this: only a `Mixin`
    /// may be inherited.
    kind: ClassKind,
    /// Cascade order: the explicit superclass first (if any), then every mixin
    /// `inherit` has folded in, in the order it was called.
    superclasses: Vec<ObjRef>,
    /// Every class that must be rebuilt when this one's own dictionaries
    /// change: populated by both [`ClassGraph::define_class`] (the explicit
    /// superclass) and [`ClassGraph::inherit`] (a mixin), and walked
    /// identically either way, populated uniformly from both `~subclass` and
    /// `~inherit`.
    subclasses: Vec<ObjRef>,
    /// This class's own instance methods, always at its own scope.
    own_instance_methods: MethodDict,
    /// This class's own class (static) methods.
    own_class_methods: MethodDict,
    /// Rebuilt in place by every change, never replaced: every instance
    /// holds this handle, so each sees every change (`r[class.methods.late]`).
    instance_behaviour: BehaviourHandle,
    class_behaviour: BehaviourHandle,
    /// What `inherit`'s base-class compatibility check reads.
    base_class: ObjRef,
    /// The class whose *instance* behaviour gets merged into this class's own
    /// class-behaviour by [`ClassGraph::cascade_build`] (a class's class
    /// behaviour includes its metaclass's instance behaviour). Every class the
    /// bootstrap builds has `.Class` here, including `.Class` itself,
    /// self-referentially: `.class~metaclass~id` is `"Class"`.
    metaclass: ObjRef,
    /// The class this class object's own behaviour belongs to, set from the
    /// metaclass `~subclass` was given or defaulted to, and what `~class`
    /// answers for a class object.
    /// ```text
    /// ::CLASS T  SUBCLASS MC METACLASS M1   ~metaClass MC     ~class M1      part
    /// ::CLASS T2 SUBCLASS MC                ~metaClass MC     ~class Class   part
    /// ::CLASS M3 SUBCLASS MC METACLASS MC   ~metaClass MC     ~class MC      same
    /// ::class MC MIXINCLASS Class           ~metaClass Class  ~class Class   same
    /// ::class Z  SUBCLASS Class             ~metaClass Class  ~class Class   same
    /// ::CLASS K  METACLASS M1               ~metaClass M1     ~class M1      same
    /// ```
    owning_class: ObjRef,
    /// This class may be named as another class's metaclass, which `~subclass`
    /// tests before it will build anything from the metaclass it was handed.
    is_metaclass: bool,
    /// This class's instances need `UNINIT` run when they are collected, which
    /// is what instance creation reads to register each new instance.
    has_uninit: bool,
    /// Some class this one inherits from carries `UNINIT`, set by each
    /// constructor that builds an inheritance edge: `~subclass`, `~mixinClass`
    /// and `~inherit`.
    parent_has_uninit: bool,
    /// `~new` on this class refuses, set by marking the class abstract and read
    /// as the first step of instance creation.
    is_abstract: bool,
    /// This class may not be altered from Rexx. Image preparation sets it on every
    /// class in the image, which is every class the image holds: the ones the native
    /// bootstrap builds and the ones the interpreter's own Rexx-written library
    /// declares alike. A `::CLASS` a program declares does not carry it.
    rexx_defined: bool,
    /// Built by [`ClassGraph::define_cloned_class`] and not yet given a
    /// superclass by [`ClassGraph::reparent`].
    cloned: bool,
}

/// The class graph and its behaviour storage.
#[derive(Default)]
pub struct ClassGraph {
    /// [`NameMap`] for `MethodDict`'s reason: the cascade reads this map
    /// once per superclass per class while the library is being built.
    classes: NameMap<ObjRef, ClassDef>,
    behaviours: Vec<Behaviour>,
    /// The behaviours an expunged class emptied, which a later allocation
    /// takes before it grows [`ClassGraph::behaviours`]. Only an instance of
    /// a class holds one of its handles, and an instance keeps its class
    /// alive.
    free_behaviours: Vec<BehaviourHandle>,
    /// The methods a `define`, `defineMethods` or `delete` took out of a
    /// class's own instance dictionary since [`ClassGraph::take_unnamed_methods`],
    /// each with that class: a method no dictionary names any more can be let
    /// go of. The image's classes record none.
    dropped: HashSet<(ObjRef, MethodId)>,
    /// The uninit table restricted to class objects: the entries the uninit
    /// check makes, in the order it makes them.
    uninit_classes: Vec<ObjRef>,
    /// See [`ClassGraph::epoch`].
    epoch: Epoch,
    /// When each instance behaviour's inputs last changed and when each was
    /// built, which says whether [`ClassGraph::define`] may recompute only
    /// the names it changed.
    inputs: InputTicks,
    /// Bumped by every change to which classes exist or to any class's
    /// `superclasses`: a define, a reparent, an `inherit`, an `uninherit` and
    /// an expunge. An answer derived from the ancestry ([`Self::ancestors`]
    /// read transitively, as `is_a` does) holds while this stays put. A define
    /// counts because an identity an expunge freed can be minted again.
    ancestry: u64,
}

/// A clock over the cascade's instance-side inputs -- each class's own
/// instance dictionary and superclass list -- read by the mutators that
/// change one name ([`ClassGraph::define`] and its siblings): a behaviour
/// built after every input it was built from last changed is the cascade of
/// those inputs, so the change can be applied to that one name of it rather
/// than rebuilding it whole. A behaviour built before is rebuilt whole, as a
/// class an update never reached (a subclass a failed `subclass` left out
/// of its superclass's list) needs.
#[derive(Default)]
struct InputTicks {
    /// Raised by every change of an input.
    now: u64,
    /// When each class's inputs last changed; a class not in the map has
    /// not changed since the clock started.
    changed: NameMap<ObjRef, u64>,
    /// When each behaviour, by index, was last built by the cascade.
    built: Vec<u64>,
}

impl InputTicks {
    /// Records a change of `class`'s own instance dictionary or superclasses.
    fn touch(&mut self, class: ObjRef) {
        self.now += 1;
        self.changed.insert(class, self.now);
    }

    fn changed(&self, class: ObjRef) -> u64 {
        self.changed.get(&class).copied().unwrap_or(0)
    }

    fn set_built(&mut self, handle: BehaviourHandle) {
        let index = handle.index();
        if self.built.len() <= index {
            self.built.resize(index + 1, 0);
        }
        self.built[index] = self.now;
    }

    fn built(&self, handle: BehaviourHandle) -> u64 {
        self.built.get(handle.index()).copied().unwrap_or(0)
    }
}

impl ClassGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// A stamp that changes whenever anything a method lookup, a super-scope
    /// resolution or a class's identity can answer may have changed: every
    /// public mutator takes a new one, except the few that touch only the
    /// uninit and abstract flags, the retirement counters and the
    /// Rexx-defined mark. Unique across every graph in the process, so a
    /// per-site cache filled under one stamp is valid exactly while the graph
    /// still answers it.
    pub fn epoch(&self) -> Epoch {
        self.epoch
    }

    fn bump_epoch(&mut self) {
        self.epoch = Epoch::fresh();
    }

    /// See [`ClassGraph::ancestry`]: the stamp an answer derived from the
    /// superclass lists is valid against.
    pub fn ancestry_stamp(&self) -> u64 {
        self.ancestry
    }

    fn ancestry_changed(&mut self) {
        self.ancestry = self.ancestry.wrapping_add(1);
    }

    fn alloc_behaviour(&mut self) -> BehaviourHandle {
        if let Some(handle) = self.free_behaviours.pop() {
            return handle;
        }
        let handle = BehaviourHandle::new(self.behaviours.len());
        self.behaviours.push(Behaviour {
            dict: MethodDict::new(),
            version: 0,
        });
        handle
    }

    /// Register a new class and build its two initial behaviours by cascading
    /// from `superclass`. `superclass` is `None` only for a root class with no
    /// ancestor (a test's own stand-in for `.Object`).
    pub fn define_class(
        &mut self,
        id: ObjRef,
        superclass: Option<ObjRef>,
        kind: ClassKind,
        metaclass: ObjRef,
    ) {
        self.bump_epoch();
        let derived_from_metaclass = superclass.filter(|sup| self.classes[sup].is_metaclass);
        let owning_class = metaclass;
        let metaclass = derived_from_metaclass.unwrap_or(metaclass);
        let (base_class, parent_has_uninit) = match kind {
            ClassKind::Regular => (id, superclass.is_some_and(|sup| self.uninit_reaches(sup))),
            ClassKind::Mixin => {
                let target = superclass.expect("a mixin class names its MIXINCLASS target");
                (
                    self.classes[&target].base_class,
                    self.uninit_reaches(target),
                )
            }
        };
        self.ancestry_changed();
        let instance_behaviour = self.alloc_behaviour();
        let class_behaviour = self.alloc_behaviour();
        self.classes.insert(
            id,
            ClassDef {
                kind,
                superclasses: superclass.into_iter().collect(),
                subclasses: Vec::new(),
                own_instance_methods: MethodDict::new(),
                own_class_methods: MethodDict::new(),
                instance_behaviour,
                class_behaviour,
                base_class,
                metaclass,
                owning_class,
                is_metaclass: derived_from_metaclass.is_some(),
                has_uninit: false,
                parent_has_uninit,
                is_abstract: false,
                rexx_defined: false,
                cloned: false,
            },
        );
        if let Some(sup) = superclass {
            self.classes.get_mut(&sup).unwrap().subclasses.push(id);
        }
        self.inputs.touch(id);
        self.rebuild_behaviour(id, Side::Instance);
        self.rebuild_behaviour(id, Side::Class);
    }

    /// A class `~new` makes: a copy of
    /// `receiver` whose class behaviour is a copy of `receiver`'s instance
    /// behaviour, whose instance side derives from `root` alone, and which no
    /// subclass list holds. `metaclass` is `receiver` where it is not one of
    /// the image's own classes, and `.Class` where it is.
    pub fn define_cloned_class(
        &mut self,
        id: ObjRef,
        receiver: ObjRef,
        root: ObjRef,
        metaclass: ObjRef,
    ) {
        self.bump_epoch();
        self.ancestry_changed();
        let instance_behaviour = self.alloc_behaviour();
        let class_behaviour = self.alloc_behaviour();
        self.classes.insert(
            id,
            ClassDef {
                kind: ClassKind::Regular,
                superclasses: vec![root],
                subclasses: Vec::new(),
                own_instance_methods: MethodDict::new(),
                own_class_methods: MethodDict::new(),
                instance_behaviour,
                class_behaviour,
                base_class: id,
                metaclass,
                owning_class: receiver,
                is_metaclass: false,
                has_uninit: false,
                parent_has_uninit: false,
                is_abstract: false,
                rexx_defined: false,
                cloned: true,
            },
        );
        self.inputs.touch(id);
        self.rebuild_behaviour(id, Side::Instance);
        let source = self.classes[&receiver].instance_behaviour;
        let dict = self.behaviours[source.index()].dict.clone();
        let behaviour = &mut self.behaviours[class_behaviour.index()];
        behaviour.dict = dict;
        behaviour.version += 1;
    }

    /// A behaviour of its own holding `class`'s current instance methods with
    /// each of `hidden` hidden in front, as `defineInstanceMethod` with a
    /// `.nil` method does to one object's behaviour copy.
    pub fn hiding_behaviour(&mut self, class: ObjRef, hidden: &[&str]) -> BehaviourHandle {
        self.bump_epoch();
        let source = self.classes[&class].instance_behaviour;
        let mut dict = self.behaviours[source.index()].dict.clone();
        for name in hidden {
            dict.add_hidden(name);
        }
        let handle = self.alloc_behaviour();
        let behaviour = &mut self.behaviours[handle.index()];
        behaviour.dict = dict;
        behaviour.version += 1;
        handle
    }

    /// How many names `class`'s two behaviours hold entries under.
    pub fn behaviour_names(&self, class: ObjRef) -> usize {
        let def = &self.classes[&class];
        self.behaviours[def.instance_behaviour.index()].dict.len()
            + self.behaviours[def.class_behaviour.index()].dict.len()
    }

    /// How many names `class`'s own two method dictionaries hold entries
    /// under, inherited names aside.
    pub fn own_method_names(&self, class: ObjRef) -> usize {
        let def = &self.classes[&class];
        def.own_instance_methods.len() + def.own_class_methods.len()
    }

    /// Whether `class` is one [`Self::define_cloned_class`] built that
    /// [`Self::reparent`] has not yet rewired.
    pub fn is_cloned(&self, class: ObjRef) -> bool {
        self.classes[&class].cloned
    }

    /// `~subclass`'s rewiring of the class its metaclass's `NEW`
    /// answered: `superclass` becomes its
    /// only superclass, it becomes a metaclass when `superclass` is one,
    /// `metaclass` owns its class behaviour, and both behaviours are rebuilt
    /// in place. Its own method dictionaries are kept.
    pub fn reparent(&mut self, id: ObjRef, superclass: ObjRef, kind: ClassKind, metaclass: ObjRef) {
        self.bump_epoch();
        let derived_from_metaclass = self.classes[&superclass].is_metaclass;
        let base_class = match kind {
            ClassKind::Regular => id,
            ClassKind::Mixin => self.classes[&superclass].base_class,
        };
        let parent_has_uninit = self.uninit_reaches(superclass);
        self.ancestry_changed();
        let old = std::mem::take(&mut self.classes.get_mut(&id).unwrap().superclasses);
        for sup in old {
            if let Some(def) = self.classes.get_mut(&sup) {
                def.subclasses.retain(|sub| *sub != id);
            }
        }
        let def = self.classes.get_mut(&id).unwrap();
        def.kind = kind;
        def.superclasses = vec![superclass];
        def.base_class = base_class;
        if derived_from_metaclass {
            def.is_metaclass = true;
            def.metaclass = superclass;
        }
        def.owning_class = metaclass;
        def.parent_has_uninit = parent_has_uninit;
        def.cloned = false;
        self.classes
            .get_mut(&superclass)
            .unwrap()
            .subclasses
            .push(id);
        self.inputs.touch(id);
        self.rebuild_behaviour(id, Side::Instance);
        self.rebuild_behaviour(id, Side::Class);
    }

    /// Makes `id`, built as a regular class, a mixin class of its superclass:
    /// the step `mixinClass` takes once the second `init` has run, which
    /// gives it the superclass's base class.
    pub fn complete_mixin(&mut self, id: ObjRef) {
        self.bump_epoch();
        let superclass = self.classes[&id].superclasses[0];
        let base_class = self.classes[&superclass].base_class;
        let def = self.classes.get_mut(&id).unwrap();
        def.kind = ClassKind::Mixin;
        def.base_class = base_class;
    }

    /// Takes `id` out of its superclasses' subclass lists, as a `subclass`
    /// that raised before its second `init` leaves the receiver: the class
    /// itself is kept, but nothing lists it.
    pub fn unlist_subclass(&mut self, id: ObjRef) {
        self.bump_epoch();
        let superclasses = self.classes[&id].superclasses.clone();
        for sup in superclasses {
            if let Some(def) = self.classes.get_mut(&sup) {
                def.subclasses.retain(|sub| *sub != id);
            }
        }
    }

    /// `~enhanced`'s merge of its methods into the dummy subclass's
    /// own instance dictionary, each defined at `scope`, `None` hiding the
    /// name, then the instance behaviour rebuilt in place.
    pub fn enhance_instance_methods(
        &mut self,
        class: ObjRef,
        scope: ObjRef,
        methods: &[(String, Option<MethodId>)],
    ) {
        self.bump_epoch();
        let def = self
            .classes
            .get_mut(&class)
            .expect("enhance_instance_methods: unknown class");
        for (name, method) in methods {
            match method {
                Some(id) => def.own_instance_methods.add_method(name, scope, *id),
                None => def.own_instance_methods.add_hidden(name),
            }
        }
        self.inputs.touch(class);
        self.rebuild_behaviour(class, Side::Instance);
        self.follow_metaclasses(class);
    }

    /// `~enhanced` sets the owning class of an enhanced object that is a class:
    /// what its `~class` answers.
    pub fn set_owning_class(&mut self, class: ObjRef, owner: ObjRef) {
        self.bump_epoch();
        self.classes
            .get_mut(&class)
            .expect("set_owning_class: unknown class")
            .owning_class = owner;
    }

    /// Whether the class defines `UNINIT` or a parent does, the exact pair
    /// every propagation site asks about the class it is deriving from.
    fn uninit_reaches(&self, class: ObjRef) -> bool {
        let def = &self.classes[&class];
        def.has_uninit || def.parent_has_uninit
    }

    /// Whether `class`'s instances need `UNINIT`. See the field for what sets
    /// it.
    pub fn has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].has_uninit
    }

    /// The uninit check, both halves of it: set [`Self::has_uninit`] when the
    /// class's **flattened** instance behaviour answers `UNINIT`, whether the
    /// class defines it or inherits it, and enter the class object itself in
    /// the `uninit_classes` list when its **class** behaviour answers `UNINIT`
    /// -- which is where a `::METHOD uninit CLASS` lands.
    pub fn check_uninit(&mut self, class: ObjRef) {
        let instance = self.behaviour_handle(class, Side::Instance);
        if self.behaviours[instance.index()].dict.has_method("UNINIT") {
            self.classes.get_mut(&class).unwrap().has_uninit = true;
        }
        let class_side = self.behaviour_handle(class, Side::Class);
        if self.behaviours[class_side.index()]
            .dict
            .has_method("UNINIT")
            && !self.uninit_classes.contains(&class)
        {
            self.uninit_classes.push(class);
        }
    }

    /// Drops every trace of `dead` from the graph, and scrubs the subclass
    /// lists of the classes that survive.
    pub fn expunge(&mut self, dead: &[ObjRef]) {
        self.bump_epoch();
        self.ancestry_changed();
        let ClassGraph {
            classes,
            behaviours,
            free_behaviours,
            dropped,
            uninit_classes,
            epoch: _,
            inputs,
            ancestry: _,
        } = self;
        for class in dead {
            // The dictionaries are the weight: a flattened behaviour holds
            // every method the class answers, inherited ones included.
            // Emptied rather than removed, because a behaviour is named by
            // its index and removing one would move every later index.
            if let Some(def) = classes.get(class) {
                for handle in [def.instance_behaviour, def.class_behaviour] {
                    let behaviour = &mut behaviours[handle.index()];
                    behaviour.dict.clear();
                    behaviour.version += 1;
                    debug_assert!(!free_behaviours.contains(&handle), "{handle:?} freed twice");
                    free_behaviours.push(handle);
                }
            }
            classes.remove(class);
            inputs.changed.remove(class);
            uninit_classes.retain(|held| held != class);
        }
        // A subclass list holds classes that may have died: the list is of weak
        // references and pruned as it is read.
        for def in classes.values_mut() {
            def.subclasses.retain(|sub| !dead.contains(sub));
        }
        dropped.retain(|(class, _)| !dead.contains(class));
    }

    /// Whether a method taken out of a class's own dictionary waits for
    /// [`Self::take_unnamed_methods`].
    pub fn has_dropped_methods(&self) -> bool {
        !self.dropped.is_empty()
    }

    /// Forgets the dropped methods without answering them, for the image's
    /// classes once the library is built: they are never collected.
    pub fn forget_dropped_methods(&mut self) {
        self.dropped = HashSet::new();
    }

    /// Records the method `class`'s own instance dictionary holds under
    /// `name`, which the change about to be made replaces or removes.
    fn note_dropped(&mut self, class: ObjRef, name: &str) {
        let def = &self.classes[&class];
        if def.rexx_defined {
            return;
        }
        if let Some(MethodSlot::Defined { method, .. }) = def.own_instance_methods.slot(name) {
            self.dropped.insert((class, method));
        }
    }

    /// Answers each dropped method that no behaviour and no class's own
    /// dictionary names any more, with the class that dropped it, and
    /// forgets every dropped method.
    pub fn take_unnamed_methods(&mut self) -> Vec<(ObjRef, MethodId)> {
        let ClassGraph {
            classes,
            behaviours,
            dropped,
            ..
        } = self;
        let mut unnamed: Vec<(ObjRef, MethodId)> = dropped.drain().collect();
        if !unnamed.is_empty() {
            // Only the dropped methods are in question, a few
            // hundred where the dictionaries hold tens of thousands of
            // entries: probe the entries against those, oldest first, and pass
            // by every dictionary whose newest method is older than the oldest
            // still open -- which, once the inherited methods are found, is
            // every dictionary built before the dropped methods were minted.
            let mut open: Vec<u32> = unnamed.iter().map(|(_, method)| method.0).collect();
            open.sort_unstable();
            open.dedup();
            let mut named = vec![false; open.len()];
            let mut oldest = 0;
            let dicts = behaviours.iter().map(|behaviour| &behaviour.dict).chain(
                classes
                    .values()
                    .flat_map(|def| [&def.own_instance_methods, &def.own_class_methods]),
            );
            'scan: for dict in dicts {
                if dict.newest().is_none_or(|newest| newest.0 < open[oldest]) {
                    continue;
                }
                for method in dict.methods() {
                    let Ok(at) = open.binary_search(&method.0) else {
                        continue;
                    };
                    named[at] = true;
                    while named[oldest] {
                        oldest += 1;
                        if oldest == open.len() {
                            break 'scan;
                        }
                    }
                }
            }
            unnamed.retain(|(_, method)| open.binary_search(&method.0).is_ok_and(|at| !named[at]));
            unnamed.sort_unstable_by_key(|(class, method)| (class.bits(), method.0));
            unnamed.dedup();
        }
        unnamed
    }

    /// Records `method` of `class` as dropped again, for one
    /// [`Self::take_unnamed_methods`] answered that is still running.
    pub fn drop_later(&mut self, class: ObjRef, method: MethodId) {
        self.dropped.insert((class, method));
    }

    /// Whether `class` is waiting to have its class-side `UNINIT` run.
    pub fn has_pending_class_uninit(&self, class: ObjRef) -> bool {
        self.uninit_classes.contains(&class)
    }

    /// Drops `class` from the pending list, for a finalizer a collection has
    /// already run.
    pub fn forget_uninit_class(&mut self, class: ObjRef) {
        self.uninit_classes.retain(|held| *held != class);
    }

    /// Every class object whose own behaviour answers `UNINIT`, in the order
    /// [`Self::check_uninit`] entered them, taken out of the graph.
    pub fn take_uninit_classes(&mut self) -> Vec<ObjRef> {
        std::mem::take(&mut self.uninit_classes)
    }

    /// Recompute [`Self::parent_has_uninit`] from the class's current
    /// superclass list.
    pub fn refresh_parent_has_uninit(&mut self, class: ObjRef) {
        let reached = self.classes[&class]
            .superclasses
            .clone()
            .into_iter()
            .any(|sup| self.uninit_reaches(sup));
        if reached {
            self.classes.get_mut(&class).unwrap().parent_has_uninit = true;
        }
    }

    /// Whether a class `class` inherits from defines `UNINIT`.
    pub fn parent_has_uninit(&self, class: ObjRef) -> bool {
        self.classes[&class].parent_has_uninit
    }

    /// The abstract marking past its metaclass refusal, which the caller raises
    /// -- see [`ClassDef::is_abstract`].
    pub fn make_abstract(&mut self, class: ObjRef) {
        self.classes.get_mut(&class).unwrap().is_abstract = true;
    }

    /// The abstract test, which instance creation reads.
    pub fn is_abstract(&self, class: ObjRef) -> bool {
        self.classes[&class].is_abstract
    }

    fn behaviour_handle(&self, class: ObjRef, side: Side) -> BehaviourHandle {
        let def = &self.classes[&class];
        match side {
            Side::Instance => def.instance_behaviour,
            Side::Class => def.class_behaviour,
        }
    }

    fn behaviour_has_scope(&self, class: ObjRef, side: Side, scope: ObjRef) -> bool {
        let handle = self.behaviour_handle(class, side);
        self.behaviours[handle.index()].dict.has_scope(scope)
    }

    /// The instance- and class-behaviour construction, unified over which
    /// per-class dictionary side is folded in. Ancestors are processed
    /// furthest-inherited-first (`superclasses.iter().rev()`), skipping a
    /// superclass already present -- the diamond guard -- then this class's own
    /// dictionary is folded in last. The most recently folded-in ancestor
    /// always wins a name conflict, which is the merge-order rule the mixin and
    /// diamond tests check: the first-listed `INHERIT` mixin is folded in
    /// *last* among the mixins (closest to `class`'s own scope in cascade
    /// order), and an explicit `SUBCLASS` target is folded in last of all the
    /// ancestors, so it outranks every mixin.
    fn cascade_build(
        classes: &NameMap<ObjRef, ClassDef>,
        behaviours: &[Behaviour],
        class: ObjRef,
        target: &mut MethodDict,
        side: Side,
    ) {
        let superclasses = classes[&class].superclasses.clone();
        for sup in superclasses.into_iter().rev() {
            if !target.has_scope(sup) {
                Self::cascade_build(classes, behaviours, sup, target, side);
            }
        }
        if !target.has_scope(class) {
            if side == Side::Class && !classes[&class].superclasses.is_empty() {
                let metaclass = classes[&class].metaclass;
                if !target.has_scope(metaclass) {
                    let meta_instance = classes[&metaclass].instance_behaviour;
                    target.merge(&behaviours[meta_instance.index()].dict);
                }
            }
            let own = match side {
                Side::Instance => &classes[&class].own_instance_methods,
                Side::Class => &classes[&class].own_class_methods,
            };
            target.merge_methods(own);
            target.add_scope(class);
        }
    }

    /// The rebuild of a behaviour: clear the *existing* dictionary in place, then cascade into
    /// it, rather than building a separate dictionary and swapping it in. The handle's
    /// identity is unaffected either way.
    fn rebuild_behaviour(&mut self, class: ObjRef, side: Side) {
        let handle = self.behaviour_handle(class, side);
        let mut dict = std::mem::take(&mut self.behaviours[handle.index()].dict);
        dict.clear();
        dict.reserve(self.rebuilt_names(class, side));
        Self::cascade_build(&self.classes, &self.behaviours, class, &mut dict, side);
        let behaviour = &mut self.behaviours[handle.index()];
        behaviour.dict = dict;
        behaviour.version += 1;
        if side == Side::Instance {
            self.inputs.set_built(handle);
        }
    }

    /// The names a rebuild of `class`'s `side` behaviour makes room for up
    /// front: its own and its first superclass's.
    fn rebuilt_names(&self, class: ObjRef, side: Side) -> usize {
        let def = &self.classes[&class];
        let own = match side {
            Side::Instance => def.own_instance_methods.len(),
            Side::Class => def.own_class_methods.len(),
        };
        let inherited = def.superclasses.first().map_or(0, |sup| {
            self.behaviours[self.behaviour_handle(*sup, side).index()]
                .dict
                .len()
        });
        inherited + own
    }

    /// The full subclass update: rebuild both behaviours in place, instance
    /// side first ("the added methods may have an impact on metaclasses"), then
    /// [`check_uninit`](Self::check_uninit), then cascade to every subclass.
    /// This is what [`inherit`](Self::inherit) calls, and it never allocates a
    /// new handle -- every class it touches, direct or cascaded, keeps the id
    /// its instances were created with.
    fn update_sub_classes(&mut self, class: ObjRef) {
        self.rebuild_behaviour(class, Side::Instance);
        self.rebuild_behaviour(class, Side::Class);
        self.check_uninit(class);
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.update_sub_classes(sub);
        }
    }

    /// `~define`. The instance behaviours of `class` and of every subclass
    /// are updated in place, so the instances made before the call see the
    /// change as the ones made after it do (`r[class.methods.late]`), and
    /// so do the class objects whose metaclass is among them.
    pub fn define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        let kept = self.has_own_entry(class, name);
        self.note_dropped(class, name);
        let def = self.classes.get_mut(&class).expect("define: unknown class");
        def.own_instance_methods.replace_method(name, class, method);
        if name.eq_ignore_ascii_case("UNINIT") {
            def.has_uninit = true;
        }
        self.redefine(class, prior, kept, &[name], true);
    }

    /// [`ClassGraph::define`] of each of `methods` in turn, with one cascade instead of
    /// one each.
    ///
    /// The two differ only in what nothing can see: the behaviours the calls in between
    /// would have built, which nothing can read because nothing runs between the calls,
    /// and how often a rebuilt behaviour's version moves. Where that equivalence needs
    /// more than this checks -- a name given twice, a name `class` already defines,
    /// `UNINIT` -- the calls are made one by one.
    pub fn define_all(&mut self, class: ObjRef, methods: &[(String, MethodId)]) {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        let def = &self.classes[&class];
        let mut names: Vec<String> = methods
            .iter()
            .map(|(name, _)| name.to_ascii_uppercase())
            .collect();
        let together = methods.len() > 1
            && names
                .iter()
                .all(|name| name != "UNINIT" && !def.own_instance_methods.contains(name))
            && {
                names.sort_unstable();
                names.windows(2).all(|pair| pair[0] != pair[1])
            };
        if !together {
            for (name, method) in methods {
                self.define(class, name, *method);
            }
            return;
        }
        let def = self
            .classes
            .get_mut(&class)
            .expect("define_all: unknown class");
        for (name, method) in methods {
            def.own_instance_methods
                .replace_method(name, class, *method);
        }
        // Every name is new to the class, so the behaviours are rebuilt whole.
        self.redefine(class, prior, false, &[], true);
    }

    /// `~define` with the method argument omitted, and the bootstrap hiding of
    /// a method: a `.nil` tombstone under `name`.
    pub fn hide(&mut self, class: ObjRef, name: &str) {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        let kept = self.has_own_entry(class, name);
        self.note_dropped(class, name);
        self.classes
            .get_mut(&class)
            .expect("hide: unknown class")
            .own_instance_methods
            .hide_method(name);
        self.redefine(class, prior, kept, &[name], true);
    }

    /// `~define` with `.nil` as the method: [`ClassGraph::hide`] with the
    /// mark [`MethodSlot::Withdrawn`], which `Class~method` does not answer.
    pub fn withdraw(&mut self, class: ObjRef, name: &str) {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        let kept = self.has_own_entry(class, name);
        self.note_dropped(class, name);
        self.classes
            .get_mut(&class)
            .expect("withdraw: unknown class")
            .own_instance_methods
            .withdraw_method(name);
        self.redefine(class, prior, kept, &[name], true);
    }

    /// `~delete`, and the bootstrap removal of a method: take `name` out of
    /// `class`'s own instance dictionary. Answers whether there was anything to
    /// remove; only then is anything rebuilt.
    pub fn delete(&mut self, class: ObjRef, name: &str) -> bool {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        self.note_dropped(class, name);
        let removed = self
            .classes
            .get_mut(&class)
            .expect("delete: unknown class")
            .own_instance_methods
            .remove_method(name);
        // A name taken out can leave the others listed in another order than
        // a cascade without it makes, so the behaviours are rebuilt whole.
        self.redefine(class, prior, false, &[name], removed);
        removed
    }

    /// `~defineMethods`: a whole table of names in one mutation, which replaces one
    /// method per entry and cascades once for the whole table rather than once per
    /// name, so no send sees part of the table (`r[class.methods.whole]`).
    pub fn define_methods(&mut self, class: ObjRef, methods: &[(String, Option<MethodId>)]) {
        self.bump_epoch();
        let prior = self.inputs.changed(class);
        let kept = methods
            .iter()
            .all(|(name, _)| self.has_own_entry(class, name));
        for (name, _) in methods {
            self.note_dropped(class, name);
        }
        let def = self
            .classes
            .get_mut(&class)
            .expect("define_methods: unknown class");
        for (name, method) in methods {
            match method {
                Some(id) => {
                    def.own_instance_methods.replace_method(name, class, *id);
                    // The uninit check reads the flattened
                    // behaviour; this crate's `has_uninit` is the same fact
                    // asked of the name being installed, exactly as
                    // `define` above asks it.
                    if name.eq_ignore_ascii_case("UNINIT") {
                        def.has_uninit = true;
                    }
                }
                None => def.own_instance_methods.hide_method(name),
            }
        }
        let names: Vec<&str> = methods.iter().map(|(name, _)| name.as_str()).collect();
        self.redefine(class, prior, kept, &names, true);
    }

    /// What follows a change to the entries under `names` in `class`'s own
    /// instance dictionary, whose inputs had last changed at `prior`: the
    /// update in place of its instance behaviour and of every subclass's
    /// (only the instance side), each followed by the uninit check where one
    /// of `names` is `UNINIT`, the one change that can make it resolve, and
    /// then the class behaviours that follow them
    /// ([`Self::follow_metaclasses`]). Without `cascade` nothing changed, and
    /// nothing is updated.
    ///
    /// A behaviour [`Self::built_from_current`] clears has only `names`
    /// recomputed, which is all a rebuild would change of it, where `kept`
    /// says each of `names` was in the class's own dictionary before the
    /// change and still is: then every dictionary the cascade reads lists its
    /// names in the order it did, a rebuild would enter the names in the
    /// order it did, and the behaviour lists them, as `Class~methods` does,
    /// in the order a rebuild would. Any other is rebuilt whole.
    fn redefine(&mut self, class: ObjRef, prior: u64, kept: bool, names: &[&str], cascade: bool) {
        if !cascade {
            return;
        }
        self.inputs.touch(class);
        let uninit = names.iter().any(|name| name.eq_ignore_ascii_case("UNINIT"));
        // A table may name one method twice; each name is recomputed once.
        let mut keys: Vec<String> = Vec::new();
        if kept {
            keys = names.iter().map(|name| name.to_ascii_uppercase()).collect();
            keys.sort_unstable();
            keys.dedup();
        }
        let handle = self.classes[&class].instance_behaviour;
        if kept && self.built_from_current(handle, class, class, prior) {
            self.recompute_names(class, &keys);
        } else {
            self.rebuild_behaviour(class, Side::Instance);
        }
        if uninit {
            self.check_uninit(class);
        }
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.redefine_below(sub, class, prior, kept, &keys, uninit);
        }
        self.follow_metaclasses(class);
    }

    /// The class behaviours that merged an instance behaviour a change to
    /// `changed` rebuilt -- its own or a subclass's, where that class is a
    /// class object's metaclass -- rebuilt in their turn, so a class object
    /// follows every change to its metaclass and the classes behind it
    /// (`r[class.methods.late]`).
    fn follow_metaclasses(&mut self, changed: ObjRef) {
        let mut reached = vec![changed];
        let mut at = 0;
        while at < reached.len() {
            for &sub in &self.classes[&reached[at]].subclasses {
                if !reached.contains(&sub) {
                    reached.push(sub);
                }
            }
            at += 1;
        }
        let merged: Vec<ObjRef> = reached
            .into_iter()
            .filter(|&class| {
                self.classes
                    .values()
                    .any(|def| def.metaclass == class || (def.cloned && def.owning_class == class))
            })
            .collect();
        if merged.is_empty() {
            return;
        }
        let followers: Vec<ObjRef> = self
            .classes
            .iter()
            .filter(|(_, def)| {
                let dict = &self.behaviours[def.class_behaviour.index()].dict;
                merged.iter().any(|&class| dict.has_scope(class))
            })
            .map(|(&class, _)| class)
            .collect();
        for class in followers {
            self.rebuild_class_side(class);
            self.check_uninit(class);
        }
    }

    /// `class`'s class behaviour built again from what it was built from: a
    /// class `~new` made copies its metaclass's instance behaviour, the root
    /// class merges its metaclass's after its own cascade, as the bootstrap
    /// builds it ([`Self::bootstrap_root_class_behaviour`]), and every other
    /// class cascades.
    fn rebuild_class_side(&mut self, class: ObjRef) {
        let def = &self.classes[&class];
        let handle = def.class_behaviour;
        if def.cloned {
            let source = self.classes[&def.owning_class].instance_behaviour;
            let dict = self.behaviours[source.index()].dict.clone();
            let behaviour = &mut self.behaviours[handle.index()];
            behaviour.dict = dict;
            behaviour.version += 1;
            return;
        }
        let root = def.superclasses.is_empty();
        let metaclass = def.metaclass;
        self.rebuild_behaviour(class, Side::Class);
        if root {
            let meta_instance = self.classes[&metaclass].instance_behaviour;
            let meta_dict = self.behaviours[meta_instance.index()].dict.clone();
            self.behaviours[handle.index()].dict.merge(&meta_dict);
        }
    }

    /// Whether `class`'s own instance dictionary holds an entry under `name`,
    /// tombstone included.
    fn has_own_entry(&self, class: ObjRef, name: &str) -> bool {
        let own = &self.classes[&class].own_instance_methods;
        if name.bytes().any(|b| b.is_ascii_lowercase()) {
            own.has_entry(&name.to_ascii_uppercase())
        } else {
            own.has_entry(name)
        }
    }

    /// The update of a subclass's instance behaviour, and its subclasses',
    /// for [`Self::redefine`], each behaviour recomputed under `keys` alone where
    /// it was built from the inputs as they stood but for `changed`'s.
    fn redefine_below(
        &mut self,
        class: ObjRef,
        changed: ObjRef,
        prior: u64,
        kept: bool,
        keys: &[String],
        uninit: bool,
    ) {
        let handle = self.classes[&class].instance_behaviour;
        if kept && self.built_from_current(handle, class, changed, prior) {
            self.recompute_names(class, keys);
        } else {
            self.rebuild_behaviour(class, Side::Instance);
        }
        if uninit {
            self.check_uninit(class);
        }
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.redefine_below(sub, changed, prior, kept, keys, uninit);
        }
    }

    /// Whether `handle`, `class`'s instance behaviour, was built after every
    /// input of its cascade last changed, taking `changed`'s last change to
    /// be at `prior`: then it is the cascade of the inputs as they stood
    /// before `changed`'s latest change.
    fn built_from_current(
        &self,
        handle: BehaviourHandle,
        class: ObjRef,
        changed: ObjRef,
        prior: u64,
    ) -> bool {
        let built = self.inputs.built(handle);
        let mut pending = vec![class];
        while let Some(at) = pending.pop() {
            let last = if at == changed {
                prior
            } else {
                self.inputs.changed(at)
            };
            if last > built {
                return false;
            }
            pending.extend_from_slice(&self.classes[&at].superclasses);
        }
        true
    }

    /// The entries under `keys` of `class`'s instance behaviour made again
    /// by the cascade, every other name left as it stands, as a rebuild of a
    /// behaviour that differs from its inputs' cascade only under `keys`
    /// would make it.
    fn recompute_names(&mut self, class: ObjRef, keys: &[String]) {
        let mut made = MethodDict::new();
        Self::cascade_names(&self.classes, class, &mut made, keys);
        let handle = self.classes[&class].instance_behaviour;
        let behaviour = &mut self.behaviours[handle.index()];
        for key in keys {
            behaviour.dict.adopt_entry(&mut made, key);
        }
        behaviour.version += 1;
        self.inputs.set_built(handle);
        #[cfg(debug_assertions)]
        {
            let mut whole = MethodDict::new();
            whole.reserve(self.rebuilt_names(class, Side::Instance));
            Self::cascade_build(
                &self.classes,
                &self.behaviours,
                class,
                &mut whole,
                Side::Instance,
            );
            assert!(
                self.behaviours[handle.index()].dict.same_contents(&whole),
                "the instance behaviour of {class:?} recomputed under {keys:?} is not its cascade"
            );
        }
    }

    /// [`Self::cascade_build`] of the instance side for the names in `keys`
    /// alone: the same walk, folding in only those names, so that each
    /// comes out as the whole cascade would make it.
    fn cascade_names(
        classes: &NameMap<ObjRef, ClassDef>,
        class: ObjRef,
        target: &mut MethodDict,
        keys: &[String],
    ) {
        let def = &classes[&class];
        for &sup in def.superclasses.iter().rev() {
            if !target.has_scope(sup) {
                Self::cascade_names(classes, sup, target, keys);
            }
        }
        if !target.has_scope(class) {
            target.merge_methods_named(&def.own_instance_methods, keys);
            target.add_scope(class);
        }
    }

    /// Install a class (static) method directly, itself a "special method to
    /// allow a class method to be added to a primitive class during image
    /// build" (its own doc comment), and one of the two methods stripped before
    /// the image ships, alongside `~inheritInstanceMethods`. It adds directly
    /// to `class`'s *current* class behaviour and to its own class-method
    /// dictionary (`own_class_methods` here) -- **no clear, no
    /// rebuild-from-scratch, no cascade to subclasses**, unlike every other
    /// mutator in this file (the method is added directly and unconditionally
    /// to the already-built behaviour, with no rebuild). A class that already
    /// has subclasses when this is called would leave them unaware of the new
    /// method until their own class behaviour is next rebuilt for an unrelated
    /// reason -- that gap is consistent with the restriction to image build,
    /// before any subclass yet exists. This crate's own tests use it to seed a
    /// mixin's class-side method before any class `inherit`s it, which is
    /// exactly that restriction in miniature.
    pub fn class_define(&mut self, class: ObjRef, name: &str, method: MethodId) {
        self.bump_epoch();
        let handle = {
            let def = self
                .classes
                .get_mut(&class)
                .expect("class_define: unknown class");
            def.own_class_methods.add_method(name, class, method);
            def.class_behaviour
        };
        let behaviour = &mut self.behaviours[handle.index()];
        behaviour.dict.add_method(name, class, method);
        behaviour.version += 1;
    }

    /// `~inherit`: it appends to the superclass list and runs the full
    /// subclass update, which rebuilds the existing behaviour objects in
    /// place, so an instance created before this call sees the donated
    /// methods immediately, as it does after [`define`](Self::define).
    pub fn inherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.inherit_at(class, mixin, None)
    }

    /// `~inherit`'s own two-argument form: the same operation as
    /// [`Self::inherit`], with the optional `position` the `INHERIT` keyword
    /// of a `::CLASS` directive cannot supply.
    pub fn inherit_at(
        &mut self,
        class: ObjRef,
        mixin: ObjRef,
        position: Option<ObjRef>,
    ) -> Result<(), InheritRefusal> {
        self.bump_epoch();
        if !matches!(self.classes[&mixin].kind, ClassKind::Mixin) {
            return Err(InheritRefusal::NotAMixin);
        }
        if self.behaviour_has_scope(class, Side::Class, mixin) {
            return Err(InheritRefusal::Recursive);
        }
        if self.behaviour_has_scope(mixin, Side::Class, class) {
            return Err(InheritRefusal::Recursive);
        }
        let mixin_base = self.classes[&mixin].base_class;
        if !self.behaviour_has_scope(class, Side::Class, mixin_base) {
            return Err(InheritRefusal::BaseClass(mixin_base));
        }
        if !self.behaviour_has_scope(class, Side::Instance, mixin_base) {
            return Err(InheritRefusal::BaseClass(mixin_base));
        }
        let at = match position {
            None => self.classes[&class].superclasses.len(),
            Some(position) => {
                match self.classes[&class]
                    .superclasses
                    .iter()
                    .position(|&sup| sup == position)
                {
                    Some(at) => at,
                    None => return Err(InheritRefusal::NotInherited(position)),
                }
            }
        };
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .insert(at, mixin);
        self.inputs.touch(class);
        self.ancestry_changed();
        self.classes.get_mut(&mixin).unwrap().subclasses.push(class);
        self.update_sub_classes(class);
        self.follow_metaclasses(class);
        if self.uninit_reaches(mixin) {
            self.classes.get_mut(&class).unwrap().parent_has_uninit = true;
        }
        Ok(())
    }

    /// `~uninherit`: take a mixin back out of the superclass list --
    /// `~uninherit`.
    pub fn uninherit(&mut self, class: ObjRef, mixin: ObjRef) -> Result<(), InheritRefusal> {
        self.bump_epoch();
        if !matches!(self.classes[&mixin].kind, ClassKind::Mixin) {
            return Err(InheritRefusal::NotAMixin);
        }
        let at = self.classes[&class]
            .superclasses
            .iter()
            .position(|&sup| sup == mixin)
            .filter(|at| *at > 0);
        let Some(at) = at else {
            return Err(InheritRefusal::NotInherited(mixin));
        };
        self.classes
            .get_mut(&class)
            .unwrap()
            .superclasses
            .remove(at);
        self.inputs.touch(class);
        self.ancestry_changed();
        // The subclass-list removal takes out the one entry it finds, and a
        // class reaches a mixin's subclass list once: a second `~inherit` of
        // the same mixin is the recursive refusal above.
        if let Some(sub) = self.classes[&mixin]
            .subclasses
            .iter()
            .position(|&sub| sub == class)
        {
            self.classes.get_mut(&mixin).unwrap().subclasses.remove(sub);
        }
        self.update_sub_classes(class);
        self.follow_metaclasses(class);
        Ok(())
    }

    /// The bootstrap donation of a source's instance methods: copy the donor's
    /// own methods into this class's dictionary under this class's scope,
    /// **leaving the donor alone**.
    pub fn donate_instance_methods(&mut self, class: ObjRef, donor: ObjRef) {
        self.bump_epoch();
        let donor_methods = self
            .classes
            .get(&donor)
            .expect("donate_instance_methods: unknown donor class")
            .own_instance_methods
            .clone();
        self.classes
            .get_mut(&class)
            .expect("donate_instance_methods: unknown recipient class")
            .own_instance_methods
            .replace_methods_from(&donor_methods, donor, class);
        self.inputs.touch(class);
        self.rebuild_behaviour(class, Side::Instance);
    }

    /// `Class~inheritInstanceMethods` takes `donor`'s own instance-method
    /// dictionary **by pointer and rewrites it in place** (every method in it
    /// is rescoped to the recipient), folds it into `class`'s own instance
    /// dictionary, then rebuilds `class`'s instance behaviour in place -- **no
    /// superclass edge** (`class`'s `superclasses` is untouched) and **no
    /// cascade** to `class`'s own subclasses (unlike [`Self::define`] and
    /// [`Self::inherit`], it runs neither of the two subclass updates).
    pub fn inherit_instance_methods(&mut self, class: ObjRef, donor: ObjRef) {
        self.bump_epoch();
        let mut donor_methods = std::mem::take(
            &mut self
                .classes
                .get_mut(&donor)
                .expect("inherit_instance_methods: unknown donor class")
                .own_instance_methods,
        );
        donor_methods.set_method_scope(class);
        self.classes
            .get_mut(&class)
            .expect("inherit_instance_methods: unknown recipient class")
            .own_instance_methods
            .merge_methods(&donor_methods);
        self.classes.get_mut(&donor).unwrap().own_instance_methods = donor_methods;
        self.inputs.touch(donor);
        self.inputs.touch(class);
        self.rebuild_behaviour(class, Side::Instance);
    }

    /// The full subclass update of every subclass of `class`, leaving `class`
    /// itself alone: what a change to `class`'s own dictionary that did not
    /// cascade ([`Self::inherit_instance_methods`]) needs for subclasses that
    /// already exist to see it.
    pub fn update_subclasses_of(&mut self, class: ObjRef) {
        self.bump_epoch();
        let subclasses = self.classes[&class].subclasses.clone();
        for sub in subclasses {
            self.update_sub_classes(sub);
            self.follow_metaclasses(sub);
        }
    }

    /// The ancestor chain a class-graph assertion reads. Two classes can answer
    /// identically here despite [`Self::inherit_instance_methods`] having
    /// donated different methods to each: that is the exact blind spot this
    /// crate's tests are built to demonstrate.
    pub fn ancestors(&self, class: ObjRef) -> &[ObjRef] {
        &self.classes[&class].superclasses
    }

    /// Every other class `class` keeps alive: its superclasses (the
    /// `INHERIT`ed mixins among them), its metaclass, its base class and its
    /// owning class. The graph names them by handle, outside the heap, so the
    /// collector asks here when it marks a class object; a class reached only
    /// this way -- a superclass no variable holds -- would otherwise be freed
    /// under a live subclass. Its subclasses are not among them: that list is
    /// weak (see [`ClassGraph::expunge`]). Appends nothing for a class the
    /// graph does not know.
    pub fn class_references(&self, class: ObjRef, out: &mut Vec<ObjRef>) {
        let Some(def) = self.classes.get(&class) else {
            return;
        };
        out.extend(def.superclasses.iter().copied());
        out.extend(
            [def.metaclass, def.base_class, def.owning_class]
                .into_iter()
                .filter(|&held| held != class),
        );
    }

    /// Every class registered as a direct subclass or `INHERIT` recipient of
    /// `class`, what a cascade (`update_sub_classes` /
    /// `redefine`) walks. Exposed for assertions that need
    /// to see the cascade's own wiring, not just its effect.
    pub fn subclasses(&self, class: ObjRef) -> &[ObjRef] {
        &self.classes[&class].subclasses
    }

    /// `class`'s metaclass. `.Class`'s own entry answers itself.
    pub fn metaclass(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].metaclass
    }

    /// `class`'s own behaviour's owning class, and what `~class` answers for a
    /// class object. See the field for why this is not
    /// [`ClassGraph::metaclass`].
    pub fn owning_class(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].owning_class
    }

    /// Whether `class` may be named as another class's metaclass, and the flag
    /// `~subclass` refuses on. See the field for what sets it.
    pub fn is_metaclass(&self, class: ObjRef) -> bool {
        self.classes[&class].is_metaclass
    }

    /// `~queryMixinClass`, the same [`ClassKind`] `inherit`'s mixin check
    /// reads.
    pub fn is_mixin(&self, class: ObjRef) -> bool {
        matches!(self.classes[&class].kind, ClassKind::Mixin)
    }

    /// `~baseClass` of `.Class`. A `Regular` class answers itself; a `Mixin`
    /// answers whatever its `MIXINCLASS` target's own base class is, which is
    /// the value [`ClassGraph::inherit`]'s compatibility check reads.
    pub fn base_class(&self, class: ObjRef) -> ObjRef {
        self.classes[&class].base_class
    }

    /// Force a class-behaviour rebuild without adding a method -- an escape
    /// hatch [`ClassGraph::define`]/[`ClassGraph::class_define`] don't need
    /// and don't provide.
    pub fn refresh_class_behaviour(&mut self, class: ObjRef) {
        self.bump_epoch();
        self.rebuild_behaviour(class, Side::Class);
    }

    /// Take the entries `scope` defined under each of `names` out of `class`'s
    /// class behaviour, in place: what rebuilding it answers once `scope` no
    /// longer defines them, where nothing else changed since it was built.
    pub fn remove_from_class_behaviour(&mut self, class: ObjRef, scope: ObjRef, names: &[String]) {
        self.bump_epoch();
        let handle = self.behaviour_handle(class, Side::Class);
        let behaviour = &mut self.behaviours[handle.index()];
        for name in names {
            behaviour.dict.remove_scoped(name, scope);
        }
        behaviour.version += 1;
    }

    /// Whether `class`'s class behaviour holds what
    /// [`Self::refresh_class_behaviour`] would build now, followed, for `root`,
    /// by [`Self::bootstrap_root_class_behaviour`] with `metaclass`.
    pub fn class_behaviour_is_rebuilt(
        &self,
        class: ObjRef,
        root: ObjRef,
        metaclass: ObjRef,
    ) -> bool {
        let mut rebuilt = MethodDict::new();
        Self::cascade_build(
            &self.classes,
            &self.behaviours,
            class,
            &mut rebuilt,
            Side::Class,
        );
        if class == root {
            let meta_instance = self.classes[&metaclass].instance_behaviour;
            rebuilt.merge(&self.behaviours[meta_instance.index()].dict);
        }
        let handle = self.behaviour_handle(class, Side::Class);
        self.behaviours[handle.index()].dict.same_as(&rebuilt)
    }

    /// Merge `metaclass`'s *current* instance behaviour into `root`'s
    /// class-behaviour, bypassing [`ClassGraph::cascade_build`]'s
    /// `Side::Class` "is this the root" guard entirely.
    pub fn bootstrap_root_class_behaviour(&mut self, root: ObjRef, metaclass: ObjRef) {
        self.bump_epoch();
        let meta_instance = self.classes[&metaclass].instance_behaviour;
        let meta_dict = self.behaviours[meta_instance.index()].dict.clone();
        let handle = self.behaviour_handle(root, Side::Class);
        self.behaviours[handle.index()].dict.merge(&meta_dict);
        self.behaviours[handle.index()].version += 1;
    }

    /// Mark `class` a metaclass without deriving it from one: the final
    /// class-behaviour build marks `.Class` itself as a metaclass, which is
    /// where the chain [`ClassGraph::define_class`] propagates starts.
    pub fn bootstrap_metaclass(&mut self, class: ObjRef) {
        self.bump_epoch();
        self.classes
            .get_mut(&class)
            .expect("bootstrap_metaclass: unknown class")
            .is_metaclass = true;
    }

    /// `class`'s own, unflattened instance-method names, what
    /// `instance~instanceMethods(class)` answers for a genuine instance
    /// (scope-filtered, so ancestor-donated names are excluded) -- the
    /// exact-match target for a per-class derivation, as opposed to
    /// [`Self::method_names_at`]'s flattened, whole-ancestry set.
    pub fn own_instance_method_names(&self, class: ObjRef) -> BTreeSet<String> {
        self.classes[&class].own_instance_methods.method_names()
    }

    /// Whether `name` is in `class`'s own, unflattened instance-method
    /// dictionary -- the same set [`Self::own_instance_method_names`]
    /// enumerates, asked about one name so a caller does not build the set to
    /// throw it away.
    pub fn has_own_instance_method(&self, class: ObjRef, name: &str) -> bool {
        self.classes[&class].own_instance_methods.has_method(name)
    }

    /// What `name` holds in `class`'s own, unflattened instance-method
    /// dictionary, tombstone included -- what `~method` reads and where it
    /// parts from [`Self::has_own_instance_method`] beside it: a hidden name is
    /// `Some(MethodSlot::Hidden)` here and `false` there, and the two answer
    /// the same way round.
    pub fn own_instance_slot(&self, class: ObjRef, name: &str) -> Option<MethodSlot> {
        self.classes[&class].own_instance_methods.slot(name)
    }

    /// Whether a class other than `class` holds `method` in its own instance or class dictionary: a
    /// method object that instance-method donation shared, which is kept as a separate method
    /// object per class.
    pub fn held_elsewhere(&self, method: MethodId, class: ObjRef) -> bool {
        self.classes.iter().any(|(id, def)| {
            *id != class
                && (def
                    .own_instance_methods
                    .methods()
                    .any(|held| held == method)
                    || def.own_class_methods.methods().any(|held| held == method))
        })
    }

    /// Repoint `class`'s own instance entry `name` from `old` to `new`, and
    /// the same entry in every behaviour already built from it: each
    /// instance then dispatches `new`.
    pub fn repoint_instance_method(
        &mut self,
        class: ObjRef,
        name: &str,
        old: MethodId,
        new: MethodId,
    ) {
        self.bump_epoch();
        self.classes
            .get_mut(&class)
            .expect("repoint_instance_method: unknown class")
            .own_instance_methods
            .repoint(name, class, old, new);
        self.inputs.touch(class);
        for behaviour in &mut self.behaviours {
            if behaviour.dict.repoint(name, class, old, new) {
                behaviour.version += 1;
            }
        }
    }

    /// Whether the class is Rexx-defined; see [`ClassDef::rexx_defined`].
    pub fn is_rexx_defined(&self, class: ObjRef) -> bool {
        self.classes[&class].rexx_defined
    }

    /// The marking of a class as Rexx-defined, which image preparation applies to every class it
    /// walks. There is no clearing counterpart.
    pub fn set_rexx_defined(&mut self, class: ObjRef) {
        self.classes
            .get_mut(&class)
            .expect("set_rexx_defined: unknown class")
            .rexx_defined = true;
    }

    /// `class`'s own, unflattened class-method names, what
    /// `class~instanceMethods(class)` answers when sent to the class object
    /// itself (its own receiver behaviour is its class-behaviour, filtered to
    /// its own scope).
    pub fn own_class_method_names(&self, class: ObjRef) -> BTreeSet<String> {
        self.classes[&class].own_class_methods.method_names()
    }

    /// `class`'s current instance-behaviour handle -- what a freshly
    /// created instance would be given right now.
    pub fn instance_behaviour_handle(&self, class: ObjRef) -> BehaviourHandle {
        self.behaviour_handle(class, Side::Instance)
    }

    /// `class`'s current class-behaviour handle -- what the class object
    /// itself answers to right now.
    pub fn class_behaviour_handle(&self, class: ObjRef) -> BehaviourHandle {
        self.behaviour_handle(class, Side::Class)
    }

    /// `~hasMethod` against a specific handle, the one an instance holds.
    pub fn has_method_at(&self, handle: BehaviourHandle, name: &str) -> bool {
        self.behaviours[handle.index()].dict.has_method(name)
    }

    /// Every method name a specific handle answers to -- the method-SET
    /// assertion `ancestors` cannot substitute for.
    pub fn method_names_at(&self, handle: BehaviourHandle) -> BTreeSet<String> {
        self.behaviours[handle.index()].dict.method_names()
    }

    /// Every defined entry of a specific handle's dictionary, as
    /// [`MethodDict::defined_entries`] lists them.
    pub fn defined_entries_at(&self, handle: BehaviourHandle) -> Vec<(String, ObjRef, MethodId)> {
        self.behaviours[handle.index()]
            .dict
            .defined_entries()
            .map(|(name, scope, method)| (name.to_owned(), scope, method))
            .collect()
    }

    /// True once `scope` has been folded into a specific handle's dictionary
    /// -- what distinguishes [`MethodDict::merge`] (methods and scope
    /// history) from `merge_methods` alone (methods only): a build that
    /// replaced the former with the latter in `cascade_build`'s metaclass
    /// branch would still pass every `has_method_at`/`method_names_at`
    /// check (the donated *methods* still land) while this would go false
    /// for the metaclass's own scope specifically.
    pub fn has_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> bool {
        self.behaviours[handle.index()].dict.has_scope(scope)
    }

    /// An ordinary (unscoped) lookup against a specific handle.
    pub fn lookup_at(&self, handle: BehaviourHandle, name: &str) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.index()].dict.lookup(name)
    }

    /// A scope-override lookup against a specific handle -- see
    /// [`MethodDict::lookup_from_scope`].
    pub fn lookup_from_scope_at(
        &self,
        handle: BehaviourHandle,
        name: &str,
        start_scope: ObjRef,
    ) -> Option<(ObjRef, MethodId)> {
        self.behaviours[handle.index()]
            .dict
            .lookup_from_scope(name, start_scope)
    }

    /// The immediate superscope of `scope` within a specific handle's own
    /// ordering -- see [`MethodDict::resolve_super_scope`].
    pub fn resolve_super_scope_at(&self, handle: BehaviourHandle, scope: ObjRef) -> Option<ObjRef> {
        self.behaviours[handle.index()]
            .dict
            .resolve_super_scope(scope)
    }

    /// The monotonic version, bumped once per write to this handle's
    /// dictionary.
    pub fn version_at(&self, handle: BehaviourHandle) -> u64 {
        self.behaviours[handle.index()].version
    }

    /// `~hasMethod` against `class`'s *current* instance behaviour --
    /// what `.class~new~hasMethod(name)` would answer right now.
    pub fn has_method(&self, class: ObjRef, name: &str) -> bool {
        self.has_method_at(self.instance_behaviour_handle(class), name)
    }

    /// `~hasMethod` sent to the class object itself, against `class`'s
    /// current class behaviour.
    pub fn class_has_method(&self, class: ObjRef, name: &str) -> bool {
        self.has_method_at(self.class_behaviour_handle(class), name)
    }
}
