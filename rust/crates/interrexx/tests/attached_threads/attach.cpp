// Threads of the extension's own that attach to the interpreter and send messages from it. Built
// as the extension `attach`, which the programs beside it load with `::requires 'attach' LIBRARY`.
//
// Each routine that starts a thread answers at once, and the thread attaches through the instance
// the call's thread context links, sends its messages, detaches, and counts itself finished.
// `AtDone` answers how many threads have finished, which the programs wait for with `SysSleep`, so
// every thread has detached before the program ends. `AtAwaitDone` blocks until one thread has
// finished and answers how many have, so a traced program waits in one clause however long that
// takes. `AtReport` answers what the last thread that
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
// * `AtSend(target, name)` sends the message `name`, with no arguments.
// * `AtSendAfter(target, name, milliseconds)` waits `milliseconds` before it attaches, then does
//   what `AtSend` does.
// * `AtPause(milliseconds)` blocks for `milliseconds` and answers 1, starting no thread.
// * `AtPauseCall(milliseconds, routine)` blocks for `milliseconds`, then calls `routine` with no
//   arguments through `CallRoutine` on its own call's thread context, and answers what it
//   answered, or the null string where it answered nothing. It starts no thread.
// * `AtSendJoin(target)` sends `JOIN`, whose method calls `AtTickJoin` from inside the attached
//   thread's message, so the call that joins is made while a callback runs.
// * `AtCallTick(routine, argument, target)` calls `routine` with `argument` through `CallRoutine`,
//   then sends `TICK` to `target`.
// * `AtPutTick(array, target)` puts a string at index 1 of `array` through `ArrayPut`, then sends
//   `TICK` to `target`.
// * `AtMembers(array, stem, supplier, directory, table, object, package, routine, method)` starts
//   no thread. Through the call's own context it stores into and fetches from each collection with
//   the members of the thread interface that do so, asks the object `IsInstanceOf` and
//   `HasMethod`, asks the package for its classes and routines, and the routine and the method
//   for their packages. It answers an array of one line per member, the member's name and what it
//   answered, `null` for a null object.
// * `AtSendReporting(target, name)` sends the message `name`, and reports the condition the send
//   left, `condition NAME CODE` or `no condition`, then clears it.
// * `AtAttachIdle(target, name)` attaches and stays idle until `AtOpen` is called, then does what
//   `AtSendReporting` does. `AtAttached` answers 1 once that thread has attached, and 0 before.
// * `AtAttachHeld(target, name)` attaches and stays idle until `AtHaltHeld` lets it go on, then
//   sends `name` without reporting.
// * `AtHalt(milliseconds)` blocks for `milliseconds`, then calls `Halt` through the instance the
//   call's thread context links, and answers 1. `AtTrace(on)` calls `SetTrace` through it.
// * `AtStart()`, called by a method, notes that the method has started. `AtHaltStarted()` blocks
//   until a method has called `AtStart`, then does what `AtHalt(0)` does.
// * `AtHaltHeld()` opens what `AtOpen` opens and blocks until a method has called `AtStart`. Then
//   it lets the thread `AtAttachHeld` started go on, blocks until that thread is about to send,
//   and a further 300 milliseconds, and then does what `AtHalt(0)` does.
// * `AtHaltIdle(milliseconds)` blocks for `milliseconds`, then calls `HaltThread`, and
//   `AtTraceIdle(on)` calls `SetThreadTrace`, on the thread context of the thread `AtAttachIdle`
//   or `AtSendReporting` started, which has attached, from the thread running the call.
// * `AtMisuse()` opens what `AtOpen` opens and blocks until a method has called `AtStart`. Then,
//   from the thread running the call, it calls `HaltThread`, `SetThreadTrace(1)` and
//   `DetachThread` on the thread context `AtHaltIdle` uses, lets `AtAwaitRelease` return, and
//   answers how many milliseconds the three calls took.
// * `AtAwaitRelease()`, called by a method, blocks until `AtMisuse` lets it return.
#include <atomic>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
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
    JOIN,
    NAMED,
    REPORTED,
    IDLE_REPORTED,
    HELD
};

struct Thread
{
    RexxInstance *instance;
    RexxObjectPtr target;
    Job job;
    size_t count;
    // The message `NAMED` sends.
    char name[32];
};

