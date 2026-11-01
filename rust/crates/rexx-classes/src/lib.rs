//! The ooRexx class/method-dictionary model.

mod class_graph;
mod method_dict;
mod native_classes;
mod registry;

pub use class_graph::{ClassGraph, ClassKind, InheritRefusal};
pub use method_dict::{MethodDict, MethodSlot};
pub use native_classes::{
    Deferral, deferred_classes, native_classes, native_classes_for_bootstrap, remove_setup_methods,
    setup_class_names, setup_method_names,
};
pub use registry::ClassRegistry;
pub use rexx_core::{BehaviourHandle, MethodId};
