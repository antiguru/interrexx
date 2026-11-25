// Handles kept across native calls, in two interpreter instances of one process. Built as the
// extension `keptacross`, it has two routines:
//   Lend(n)     makes a string long enough for the heap, keeps it alive in `.local` and keeps its
//               handle in kept slot n (1 or 2).
//   UseKept(n)  sends LENGTH through the handle in kept slot n, in a later call, and answers the
//               length, or `refused` where the handle names nothing.
//
// Built with -DDRIVER it is also a program, which does not link the extension: it creates
// instances A and B, each running `kept_across.rex` with one argument. A runs it with `A`, which
// lends into slot 1. B runs it with `B`, which lends into slot 2 and then uses slot 2, its own
// handle, and slot 1, A's. Both instances lay their heaps out alike, so A's object and B's sit in
// the same slot at the same generation: only the interpreter's key keeps A's handle from naming
// B's object.
#include <stdio.h>
#include "oorexxapi.h"

static RexxObjectPtr kept[3];

RexxRoutine1(int, Lend, int, n)
{
    RexxObjectPtr s = context->String("a string long enough for the heap");
    context->DirectoryPut(context->GetLocalEnvironment(), s, "KEPTACROSS");
    kept[n] = s;
    return 0;
}

RexxRoutine1(RexxObjectPtr, UseKept, int, n)
{
    RexxObjectPtr length = context->SendMessage0(kept[n], "LENGTH");
    context->ClearCondition();
    return length != NULLOBJECT ? length : context->String("refused");
}

RexxRoutineEntry keptacross_routines[] = {
    REXX_TYPED_ROUTINE(Lend, Lend),
    REXX_TYPED_ROUTINE(UseKept, UseKept),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry keptacross_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "keptacross", "1.0.0", NULL, NULL, keptacross_routines, NULL
};
OOREXX_GET_PACKAGE(keptacross);

#ifdef DRIVER
int main()
{
    RexxInstance *a, *b;
    RexxThreadContext *ta, *tb;
    if (!RexxCreateInterpreter(&a, &ta, NULL) || !RexxCreateInterpreter(&b, &tb, NULL)) {
        return 1;
    }
    ta->CallProgram("kept_across.rex", ta->ArrayOfOne(ta->String("A")));
    fflush(stdout);
    tb->CallProgram("kept_across.rex", tb->ArrayOfOne(tb->String("B")));
    fflush(stdout);
    a->Terminate();
    b->Terminate();
    return 0;
}
#endif