static std::atomic<int> finished(0);
// Held while `finished` grows, which `AtAwaitDone` waits on.
static pthread_mutex_t finished_lock = PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t finished_grew = PTHREAD_COND_INITIALIZER;
static char report[128] = "";
// The thread context of the thread `AtAttachIdle` or `AtSendReporting` started, once it has
// attached, and whether `AtOpen` has let it send.
static std::atomic<RexxThreadContext *> idle_context(NULL);
static std::atomic<int> gate_open(0);
// Whether a method has called `AtStart`, and whether `AtMisuse` has let `AtAwaitRelease` return.
static std::atomic<int> started(0);
static std::atomic<int> released(0);
// Whether `AtHaltHeld` has let the thread `AtAttachHeld` started go on, and whether that thread
// is about to send.
static std::atomic<int> held_open(0);
static std::atomic<int> held_sending(0);

// Counts a thread as finished and wakes `AtAwaitDone`.
static void finish(void)
{
    pthread_mutex_lock(&finished_lock);
    finished++;
    pthread_cond_broadcast(&finished_grew);
    pthread_mutex_unlock(&finished_lock);
}

// Sends `name` to `target` and reports the condition the send left, which it clears.
static void send_reporting(RexxThreadContext *thread, RexxObjectPtr target, const char *name)
{
    thread->SendMessage0(target, name);
    if (!thread->CheckCondition())
    {
        snprintf(report, sizeof report, "no condition");
        return;
    }
    RexxDirectoryObject info = thread->GetConditionInfo();
    RexxObjectPtr condition =
        info != NULLOBJECT ? thread->DirectoryAt(info, "CONDITION") : NULLOBJECT;
    RexxObjectPtr code = info != NULLOBJECT ? thread->DirectoryAt(info, "CODE") : NULLOBJECT;
    snprintf(report, sizeof report, "condition %s %s",
             condition != NULLOBJECT ? thread->CString(condition) : "unknown",
             code != NULLOBJECT ? thread->CString(code) : "none");
    thread->ClearCondition();
}

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
        case NAMED:
            thread->SendMessage0(self->target, self->name);
            break;
        case REPORTED:
            idle_context = thread;
            send_reporting(thread, self->target, self->name);
            break;
        case HELD:
            while (held_open == 0)
            {
                usleep(1000);
            }
            held_sending = 1;
            thread->SendMessage0(self->target, self->name);
            break;
        case IDLE_REPORTED:
            idle_context = thread;
            while (gate_open == 0)
            {
                usleep(1000);
            }
            send_reporting(thread, self->target, self->name);
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
    finish();
    return NULL;
}

static void *body(void *argument)
{
    Thread *self = (Thread *)argument;
    RexxThreadContext *thread = NULL;
    // A message sent by name counts the milliseconds to wait before attaching.
    if (self->job == NAMED && self->count > 0)
    {
        usleep((useconds_t)(self->count * 1000));
    }
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
    finish();
    return NULL;
}

