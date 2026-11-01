**OPEN, Moritz 2026-11-01, unscheduled.** Replace the embedded CPL Rexx library with our own.

`rexx-lib` embeds `CoreClasses.orx`, `StreamClasses.orx` and `PlatformObjects.orx`, so built binaries contain CPL code while the crate is MIT OR Apache-2.0. Moritz: "We might have to re-write CoreClasses, StreamClasses ourselves."

Write from `oodocs` and ooTest behaviour, not by reading the `.orx` files, or the result stays derivative. Rust-native bodies are an alternative to new Rexx source. Size it before scheduling. Generated tables from `Setup.cpp`, `BuiltinFunctions.cpp` and `rexxmsg.xml` need the same question answered.
