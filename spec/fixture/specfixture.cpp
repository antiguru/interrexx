// The native libraries the examples of the native chapter call (`spec/src/native/`).
//
// One source, compiled three ways by the spec runner, against the public API headers only. It
// links nothing from an interpreter: every call into the interpreter goes through the context
// handed in, so the same file loads into either interpreter the examples are compared on.
//
//   SPEC_FIXTURE_PACKAGE  libspecfixture.so: a package entry with routines, methods and a loader.
//   SPEC_FIXTURE_CLASSIC  libspecclassic.so: classic function handlers only, no package entry.
//   SPEC_FIXTURE_NEWER    libspecnewer.so:   a package entry that asks for a newer interpreter.
//
// The entries are described, by their Rexx-side behaviour, in `spec/src/native/index.md`. No
// entry keeps state of its own between calls: an interpreter may load the library once for
// several programs.

#include "oorexxapi.h"

#include <pthread.h>
#include <stdio.h>
#include <string.h>

// ---------------------------------------------------------------------------------------------
// Classic function handlers: in every variant, so that the package can list one as a classic
// row and the classic library can be registered by name.

// Answers `NAME:ARGC:a1,a2,...`, with an omitted argument shown as `()`.
// Serves r[native.loading.classic-call].
extern "C" size_t RexxEntry FxClassicJoin(const char *name, size_t argc, CONSTRXSTRING *argv,
                                          const char *, RXSTRING *result)
{
    size_t room = result->strlength;
    int used = snprintf(result->strptr, room, "%s:%zu:", name, argc);
    for (size_t i = 0; i < argc && used >= 0 && (size_t)used < room; i++)
    {
        const char *sep = i == 0 ? "" : ",";
        if (argv[i].strptr == NULL)
        {
            used += snprintf(result->strptr + used, room - used, "%s()", sep);
        }
        else
        {
            used += snprintf(result->strptr + used, room - used, "%s%.*s", sep,
                             (int)argv[i].strlength, argv[i].strptr);
        }
    }
    result->strlength = (used < 0 || (size_t)used >= room) ? room - 1 : (size_t)used;
    return 0;
}

// The same handler under a name written in capitals only.
// Serves r[native.loading.classic].
extern "C" size_t RexxEntry FXCLASSICUPPER(const char *name, size_t argc, CONSTRXSTRING *argv,
                                           const char *queue, RXSTRING *result)
{
    return FxClassicJoin(name, argc, argv, queue, result);
}

// Refuses the call: a handler's non-zero answer.
// Serves r[native.loading.classic-call].
extern "C" size_t RexxEntry FxClassicFail(const char *, size_t, CONSTRXSTRING *, const char *,
                                          RXSTRING *)
{
    return 1;
}

// Answers no result: a handler that leaves no string.
// Serves r[native.loading.classic-call].
extern "C" size_t RexxEntry FxClassicNone(const char *, size_t, CONSTRXSTRING *, const char *,
                                          RXSTRING *result)
{
    result->strptr = NULL;
    result->strlength = 0;
    return 0;
}

#if defined(SPEC_FIXTURE_PACKAGE)

// ---------------------------------------------------------------------------------------------
// Typed routines.

// Serves r[native.loading.package].
RexxRoutine0(CSTRING, FxHello)
{
    return "hello from specfixture";
}

// Serves r[native.calls.result].
RexxRoutine1(RexxObjectPtr, FxEcho, RexxObjectPtr, value)
{
    return value;
}

// Serves r[native.calls.conversion].
RexxRoutine1(CSTRING, FxString, CSTRING, value)
{
    return value;
}

// Serves r[native.calls.conversion].
RexxRoutine2(int32_t, FxAdd, int32_t, a, int32_t, b)
{
    return a + b;
}

// Serves r[native.calls.conversion].
RexxRoutine1(wholenumber_t, FxPositive, positive_wholenumber_t, n)
{
    return n;
}

// Serves r[native.calls.conversion].
RexxRoutine1(wholenumber_t, FxNonNegative, nonnegative_wholenumber_t, n)
{
    return n;
}

