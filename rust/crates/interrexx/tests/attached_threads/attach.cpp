// Threads of the extension's own that attach to the interpreter and send messages from it. Built
// as the extension `attach`, which the programs beside it load with `::requires 'attach' LIBRARY`.
//
// Each routine that starts a thread answers at once, and the thread attaches through the instance
// the call's thread context links, sends its messages, detaches, and counts itself finished.
// `AtDone` answers how many threads have finished, which the programs wait for with `SysSleep`, so
// every thread has detached before the program ends. `AtReport` answers what the last thread that
// reports saw.
//
// * `AtTicker(target, count)` sends `TICK` to `target` `count` times.
// * `AtRaising(target)` sends `BOOM`, which raises a condition, checks and clears the condition
//   the send left, and sends `TICK`.
// * `AtLocal(target)` makes a string, sends `TICK`, then sends `SHOW` with that string as its
//   argument: a local reference made in one callback of the attachment and used in a later one.
// * `AtString(target)` takes `StringData` of a string it makes, sends `TICK`, and reports the
//   bytes it reads through that pointer afterwards.
// * `AtBuffer(target)` makes a buffer string, writes its bytes through `BufferStringData`,
//   finishes it, and sends `SHOW` with it as its argument.
// * `AtJoin()` starts a thread that attaches and detaches, joins it while the call runs, and
//   answers whether it attached.
#include <atomic>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"

enum Job
{
    TICKER,
    RAISING,
    LOCAL,
    STRING_DATA,
    BUFFER_STRING
};

struct Thread
{
    RexxInstance *instance;
    RexxObjectPtr target;
    Job job;
    size_t count;
};

static std::atomic<int> finished(0);
static char report[128] = "";

static void run(RexxThreadContext *thread, Thread *self)
{
    switch (self->job)
    {
        case TICKER:
            for (size_t i = 0; i < self->count; i++)
            {
                thread->SendMessage0(self->target, "TICK");
            }
            break;
        case RAISING:
        {
            thread->SendMessage0(self->target, "BOOM");
            int checked = thread->CheckCondition() ? 1 : 0;
            thread->ClearCondition();
            int cleared = thread->CheckCondition() ? 1 : 0;
            thread->SendMessage0(self->target, "TICK");
            snprintf(report, sizeof report, "checked %d cleared %d", checked, cleared);
            break;
        }
        case LOCAL:
        {
            RexxStringObject kept = thread->NewStringFromAsciiz("kept");
            thread->SendMessage0(self->target, "TICK");
            thread->SendMessage1(self->target, "SHOW", kept);
            break;
        }
        case STRING_DATA:
        {
            CSTRING data = thread->StringData(thread->NewStringFromAsciiz("abc"));
            thread->SendMessage0(self->target, "TICK");
            snprintf(report, sizeof report, "string %s", data != NULL ? data : "null");
            break;
        }
        case BUFFER_STRING:
        {
            RexxBufferStringObject buffer = thread->NewBufferString(4);
            POINTER data = thread->BufferStringData(buffer);
            if (data == NULL)
            {
                snprintf(report, sizeof report, "buffer null");
                break;
            }
            memcpy(data, "abcd", 4);
            RexxStringObject finished = thread->FinishBufferString(buffer, 3);
            thread->SendMessage1(self->target, "SHOW", finished);
            snprintf(report, sizeof report, "buffer %zu", thread->BufferStringLength(buffer));
            break;
        }
    }
}

static void *body(void *argument)
{
    Thread *self = (Thread *)argument;
    RexxThreadContext *thread = NULL;
    if (self->instance->AttachThread(&thread))
    {
        run(thread, self);
        thread->ReleaseGlobalReference(self->target);
        thread->DetachThread();
    }
    delete self;
    finished++;
    return NULL;
}

// Starts a detached thread doing `job`, and answers 1 where it started.
static logical_t start(RexxCallContext *context, RexxObjectPtr target, Job job, size_t count)
{
    Thread *self = new Thread;
    self->instance = context->threadContext->instance;
    self->target = context->RequestGlobalReference(target);
    self->job = job;
    self->count = count;
    pthread_t thread;
    pthread_attr_t attributes;
    pthread_attr_init(&attributes);
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    int started = pthread_create(&thread, &attributes, body, self);
    pthread_attr_destroy(&attributes);
    if (started != 0)
    {
        context->ReleaseGlobalReference(self->target);
        delete self;
        return 0;
    }
    return 1;
}

RexxRoutine2(logical_t, AtTicker, RexxObjectPtr, target, positive_wholenumber_t, count)
{
    return start(context, target, TICKER, (size_t)count);
}

RexxRoutine1(logical_t, AtRaising, RexxObjectPtr, target)
{
    return start(context, target, RAISING, 0);
}

RexxRoutine1(logical_t, AtLocal, RexxObjectPtr, target)
{
    return start(context, target, LOCAL, 0);
}

RexxRoutine1(logical_t, AtString, RexxObjectPtr, target)
{
    return start(context, target, STRING_DATA, 0);
}

RexxRoutine1(logical_t, AtBuffer, RexxObjectPtr, target)
{
    return start(context, target, BUFFER_STRING, 0);
}

static void *attach_and_detach(void *argument)
{
    RexxThreadContext *thread = NULL;
    logical_t attached = ((RexxInstance *)argument)->AttachThread(&thread);
    if (attached)
    {
        thread->DetachThread();
    }
    return (void *)(intptr_t)attached;
}

RexxRoutine0(int, AtJoin)
{
    pthread_t thread;
    if (pthread_create(&thread, NULL, attach_and_detach, context->threadContext->instance) != 0)
    {
        return -1;
    }
    void *attached = NULL;
    pthread_join(thread, &attached);
    return (int)(intptr_t)attached;
}

RexxRoutine0(int, AtDone)
{
    return finished;
}

RexxRoutine0(CSTRING, AtReport)
{
    return report;
}

static RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(AtTicker, AtTicker),
    REXX_TYPED_ROUTINE(AtRaising, AtRaising),
    REXX_TYPED_ROUTINE(AtLocal, AtLocal),
    REXX_TYPED_ROUTINE(AtString, AtString),
    REXX_TYPED_ROUTINE(AtBuffer, AtBuffer),
    REXX_TYPED_ROUTINE(AtJoin, AtJoin),
    REXX_TYPED_ROUTINE(AtDone, AtDone),
    REXX_TYPED_ROUTINE(AtReport, AtReport),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry attach_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "attach",
    "1.0",
    NULL,
    NULL,
    routines,
    NULL
};

OOREXX_GET_PACKAGE(attach);
