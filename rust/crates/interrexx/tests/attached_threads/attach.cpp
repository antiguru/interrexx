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
// * `AtIdle(target)` sends `TICK`, then stays attached, sending nothing, for two seconds before it
//   detaches.
// * `AtSlow(target)` sends `SLOW`, which runs past the main program's end, and detaches.
// * `AtCycles(target, count)` attaches, sends `TICK` and detaches `count` times, and reports
//   whether the thread contexts its attaches answered stayed at two or fewer distinct ones.
// * `AtWrong(target)` sends `TICK`, has another thread call `DetachThread` on its context, sends
//   `TICK` again, detaches twice, and reports what each step answered.
// * `AtLate(target)` sends `TICK`, detaches, and sends `TICK` once more through the context it
//   detached from, reporting whether that answered nothing.
// * `AtInner(target)` sends `INNER`, whose method calls `AtDetachHere`, which calls `DetachThread`
//   through its own call's thread context, the attachment's, while the send is in flight. Then it
//   attaches again, reports whether that attach answered the same context, sends `TICK` and
//   detaches.
// * `AtBuffers(target, count)` makes a buffer string of 100000 bytes, writes and finishes it, and
//   releases it, `count` times on one attachment, and reports how many times every step answered
//   as it should.
// * `AtTickJoin(target)` starts a thread that attaches, sends `TICK` to `target` and detaches, and
//   joins it before it returns: the message runs while the call blocks.
// * `AtSendJoin(target)` sends `JOIN`, whose method calls `AtTickJoin` from inside the attached
//   thread's message, so the call that joins is made while a callback runs.
#include <atomic>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include "oorexxapi.h"

enum Job
{
    TICKER,
    RAISING,
    LOCAL,
    STRING_DATA,
    BUFFER_STRING,
    IDLE,
    SLOW,
    WRONG,
    LATE,
    BUFFERS,
    INNER,
    JOIN
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
        case IDLE:
            thread->SendMessage0(self->target, "TICK");
            sleep(2);
            break;
        case SLOW:
            thread->SendMessage0(self->target, "SLOW");
            break;
        case JOIN:
            thread->SendMessage0(self->target, "JOIN");
            break;
        case BUFFERS:
        {
            size_t good = 0;
            for (size_t i = 0; i < self->count; i++)
            {
                size_t length = 100000 - i % 7;
                RexxBufferStringObject buffer = thread->NewBufferString(length);
                POINTER data = thread->BufferStringData(buffer);
                if (data == NULL)
                {
                    continue;
                }
                memset(data, 'a' + (int)(i % 26), length);
                thread->FinishBufferString(buffer, length);
                if (thread->BufferStringLength(buffer) == length
                    && thread->StringLength((RexxStringObject)buffer) == length)
                {
                    good++;
                }
                thread->ReleaseLocalReference(buffer);
            }
            snprintf(report, sizeof report, "buffers %zu", good);
            break;
        }
        case INNER:
        {
            thread->SendMessage0(self->target, "INNER");
            RexxThreadContext *again = NULL;
            if (!self->instance->AttachThread(&again))
            {
                snprintf(report, sizeof report, "attach failed");
                break;
            }
            RexxObjectPtr ticked = again->SendMessage0(self->target, "TICK");
            snprintf(report, sizeof report, "%s ticked %s", again == thread ? "reused" : "fresh",
                     ticked != NULLOBJECT ? again->CString(ticked) : "null");
            again->ReleaseGlobalReference(self->target);
            again->DetachThread();
            break;
        }
        case WRONG:
        case LATE:
            break;
    }
}

// The other thread of `AtWrong`, which calls `DetachThread` on a context it is not attached to.
static void *detach_other(void *argument)
{
    ((RexxThreadContext *)argument)->DetachThread();
    return NULL;
}

// What `AtWrong` and `AtLate` do: each step on the one attachment, reported.
static void misuse(RexxThreadContext *thread, Thread *self)
{
    RexxObjectPtr first = thread->SendMessage0(self->target, "TICK");
    if (self->job == WRONG)
    {
        pthread_t other;
        if (pthread_create(&other, NULL, detach_other, thread) == 0)
        {
            pthread_join(other, NULL);
        }
        RexxObjectPtr second = thread->SendMessage0(self->target, "TICK");
        thread->ReleaseGlobalReference(self->target);
        thread->DetachThread();
        thread->DetachThread();
        snprintf(report, sizeof report, "first %s second %s", first != NULLOBJECT ? "answered" : "null",
                 second != NULLOBJECT ? "answered" : "null");
        return;
    }
    thread->DetachThread();
    RexxObjectPtr late = thread->SendMessage0(self->target, "TICK");
    snprintf(report, sizeof report, "first %s late %s", first != NULLOBJECT ? "answered" : "null",
             late != NULLOBJECT ? "answered" : "null");
}