// Serves r[native.calls.conversion].
RexxRoutine1(double, FxDouble, double, d)
{
    return d;
}

// Serves r[native.calls.conversion].
RexxRoutine1(logical_t, FxLogical, logical_t, b)
{
    return b;
}

// Serves r[native.calls.conversion].
RexxRoutine1(size_t, FxItems, RexxArrayObject, a)
{
    return context->ArrayItems(a);
}

// Answers the argument, or `omitted` where the caller left it out.
// Serves r[native.calls.count].
RexxRoutine1(RexxObjectPtr, FxOptional, OPTIONAL_int32_t, n)
{
    if (argumentOmitted(1))
    {
        return context->String("omitted");
    }
    return context->Int32ToObject(n);
}

// Answers `SIZE ITEMS` of the argument array.
// Serves r[native.calls.arglist].
RexxRoutine1(RexxObjectPtr, FxArgs, ARGLIST, args)
{
    char text[64];
    snprintf(text, sizeof text, "%zu %zu", context->ArraySize(args), context->ArrayItems(args));
    return context->String(text);
}

// Answers the name the routine was called by.
// Serves r[native.calls.name].
RexxRoutine1(CSTRING, FxName, NAME, name)
{
    return name;
}

// Answers no result.
// Serves r[native.calls.result].
RexxRoutine0(RexxObjectPtr, FxNothing)
{
    return NULLOBJECT;
}

// Raises SYNTAX 40.900 with `message`, then sets the caller's variable `AFTER` and answers
// `returned`.
// Serves r[native.calls.raise].
RexxRoutine1(CSTRING, FxRaise, CSTRING, message)
{
    context->RaiseException1(Rexx_Error_Incorrect_call_user_defined, context->String(message));
    context->SetContextVariable("AFTER", context->String("ran on"));
    return "returned";
}

// Raises the condition `name` with the description `description` and the extra object `extra`,
// then answers `returned`.
// Serves r[native.calls.condition].
RexxRoutine3(CSTRING, FxCondition, CSTRING, name, CSTRING, description, RexxObjectPtr, extra)
{
    context->RaiseCondition(name, context->String(description), extra, NULLOBJECT);
    return "returned";
}

// Sends `message` to `target` and answers its result, or `no result`; a condition the message
// raised is cleared and answered as `condition` followed by its code.
// Serves r[native.calls.send].
RexxRoutine2(RexxObjectPtr, FxSend, RexxObjectPtr, target, CSTRING, message)
{
    RexxObjectPtr answer = context->SendMessage0(target, message);
    if (context->CheckCondition())
    {
        RexxDirectoryObject info = context->GetConditionInfo();
        RexxObjectPtr code = context->DirectoryAt(info, "CODE");
        context->ClearCondition();
        char text[64];
        snprintf(text, sizeof text, "condition %s",
                 code == NULLOBJECT ? "?" : context->ObjectToStringValue(code));
        return context->String(text);
    }
    return answer == NULLOBJECT ? context->String("no result") : answer;
}

// The caller's variable pool.
// Serves r[native.calls.variables].
RexxRoutine2(RexxObjectPtr, FxSetVar, CSTRING, name, RexxObjectPtr, value)
{
    context->SetContextVariable(name, value);
    return NULLOBJECT;
}

// Serves r[native.calls.variables].
RexxRoutine1(RexxObjectPtr, FxGetVar, CSTRING, name)
{
    RexxObjectPtr value = context->GetContextVariable(name);
    return value == NULLOBJECT ? context->String("(unset)") : value;
}

// Serves r[native.calls.variables].
RexxRoutine1(RexxObjectPtr, FxDropVar, CSTRING, name)
{
    context->DropContextVariable(name);
    return NULLOBJECT;
}

// Answers `DIGITS FUZZ FORM` of the caller.
// Serves r[native.calls.numeric].
RexxRoutine0(RexxObjectPtr, FxNumeric)
{
    char text[64];
    snprintf(text, sizeof text, "%zu %zu %s", context->GetContextDigits(),
             context->GetContextFuzz(), context->GetContextForm() ? "ENGINEERING" : "SCIENTIFIC");
    return context->String(text);
}

