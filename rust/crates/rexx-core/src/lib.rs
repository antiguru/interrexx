//! The Rexx object model: handles, the arena heap, tracing, and behaviours.

mod behaviour;
mod body;
// **An `unsafe` exception.** The workspace lint is `deny` rather than `forbid` so that this
// line can exist at all -- the root `Cargo.toml` has why.
// `bytes.rs`'s own `Bytes` doc states the invariant, names the only writer and
// the only reader, and gives the measurement that bought it;
// `tests/unsafe_sites.rs` is what stops a second site appearing quietly.
#[allow(unsafe_code)]
mod bytes;
mod frame;
mod handle;
mod heap;
mod roots;
mod slots;

pub use behaviour::BehaviourTable;
pub use body::{
    AlignedBytes, BehaviourHandle, BehaviourId, Body, BufferBytes, BufferState, ExtensionFields,
    ExtensionState, MethodId, NativeObject, NativeState, NotNumeric, Object, ObjectMethod,
    ObjectMethods, OpenFile, OpenMode, PackageFields, PackageState, SLOT_BYTES, STREAM_BUFFER,
    ScopePools, StandardStream, StreamState, StreamStatus, VarRef, VarRefHome, pointer_to_string,
};
pub use bytes::{Bytes, INLINE_BYTES};
pub use frame::{FrameArena, FrameBlock, ParkedFrame, RegFrame};
pub use handle::{
    Decoded, GENERATION_MAX, INLINE_TEXT, InlineText, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN,
};
#[cfg(feature = "sharing")]
pub use heap::SharingCount;
pub use heap::{CollectStats, Heap};
pub use roots::{ActivityRoots, FrameAliases, FrameId, Parked, RootSet, SlotFrame, SlotRef};
pub use slots::ArraySlots;

/// The hasher behind [`NameMap`].
pub type NameHasher = rustc_hash::FxBuildHasher;

/// A map keyed by a name out of the program text.
pub type NameMap<K, V> = std::collections::HashMap<K, V, NameHasher>;