// The body of `AtCycles`: attach, send and detach, over and over.
static void *cycles(void *argument)
{
    Thread *self = (Thread *)argument;
    RexxThreadContext *seen[3] = {NULL, NULL, NULL};
    size_t distinct = 0;
    for (size_t i = 0; i < self->count; i++)
    {
        RexxThreadContext *thread = NULL;
        if (!self->instance->AttachThread(&thread))
        {
            break;
        }
        bool known = false;
        for (size_t j = 0; j < distinct && j < 3; j++)
        {
            known = known || seen[j] == thread;
        }
        if (!known)
        {
            if (distinct < 3)
            {
                seen[distinct] = thread;
            }
            distinct++;
        }
        thread->SendMessage0(self->target, "TICK");
        if (i + 1 == self->count)
        {
            thread->ReleaseGlobalReference(self->target);
        }
        thread->DetachThread();
    }
    snprintf(report, sizeof report, "contexts %s", distinct <= 2 ? "bounded" : "grew");
    delete self;
    finished++;
    return NULL;
}

static void *body(void *argument)
{
    Thread *self = (Thread *)argument;
    RexxThreadContext *thread = NULL;
    if (self->instance->AttachThread(&thread))
    {
        if (self->job == WRONG || self->job == LATE)
        {
            misuse(thread, self);
        }
        else if (self->job == INNER)
        {
            run(thread, self);
        }
        else
        {
            run(thread, self);
            thread->ReleaseGlobalReference(self->target);
            thread->DetachThread();
        }
    }
    delete self;
    finished++;
    return NULL;
}

// Starts a detached thread running `entry` for `job`, and answers 1 where it started.
static logical_t start_with(RexxCallContext *context, RexxObjectPtr target, Job job, size_t count,
                            void *(*entry)(void *))
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
    int started = pthread_create(&thread, &attributes, entry, self);
    pthread_attr_destroy(&attributes);
    if (started != 0)
    {
        context->ReleaseGlobalReference(self->target);
        delete self;
        return 0;
    }
    return 1;
}

// Starts a detached thread doing `job`, and answers 1 where it started.
static logical_t start(RexxCallContext *context, RexxObjectPtr target, Job job, size_t count)
{
    return start_with(context, target, job, count, body);
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

RexxRoutine1(logical_t, AtIdle, RexxObjectPtr, target)
{
    return start(context, target, IDLE, 0);
}

RexxRoutine1(logical_t, AtSlow, RexxObjectPtr, target)
{
    return start(context, target, SLOW, 0);
}

RexxRoutine2(logical_t, AtCycles, RexxObjectPtr, target, positive_wholenumber_t, count)
{
    return start_with(context, target, TICKER, (size_t)count, cycles);
}

RexxRoutine1(logical_t, AtWrong, RexxObjectPtr, target)
{
    return start(context, target, WRONG, 0);
}

RexxRoutine1(logical_t, AtLate, RexxObjectPtr, target)
{
    return start(context, target, LATE, 0);
}

RexxRoutine2(logical_t, AtBuffers, RexxObjectPtr, target, positive_wholenumber_t, count)
{
    return start(context, target, BUFFERS, (size_t)count);
}

RexxRoutine1(logical_t, AtInner, RexxObjectPtr, target)
{
    return start(context, target, INNER, 0);
}

RexxRoutine1(logical_t, AtSendJoin, RexxObjectPtr, target)
{
    return start(context, target, JOIN, 0);
}

// What the thread `AtTickJoin` starts is given: the instance to attach to, and a global reference
// to the object it sends `TICK` to.
struct Tick
{
    RexxInstance *instance;
    RexxObjectPtr target;
};

static void *attach_and_tick(void *argument)
{
    Tick *tick = (Tick *)argument;
    RexxThreadContext *thread = NULL;
    if (tick->instance->AttachThread(&thread))
    {
        thread->SendMessage0(tick->target, "TICK");
        thread->ReleaseGlobalReference(tick->target);
        thread->DetachThread();
    }
    return NULL;
}

// Starts a thread that attaches, sends `TICK` to `target` and detaches, and joins it: answers 1,
// or -1 where the thread did not start.
RexxRoutine1(int, AtTickJoin, RexxObjectPtr, target)
{
    Tick tick = {context->threadContext->instance, context->RequestGlobalReference(target)};
    pthread_t thread;
    if (pthread_create(&thread, NULL, attach_and_tick, &tick) != 0)
    {
        context->ReleaseGlobalReference(tick.target);
        return -1;
    }
    pthread_join(thread, NULL);
    return 1;
}

// Detaches the thread running this call from the thread context of the call, and answers 1.
RexxRoutine0(int, AtDetachHere)
{
    context->threadContext->DetachThread();
    return 1;
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
    REXX_TYPED_ROUTINE(AtIdle, AtIdle),
    REXX_TYPED_ROUTINE(AtSlow, AtSlow),
    REXX_TYPED_ROUTINE(AtCycles, AtCycles),
    REXX_TYPED_ROUTINE(AtWrong, AtWrong),
    REXX_TYPED_ROUTINE(AtLate, AtLate),
    REXX_TYPED_ROUTINE(AtBuffers, AtBuffers),
    REXX_TYPED_ROUTINE(AtInner, AtInner),
    REXX_TYPED_ROUTINE(AtDetachHere, AtDetachHere),
    REXX_TYPED_ROUTINE(AtSendJoin, AtSendJoin),
    REXX_TYPED_ROUTINE(AtTickJoin, AtTickJoin),
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