// Two rows of names that differ only in case (see the package table below).
// Serves r[native.loading.table].
RexxRoutine0(CSTRING, FxTwinFirst)
{
    return "first row";
}

// Serves r[native.loading.table].
RexxRoutine0(CSTRING, FxTwinSecond)
{
    return "second row";
}

// ---------------------------------------------------------------------------------------------
// Threads.

// A job for a thread the fixture starts: send `message` to `target` from an activity of its own.
struct ThreadJob
{
    RexxInstance *instance;
    RexxObjectPtr target;
    char message[64];
};

static void *runJob(void *arg)
{
    ThreadJob *job = (ThreadJob *)arg;
    RexxThreadContext *thread;
    if (job->instance->AttachThread(&thread))
    {
        thread->SendMessage0(job->target, job->message);
        thread->ReleaseGlobalReference(job->target);
        thread->DetachThread();
    }
    delete job;
    return NULL;
}

// Starts a thread that attaches to the interpreter, sends `message` to `target`, and detaches.
// Answers at once, without waiting for the thread: 1 where it started, 0 otherwise.
// Serves r[native.threads.attach].
RexxRoutine2(logical_t, FxOnThread, RexxObjectPtr, target, CSTRING, message)
{
    ThreadJob *job = new ThreadJob;
    job->instance = context->threadContext->instance;
    job->target = context->RequestGlobalReference(target);
    snprintf(job->message, sizeof job->message, "%s", message);
    pthread_t thread;
    pthread_attr_t attributes;
    pthread_attr_init(&attributes);
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    int started = pthread_create(&thread, &attributes, runJob, job);
    pthread_attr_destroy(&attributes);
    if (started != 0)
    {
        context->ReleaseGlobalReference(job->target);
        delete job;
        return 0;
    }
    return 1;
}

// Asks the interpreter to halt every activity.
// Serves r[native.threads.halt].
RexxRoutine0(RexxObjectPtr, FxHaltAll)
{
    context->threadContext->instance->Halt();
    return NULLOBJECT;
}

// Asks the interpreter to halt the calling activity.
// Serves r[native.threads.halt].
RexxRoutine0(RexxObjectPtr, FxHaltThis)
{
    context->threadContext->HaltThread();
    return NULLOBJECT;
}

// ---------------------------------------------------------------------------------------------
// Methods.

// Serves r[native.calls.method].
RexxMethod1(RexxObjectPtr, FxSelf, OSELF, self)
{
    return self;
}

// Serves r[native.calls.method].
RexxMethod1(CSTRING, FxMessage, NAME, name)
{
    return name;
}

// Serves r[native.calls.method].
RexxMethod1(RexxObjectPtr, FxScope, SCOPE, scope)
{
    return scope;
}

// Serves r[native.calls.method].
RexxMethod2(RexxObjectPtr, FxObjSet, CSTRING, name, RexxObjectPtr, value)
{
    context->SetObjectVariable(name, value);
    return NULLOBJECT;
}

// Serves r[native.calls.method].
RexxMethod1(RexxObjectPtr, FxObjGet, CSTRING, name)
{
    RexxObjectPtr value = context->GetObjectVariable(name);
    return value == NULLOBJECT ? context->String("(unset)") : value;
}

// Serves r[native.calls.raise].
RexxMethod1(CSTRING, FxMethodRaise, CSTRING, message)
{
    context->RaiseException1(Rexx_Error_Incorrect_method_user_defined, context->String(message));
    return "returned";
}

// ---------------------------------------------------------------------------------------------
// The package.

// Counts, in `.local`'s entry `SPECFIXTURE.LOADS`, how often the loader ran in this interpreter.
// Serves r[native.loading.once].
static void RexxEntry fixtureLoader(RexxThreadContext *context)
{
    RexxDirectoryObject local = context->GetLocalEnvironment();
    RexxObjectPtr previous = context->DirectoryAt(local, "SPECFIXTURE.LOADS");
    wholenumber_t count = 0;
    if (previous != NULLOBJECT)
    {
        context->ObjectToWholeNumber(previous, &count);
    }
    context->DirectoryPut(local, context->WholeNumberToObject(count + 1), "SPECFIXTURE.LOADS");
}

