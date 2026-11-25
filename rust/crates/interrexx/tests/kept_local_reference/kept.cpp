#include <stdint.h>
#include <stdio.h>
#include <time.h>
#include "oorexxapi.h"

// Handles kept from one native call for a later one, as extensions that
// cache a RexxObjectPtr in their own structures do.
static RexxObjectPtr anchored;  // a heap string the receiver also holds
static RexxObjectPtr loose;     // a heap string nothing else holds
static RexxObjectPtr word;      // a string short enough to live in its handle
static RexxObjectPtr number;    // a small integer

// Makes the strings, anchoring one in an object variable of the receiver.
RexxMethod0(RexxObjectPtr, KeepMake)
{
    anchored = context->String("anchored in an object variable");
    context->SetObjectVariable("KEPT", anchored);
    loose = context->String("held by nothing but the extension");
    word = context->String("short");
    number = context->WholeNumber(42);
    return context->String("made");
}

// Answers the anchored string through its kept handle.
RexxMethod0(RexxObjectPtr, KeepAnchored)
{
    return anchored;
}

// Sends LENGTH to the anchored string, using its kept handle as a receiver.
RexxMethod0(RexxObjectPtr, KeepLength)
{
    return context->SendMessage0(anchored, "LENGTH");
}

// Answers the short string and the number, joined by a blank.
RexxMethod0(RexxObjectPtr, KeepValues)
{
    RexxObjectPtr joined = context->SendMessage1(word, " ", number);
    return joined;
}

// Answers the string nothing else holds.
RexxMethod0(RexxObjectPtr, KeepLoose)
{
    return loose;
}

// The command environment KEPT: `stash ...` keeps the handle of the command
// string it was handed; `use` answers, as RC, that string's length through
// the kept handle, or `refused` where the handle names nothing.
static RexxObjectPtr command;

static RexxObjectPtr RexxEntry KeptHandler(RexxExitContext *context, RexxStringObject,
                                           RexxStringObject string)
{
    if (context->StringLength(string) > 4) {
        command = string;
        return context->String("stashed");
    }
    RexxObjectPtr length = context->SendMessage0(command, "LENGTH");
    context->ClearCondition();
    return length != NULLOBJECT ? length : context->String("refused");
}

// Adds the command environment KEPT.
RexxMethod0(RexxObjectPtr, KeepCommands)
{
    context->threadContext->AddCommandEnvironment("KEPT", (REXXPFN)KeptHandler,
                                                  DIRECT_COMMAND_ENVIRONMENT);
    return context->String("added");
}

// Answers the handle of a string it makes, as hexadecimal text.
RexxMethod0(RexxObjectPtr, KeepAddress)
{
    char text[32];
    snprintf(text, sizeof text, "%#lx",
             (unsigned long)(uintptr_t)context->String("a string long enough for the heap"));
    return context->String(text);
}

// Two interpreters running at once on two threads: each lends a string and
// keeps its handle in `mine`, one per thread; `KeepPublish` puts this
// thread's in `published` for the other; flags let them take turns.
static thread_local RexxObjectPtr mine;
static RexxObjectPtr published;
static volatile int flags[7];

// Makes a string long enough for the heap, keeps it alive in `.local` and
// keeps its handle in this thread's `mine`.
RexxMethod0(RexxObjectPtr, KeepLendMine)
{
    mine = context->String("a string long enough for the heap");
    context->DirectoryPut(context->GetLocalEnvironment(), mine, "KEPTMINE");
    return context->String("lent");
}

// Answers A to the first thread that asks and B to the second.
static int roles;
RexxMethod0(RexxObjectPtr, KeepRole)
{
    return context->String(__atomic_fetch_add(&roles, 1, __ATOMIC_SEQ_CST) == 0 ? "A" : "B");
}

// Puts this thread's kept handle where the other thread can read it.
RexxMethod0(RexxObjectPtr, KeepPublish)
{
    published = mine;
    return context->String("published");
}

// Sets flag n.
RexxMethod1(RexxObjectPtr, KeepSignal, int, n)
{
    __atomic_store_n(&flags[n], 1, __ATOMIC_SEQ_CST);
    return context->String("signalled");
}

// Waits, for at most 20 seconds, until flag n is set; answers 1 if it was.
RexxMethod1(int, KeepWait, int, n)
{
    for (int i = 0; i < 2000 && !__atomic_load_n(&flags[n], __ATOMIC_SEQ_CST); i++) {
        struct timespec pause = {0, 10000000};
        nanosleep(&pause, NULL);
    }
    return __atomic_load_n(&flags[n], __ATOMIC_SEQ_CST);
}

// Answers the length of the string a handle names, through this thread's
// kept handle (`mine`) or the other thread's (`published`), or `refused`.
RexxMethod1(RexxObjectPtr, KeepUse, CSTRING, which)
{
    RexxObjectPtr handle = which[0] == 'm' ? mine : published;
    RexxObjectPtr length = context->SendMessage0(handle, "LENGTH");
    context->ClearCondition();
    return length != NULLOBJECT ? length : context->String("refused");
}

RexxMethodEntry methods[] = {
    REXX_METHOD(KeepMake, KeepMake),
    REXX_METHOD(KeepAnchored, KeepAnchored),
    REXX_METHOD(KeepLength, KeepLength),
    REXX_METHOD(KeepValues, KeepValues),
    REXX_METHOD(KeepLoose, KeepLoose),
    REXX_METHOD(KeepCommands, KeepCommands),
    REXX_METHOD(KeepAddress, KeepAddress),
    REXX_METHOD(KeepLendMine, KeepLendMine),
    REXX_METHOD(KeepRole, KeepRole),
    REXX_METHOD(KeepPublish, KeepPublish),
    REXX_METHOD(KeepSignal, KeepSignal),
    REXX_METHOD(KeepWait, KeepWait),
    REXX_METHOD(KeepUse, KeepUse),
    REXX_LAST_METHOD()
};
RexxPackageEntry kept_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "kept", "1.0.0", NULL, NULL, NULL, methods
};
OOREXX_GET_PACKAGE(kept);
