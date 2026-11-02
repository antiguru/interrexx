**OPEN, queued 2026-11-02 by PR 0 of Phase 10. Owner to be ruled; the entry points refuse as `Phase 10 Embedding`.**

# `.CommandHandler` and its classes

r13346's image gains `CommandClasses.orx` (RFE #882): `CommandHandler`, `NativeCommandHandler`,
`CommandContext`, `CommandInput`, `CommandOutput`. This crate bootstraps the file and binds its seventeen
`LIBRARY REXX` entry points as deferred rows in `dispatch/native.rs`, family `command`: a hollow shell, loud at
every send. `corpus/gate-tables/methods/commandhandler__*.rex` rows are `hasMethod` readbacks and say nothing about
bodies. `base/class/CommandHandler.testGroup` (ooTest r13347) has no passing path until the bodies exist, and is
excluded from the derived concurrency list. RFE #881's `RexxCallBack` (`API/classic/RexxCallBack.testGroup`) and
`StackCommandEnvironment` are the native half.