static RexxRoutineEntry fixtureRoutines[] =
{
    REXX_TYPED_ROUTINE(FxHello, FxHello),
    REXX_TYPED_ROUTINE(FxEcho, FxEcho),
    REXX_TYPED_ROUTINE(FxString, FxString),
    REXX_TYPED_ROUTINE(FxAdd, FxAdd),
    REXX_TYPED_ROUTINE(FxPositive, FxPositive),
    REXX_TYPED_ROUTINE(FxNonNegative, FxNonNegative),
    REXX_TYPED_ROUTINE(FxDouble, FxDouble),
    REXX_TYPED_ROUTINE(FxLogical, FxLogical),
    REXX_TYPED_ROUTINE(FxItems, FxItems),
    REXX_TYPED_ROUTINE(FxOptional, FxOptional),
    REXX_TYPED_ROUTINE(FxArgs, FxArgs),
    REXX_TYPED_ROUTINE(FxName, FxName),
    REXX_TYPED_ROUTINE(FxNothing, FxNothing),
    REXX_TYPED_ROUTINE(FxRaise, FxRaise),
    REXX_TYPED_ROUTINE(FxCondition, FxCondition),
    REXX_TYPED_ROUTINE(FxSend, FxSend),
    REXX_TYPED_ROUTINE(FxSetVar, FxSetVar),
    REXX_TYPED_ROUTINE(FxGetVar, FxGetVar),
    REXX_TYPED_ROUTINE(FxDropVar, FxDropVar),
    REXX_TYPED_ROUTINE(FxNumeric, FxNumeric),
    REXX_TYPED_ROUTINE(FxOnThread, FxOnThread),
    REXX_TYPED_ROUTINE(FxHaltAll, FxHaltAll),
    REXX_TYPED_ROUTINE(FxHaltThis, FxHaltThis),
    REXX_CLASSIC_ROUTINE(FxClassicJoin, FxClassicJoin),
    REXX_CLASSIC_ROUTINE(FxClassicFail, FxClassicFail),
    REXX_CLASSIC_ROUTINE(FxClassicNone, FxClassicNone),
    // Two rows whose names differ only in case.
    REXX_TYPED_ROUTINE(FxTwin, FxTwinFirst),
    REXX_TYPED_ROUTINE(FXTWIN, FxTwinSecond),
    REXX_LAST_ROUTINE()
};

static RexxMethodEntry fixtureMethods[] =
{
    REXX_METHOD(FxSelf, FxSelf),
    REXX_METHOD(FxMessage, FxMessage),
    REXX_METHOD(FxScope, FxScope),
    REXX_METHOD(FxObjSet, FxObjSet),
    REXX_METHOD(FxObjGet, FxObjGet),
    REXX_METHOD(FxMethodRaise, FxMethodRaise),
    REXX_LAST_METHOD()
};

static RexxPackageEntry fixture_package_entry =
{
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "specfixture",
    "1.0.0",
    fixtureLoader,
    NULL,
    fixtureRoutines,
    fixtureMethods
};

OOREXX_GET_PACKAGE(fixture);

#elif defined(SPEC_FIXTURE_NEWER)

// Serves r[native.loading.version].
RexxRoutine0(CSTRING, FxNewer)
{
    return "newer";
}

static RexxRoutineEntry newerRoutines[] =
{
    REXX_TYPED_ROUTINE(FxNewer, FxNewer),
    REXX_LAST_ROUTINE()
};

// Asks for an interpreter version above any there is: r[native.loading.version].
static RexxPackageEntry newer_package_entry =
{
    STANDARD_PACKAGE_HEADER
    0x7f000000,
    "specnewer",
    "1.0.0",
    NULL,
    NULL,
    newerRoutines,
    NULL
};

OOREXX_GET_PACKAGE(newer);

#elif !defined(SPEC_FIXTURE_CLASSIC)
#error "define one of SPEC_FIXTURE_PACKAGE, SPEC_FIXTURE_CLASSIC or SPEC_FIXTURE_NEWER"
#endif