// Starts a detached thread running `entry` for `job`, which sends `name` where it names a message,
// and answers 1 where it started.
static logical_t start_with(RexxCallContext *context, RexxObjectPtr target, Job job, size_t count,
                            void *(*entry)(void *), const char *name = "")
{
    Thread *self = new Thread;
    self->instance = context->threadContext->instance;
    self->target = context->RequestGlobalReference(target);
    self->job = job;
    self->count = count;
    snprintf(self->name, sizeof self->name, "%s", name);
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

RexxRoutine2(logical_t, AtSend, RexxObjectPtr, target, CSTRING, name)
{
    return start_with(context, target, NAMED, 0, body, name);
}

RexxRoutine3(logical_t, AtSendAfter, RexxObjectPtr, target, CSTRING, name, positive_wholenumber_t,
             milliseconds)
{
    return start_with(context, target, NAMED, (size_t)milliseconds, body, name);
}

RexxRoutine2(logical_t, AtSendReporting, RexxObjectPtr, target, CSTRING, name)
{
    return start_with(context, target, REPORTED, 0, body, name);
}

RexxRoutine2(logical_t, AtAttachIdle, RexxObjectPtr, target, CSTRING, name)
{
    return start_with(context, target, IDLE_REPORTED, 0, body, name);
}

RexxRoutine2(logical_t, AtAttachHeld, RexxObjectPtr, target, CSTRING, name)
{
    return start_with(context, target, HELD, 0, body, name);
}

RexxRoutine0(int, AtAttached)
{
    return idle_context != NULL ? 1 : 0;
}

RexxRoutine0(int, AtOpen)
{
    gate_open = 1;
    return 1;
}

RexxRoutine1(int, AtHalt, wholenumber_t, milliseconds)
{
    usleep((useconds_t)(milliseconds * 1000));
    context->threadContext->instance->Halt();
    return 1;
}

RexxRoutine1(int, AtTrace, logical_t, on)
{
    context->threadContext->instance->SetTrace(on);
    return 1;
}

// Blocks until a method has called `AtStart`.
static void await_start()
{
    while (started == 0)
    {
        usleep(1000);
    }
}

RexxRoutine0(int, AtStart)
{
    started = 1;
    return 1;
}

RexxRoutine0(int, AtHaltStarted)
{
    await_start();
    context->threadContext->instance->Halt();
    return 1;
}

RexxRoutine0(int, AtHaltHeld)
{
    gate_open = 1;
    await_start();
    held_open = 1;
    while (held_sending == 0)
    {
        usleep(1000);
    }
    usleep(300000);
    context->threadContext->instance->Halt();
    return 1;
}

RexxRoutine0(wholenumber_t, AtMisuse)
{
    gate_open = 1;
    await_start();
    RexxThreadContext *attached = idle_context.load();
    struct timespec before, after;
    clock_gettime(CLOCK_MONOTONIC, &before);
    attached->HaltThread();
    attached->SetThreadTrace(1);
    attached->DetachThread();
    clock_gettime(CLOCK_MONOTONIC, &after);
    released = 1;
    return (wholenumber_t)((after.tv_sec - before.tv_sec) * 1000 +
                           (after.tv_nsec - before.tv_nsec) / 1000000);
}

RexxRoutine0(int, AtAwaitRelease)
{
    while (released == 0)
    {
        usleep(1000);
    }
    return 1;
}

RexxRoutine1(int, AtHaltIdle, wholenumber_t, milliseconds)
{
    usleep((useconds_t)(milliseconds * 1000));
    idle_context.load()->HaltThread();
    return 1;
}

RexxRoutine1(int, AtTraceIdle, logical_t, on)
{
    idle_context.load()->SetThreadTrace(on);
    return 1;
}

RexxRoutine1(int, AtPause, positive_wholenumber_t, milliseconds)
{
    usleep((useconds_t)(milliseconds * 1000));
    return 1;
}

RexxRoutine2(RexxObjectPtr, AtPauseCall, positive_wholenumber_t, milliseconds, RexxObjectPtr,
             routine)
{
    usleep((useconds_t)(milliseconds * 1000));
    RexxObjectPtr answer = context->CallRoutine((RexxRoutineObject)routine, context->NewArray(0));
    return answer == NULLOBJECT ? (RexxObjectPtr)context->NullString() : answer;
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

// What the thread `AtCallTick` or `AtPutTick` starts is given: the instance to attach to, and
// global references to the routine or array it calls or puts to first, the routine's argument and
// the object it then sends `TICK` to.
struct Then
{
    RexxInstance *instance;
    bool put;
    RexxObjectPtr first;
    RexxObjectPtr argument;
    RexxObjectPtr target;
};

static void *call_or_put_then_tick(void *argument)
{
    Then *then = (Then *)argument;
    RexxThreadContext *thread = NULL;
    if (then->instance->AttachThread(&thread))
    {
        if (then->put)
        {
            thread->ArrayPut((RexxArrayObject)then->first, thread->NewStringFromAsciiz("put"), 1);
        }
        else
        {
            thread->CallRoutine((RexxRoutineObject)then->first, thread->ArrayOfOne(then->argument));
        }
        thread->SendMessage0(then->target, "TICK");
        thread->ReleaseGlobalReference(then->first);
        thread->ReleaseGlobalReference(then->argument);
        thread->ReleaseGlobalReference(then->target);
        thread->DetachThread();
    }
    delete then;
    finish();
    return NULL;
}

// Starts a detached thread running `call_or_put_then_tick`, and answers 1 where it started.
static logical_t start_then(RexxCallContext *context, bool put, RexxObjectPtr first,
                            RexxObjectPtr argument, RexxObjectPtr target)
{
    Then *then = new Then;
    then->instance = context->threadContext->instance;
    then->put = put;
    then->first = context->RequestGlobalReference(first);
    then->argument = context->RequestGlobalReference(argument);
    then->target = context->RequestGlobalReference(target);
    pthread_t thread;
    pthread_attr_t attributes;
    pthread_attr_init(&attributes);
    pthread_attr_setdetachstate(&attributes, PTHREAD_CREATE_DETACHED);
    int started = pthread_create(&thread, &attributes, call_or_put_then_tick, then);
    pthread_attr_destroy(&attributes);
    if (started != 0)
    {
        context->ReleaseGlobalReference(then->first);
        context->ReleaseGlobalReference(then->argument);
        context->ReleaseGlobalReference(then->target);
        delete then;
        return 0;
    }
    return 1;
}

RexxRoutine3(logical_t, AtCallTick, RexxObjectPtr, routine, RexxObjectPtr, argument, RexxObjectPtr,
             target)
{
    return start_then(context, false, routine, argument, target);
}

RexxRoutine2(logical_t, AtPutTick, RexxObjectPtr, array, RexxObjectPtr, target)
{
    return start_then(context, true, array, target, target);
}

// Detaches the thread running this call from the thread context of the call, and answers 1.
// One line of `AtMembers`'s answer: `member`, then `text`.
static void line(RexxCallContext *context, RexxArrayObject lines, const char *member,
                 const char *text)
{
    char buffer[128];
    snprintf(buffer, sizeof(buffer), "%s %s", member, text);
    context->ArrayAppendString(lines, buffer, strlen(buffer));
}

// `line` with a whole number.
static void number(RexxCallContext *context, RexxArrayObject lines, const char *member,
                   size_t value)
{
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%zu", value);
    line(context, lines, member, buffer);
}

// `line` with an object that is null or a string, or with `present` for any other object.
static void object(RexxCallContext *context, RexxArrayObject lines, const char *member,
                   RexxObjectPtr value)
{
    if (value == NULLOBJECT)
    {
        line(context, lines, member, "null");
    }
    else if (context->IsString(value))
    {
        line(context, lines, member, context->CString(value));
    }
    else
    {
        line(context, lines, member, "present");
    }
}

RexxRoutine9(RexxObjectPtr, AtMembers, RexxObjectPtr, array, RexxObjectPtr, stem, RexxObjectPtr,
             supplier, RexxObjectPtr, directory, RexxObjectPtr, table, RexxObjectPtr, target,
             RexxObjectPtr, package, RexxObjectPtr, routine, RexxObjectPtr, method)
{
    RexxArrayObject lines = context->NewArray(0);
    RexxArrayObject a = (RexxArrayObject)array;
    context->ArrayPut(a, context->String("v"), 1);
    object(context, lines, "ArrayAt", context->ArrayAt(a, 1));
    object(context, lines, "ArrayAt", context->ArrayAt(a, 2));
    number(context, lines, "ArrayAppend", context->ArrayAppend(a, context->String("w")));
    number(context, lines, "ArrayAppendString", context->ArrayAppendString(a, "x", 1));
    number(context, lines, "ArraySize", context->ArraySize(a));
    number(context, lines, "ArrayItems", context->ArrayItems(a));
    number(context, lines, "ArrayDimension", context->ArrayDimension(a));

    RexxStemObject s = (RexxStemObject)stem;
    context->SetStemElement(s, "a", context->String("v"));
    object(context, lines, "GetStemElement", context->GetStemElement(s, "a"));
    context->DropStemElement(s, "a");
    object(context, lines, "GetStemElement", context->GetStemElement(s, "a"));
    context->SetStemArrayElement(s, 1, context->String("w"));
    object(context, lines, "GetStemArrayElement", context->GetStemArrayElement(s, 1));
    object(context, lines, "GetAllStemElements", context->GetAllStemElements(s));
    context->DropStemArrayElement(s, 1);
    object(context, lines, "GetStemArrayElement", context->GetStemArrayElement(s, 1));
    object(context, lines, "GetStemValue", context->GetStemValue(s));

    RexxSupplierObject p = (RexxSupplierObject)supplier;
    number(context, lines, "SupplierAvailable", context->SupplierAvailable(p));
    object(context, lines, "SupplierItem", context->SupplierItem(p));
    object(context, lines, "SupplierIndex", context->SupplierIndex(p));
    context->SupplierNext(p);
    number(context, lines, "SupplierAvailable", context->SupplierAvailable(p));

    RexxDirectoryObject d = (RexxDirectoryObject)directory;
    context->DirectoryPut(d, context->String("v"), "K");
    object(context, lines, "DirectoryAt", context->DirectoryAt(d, "K"));
    object(context, lines, "DirectoryRemove", context->DirectoryRemove(d, "K"));
    RexxStringTableObject t = (RexxStringTableObject)table;
    context->StringTablePut(t, context->String("v"), "K");
    object(context, lines, "StringTableAt", context->StringTableAt(t, "K"));
    object(context, lines, "StringTableRemove", context->StringTableRemove(t, "K"));

    RexxClassObject objectClass = context->FindClass("OBJECT");
    number(context, lines, "IsInstanceOf", context->IsInstanceOf(target, objectClass));
    number(context, lines, "HasMethod", context->HasMethod(target, "NOSUCH"));

    RexxPackageObject k = (RexxPackageObject)package;
    object(context, lines, "FindPackageClass", context->FindPackageClass(k, "NOSUCH"));
    object(context, lines, "GetPackageRoutines", context->GetPackageRoutines(k));
    object(context, lines, "GetPackagePublicRoutines", context->GetPackagePublicRoutines(k));
    object(context, lines, "GetPackageClasses", context->GetPackageClasses(k));
    object(context, lines, "GetPackagePublicClasses", context->GetPackagePublicClasses(k));
    object(context, lines, "GetPackageMethods", context->GetPackageMethods(k));
    object(context, lines, "GetRoutinePackage",
           context->GetRoutinePackage((RexxRoutineObject)routine));
    object(context, lines, "GetMethodPackage", context->GetMethodPackage((RexxMethodObject)method));
    return lines;
}

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

RexxRoutine0(int, AtAwaitDone)
{
    pthread_mutex_lock(&finished_lock);
    while (finished < 1)
    {
        pthread_cond_wait(&finished_grew, &finished_lock);
    }
    int count = finished;
    pthread_mutex_unlock(&finished_lock);
    return count;
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
    REXX_TYPED_ROUTINE(AtSend, AtSend),
    REXX_TYPED_ROUTINE(AtSendAfter, AtSendAfter),
    REXX_TYPED_ROUTINE(AtPause, AtPause),
    REXX_TYPED_ROUTINE(AtPauseCall, AtPauseCall),
    REXX_TYPED_ROUTINE(AtSendJoin, AtSendJoin),
    REXX_TYPED_ROUTINE(AtTickJoin, AtTickJoin),
    REXX_TYPED_ROUTINE(AtCallTick, AtCallTick),
    REXX_TYPED_ROUTINE(AtPutTick, AtPutTick),
    REXX_TYPED_ROUTINE(AtMembers, AtMembers),
    REXX_TYPED_ROUTINE(AtSendReporting, AtSendReporting),
    REXX_TYPED_ROUTINE(AtAttachIdle, AtAttachIdle),
    REXX_TYPED_ROUTINE(AtAttached, AtAttached),
    REXX_TYPED_ROUTINE(AtOpen, AtOpen),
    REXX_TYPED_ROUTINE(AtHalt, AtHalt),
    REXX_TYPED_ROUTINE(AtTrace, AtTrace),
    REXX_TYPED_ROUTINE(AtHaltIdle, AtHaltIdle),
    REXX_TYPED_ROUTINE(AtTraceIdle, AtTraceIdle),
    REXX_TYPED_ROUTINE(AtAttachHeld, AtAttachHeld),
    REXX_TYPED_ROUTINE(AtStart, AtStart),
    REXX_TYPED_ROUTINE(AtHaltStarted, AtHaltStarted),
    REXX_TYPED_ROUTINE(AtHaltHeld, AtHaltHeld),
    REXX_TYPED_ROUTINE(AtMisuse, AtMisuse),
    REXX_TYPED_ROUTINE(AtAwaitRelease, AtAwaitRelease),
    REXX_TYPED_ROUTINE(AtDone, AtDone),
    REXX_TYPED_ROUTINE(AtAwaitDone, AtAwaitDone),
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
