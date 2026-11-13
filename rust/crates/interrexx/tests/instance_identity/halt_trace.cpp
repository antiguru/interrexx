// The instance members `Halt` and `SetTrace` from a thread that runs no Rexx code of the
// interpreter. Built as the extension `halttrace`, which the programs load with
// `::requires 'halttrace' LIBRARY` or, part-way through a run, with `loadLibrary`. `HtSave`
// keeps the instance its thread context links, `HtHalt` and `HtTrace` call `Halt` and
// `SetTrace` through it from a thread of their own and wait for that thread. `HtThreadTrace`
// calls the thread context member `SetThreadTrace` on the thread of the call.
#include <pthread.h>
#include "oorexxapi.h"

static RexxInstance *saved = NULL;
static logical_t tracing = 0;

static void *halter(void *)
{
    saved->Halt();
    return NULL;
}

static void *tracer(void *)
{
    saved->SetTrace(tracing);
    return NULL;
}

static void run_on_thread(void *(*body)(void *))
{
    pthread_t thread;
    if (pthread_create(&thread, NULL, body, NULL) == 0)
    {
        pthread_join(thread, NULL);
    }
}

RexxRoutine0(int, HtSave)
{
    saved = context->threadContext->instance;
    return 0;
}

RexxRoutine0(int, HtHalt)
{
    run_on_thread(halter);
    return 0;
}

RexxRoutine1(int, HtTrace, int, on)
{
    tracing = on != 0;
    run_on_thread(tracer);
    return 0;
}

RexxRoutine1(int, HtThreadTrace, int, on)
{
    context->threadContext->SetThreadTrace(on != 0);
    return 0;
}

static RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(HtSave, HtSave),
    REXX_TYPED_ROUTINE(HtHalt, HtHalt),
    REXX_TYPED_ROUTINE(HtTrace, HtTrace),
    REXX_TYPED_ROUTINE(HtThreadTrace, HtThreadTrace),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry halttrace_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "halttrace",
    "1.0",
    NULL,
    NULL,
    routines,
    NULL
};

OOREXX_GET_PACKAGE(halttrace);

#ifdef DRIVER
#include <stdio.h>

// Built with -DDRIVER it is a program: it creates an interpreter, terminates it, and then calls
// `Halt`, `SetTrace(1)` and `SetTrace(0)` through the instance that has ended.
int main()
{
    setvbuf(stdout, NULL, _IONBF, 0);
    RexxInstance *instance;
    RexxThreadContext *thread;
    if (!RexxCreateInterpreter(&instance, &thread, NULL))
    {
        printf("create failed\n");
        return 1;
    }
    instance->Terminate();
    instance->Halt();
    instance->SetTrace(1);
    instance->SetTrace(0);
    printf("returned\n");
    return 0;
}
#endif
