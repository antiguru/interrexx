// An embedding host whose program waits on something only the host can end.
//
// Usage: hostwait instance | hostwait terminate | hostwait self | hostwait selfclear
//        | hostwait attached | hostwait nested | hostwait stall <ms> | hostwait start <program>.
//
// * `instance`: the host creates an instance, makes an event semaphore in it, and starts a thread
//   that waits a while, attaches and posts the semaphore. The creating thread meanwhile calls a
//   routine that waits on the semaphore, and prints what the routine answered.
// * `terminate`: the host creates an instance, calls a routine that starts an activity waiting on
//   an event semaphore nothing posts, prints what the routine answered, and calls `Terminate`.
// * `self`: the host creates an instance and calls a routine that sends a message whose method
//   sets `.environment~ran` to 1 and waits for that message's own result before it replies. It
//   prints what the routine answered, whether the call left a condition and what
//   `.environment~ran` holds, and calls `Terminate`.
// * `selfclear`: as `self`, but the creating thread clears the condition before it calls
//   `Terminate`.
// * `attached`: as `self`, but a thread the host starts attaches and calls the routine, while the
//   creating thread waits for it outside any call. The thread prints what the routine answered,
//   whether the call left a condition and what `.environment~ran` holds, and detaches. The creating thread then calls `Terminate`.
// * `nested`: as `attached`, but the creating thread meanwhile calls a routine that waits on an
//   event semaphore. Once its own call has answered, the attached thread clears the condition and
//   posts the semaphore. The creating thread prints what its routine answered, and calls
//   `Terminate`.
// * `stall`: the host creates an instance and makes an event semaphore in it. One thread attaches
//   and calls a routine that waits on the semaphore, and another waits `<ms>` milliseconds,
//   attaches and posts it, while the creating thread waits for both outside any call. Then it
//   calls `Terminate`.
// * `start`: the host starts a thread that waits a while and halts the thread `RexxStart` runs
//   `<program>` on, then runs it there and prints what it answered.
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <thread>
#include <unistd.h>
#include <pthread.h>
#include "oorexxapi.h"

static void post_later(RexxInstance *instance, RexxObjectPtr semaphore)
{
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        return;
    }
    context->SendMessage0(semaphore, "POST");
    context->DetachThread();
}

static int instance_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    RexxObjectPtr semaphore = context->SendMessage0(context->FindClass("EVENTSEMAPHORE"), "NEW");
    semaphore = context->RequestGlobalReference(semaphore);
    const char *source = "use arg semaphore\nsemaphore~wait\nreturn 'posted'\n";
    RexxRoutineObject routine = context->NewRoutine("WAITER", source, strlen(source));
    RexxArrayObject arguments = context->ArrayOfOne(semaphore);
    std::thread poster(post_later, instance, semaphore);
    RexxObjectPtr result = context->CallRoutine(routine, arguments);
    printf("routine %s\n", result == NULLOBJECT ? "(none)" : context->CString(result));
    fflush(stdout);
    poster.join();
    instance->Terminate();
    return 0;
}

static int terminate_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    const char *source = ".message~new(.eventsemaphore~new, 'WAIT')~start\nreturn 'started'\n";
    RexxRoutineObject routine = context->NewRoutine("STARTER", source, strlen(source));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("routine %s\n", result == NULLOBJECT ? "(none)" : context->CString(result));
    fflush(stdout);
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static const char *self_waiter = "m = .message~new(.k~new, 'RUN')\n"
                                ".environment~mm = m\n"
                                "return m~send\n"
                                "::class k\n"
                                "::method run\n"
                                "  .environment~ran = 1\n"
                                "  return .environment~mm~result\n";

// What `.environment~ran` holds, which the method of `self_waiter` sets before it waits, or
// `(none)`. Read once the call has returned, so it does not race the method as a `SAY` would.
static const char *ran(RexxThreadContext *context)
{
    RexxObjectPtr value = context->DirectoryAt(context->GetGlobalEnvironment(), "RAN");
    return value == NULLOBJECT ? "(none)" : context->CString(value);
}

static int self_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    RexxRoutineObject routine = context->NewRoutine("SELFWAIT", self_waiter, strlen(self_waiter));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("routine %s condition %d ran %s\n",
           result == NULLOBJECT ? "(none)" : context->CString(result),
           context->CheckCondition() ? 1 : 0, ran(context));
    fflush(stdout);
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static int self_wait_clear()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    RexxRoutineObject routine = context->NewRoutine("SELFWAIT", self_waiter, strlen(self_waiter));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("routine %s condition %d ran %s\n",
           result == NULLOBJECT ? "(none)" : context->CString(result),
           context->CheckCondition() ? 1 : 0, ran(context));
    fflush(stdout);
    context->ClearCondition();
    printf("cleared condition %d\n", context->CheckCondition() ? 1 : 0);
    fflush(stdout);
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static RexxObjectPtr stall_semaphore;

static void stall_waiter(RexxInstance *instance)
{
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        return;
    }
    const char *source = "use arg semaphore\nsemaphore~wait\nreturn 'posted'\n";
    RexxRoutineObject routine = context->NewRoutine("W", source, strlen(source));
    RexxObjectPtr result = context->CallRoutine(routine, context->ArrayOfOne(stall_semaphore));
    printf("waiter %s\n", result == NULLOBJECT ? "(none)" : context->CString(result));
    fflush(stdout);
    context->DetachThread();
}

static void stall_poster(RexxInstance *instance, int milliseconds)
{
    std::this_thread::sleep_for(std::chrono::milliseconds(milliseconds));
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        return;
    }
    context->SendMessage0(stall_semaphore, "POST");
    context->DetachThread();
}

static int stall(int milliseconds)
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    stall_semaphore = context->RequestGlobalReference(
        context->SendMessage0(context->FindClass("EVENTSEMAPHORE"), "NEW"));
    std::thread waiter(stall_waiter, instance);
    std::thread poster(stall_poster, instance, milliseconds);
    waiter.join();
    poster.join();
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static void call_self_waiter(RexxInstance *instance)
{
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        return;
    }
    RexxRoutineObject routine = context->NewRoutine("SELFWAIT", self_waiter, strlen(self_waiter));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("thread routine %s condition %d ran %s\n",
           result == NULLOBJECT ? "(none)" : context->CString(result),
           context->CheckCondition() ? 1 : 0, ran(context));
    fflush(stdout);
    context->DetachThread();
}

static void call_self_waiter_then_post(RexxInstance *instance, RexxObjectPtr semaphore)
{
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        return;
    }
    RexxRoutineObject routine = context->NewRoutine("SELFWAIT", self_waiter, strlen(self_waiter));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("thread routine %s condition %d ran %s\n",
           result == NULLOBJECT ? "(none)" : context->CString(result),
           context->CheckCondition() ? 1 : 0, ran(context));
    fflush(stdout);
    context->ClearCondition();
    context->SendMessage0(semaphore, "POST");
    context->DetachThread();
}

static int nested_self_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    RexxObjectPtr semaphore = context->SendMessage0(context->FindClass("EVENTSEMAPHORE"), "NEW");
    semaphore = context->RequestGlobalReference(semaphore);
    const char *source = "use arg semaphore\nsemaphore~wait\nreturn 'posted'\n";
    RexxRoutineObject routine = context->NewRoutine("WAITER", source, strlen(source));
    RexxArrayObject arguments = context->ArrayOfOne(semaphore);
    std::thread caller(call_self_waiter_then_post, instance, semaphore);
    RexxObjectPtr result = context->CallRoutine(routine, arguments);
    printf("routine %s\n", result == NULLOBJECT ? "(none)" : context->CString(result));
    fflush(stdout);
    caller.join();
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static int attached_self_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    std::thread caller(call_self_waiter, instance);
    caller.join();
    printf("joined\n");
    fflush(stdout);
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    return 0;
}

static void halt_later(pthread_t thread)
{
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    RexxSetHalt(getpid(), (thread_id_t)thread);
}

static int start_wait(const char *program)
{
    std::thread halter(halt_later, pthread_self());
    short code = 0;
    RXSTRING result = {0, NULL};
    int rc = RexxStart(0, NULL, program, NULL, NULL, RXCOMMAND, NULL, &code, &result);
    printf("rc %d result %s\n", rc, result.strptr ? result.strptr : "(none)");
    fflush(stdout);
    RexxFreeMemory(result.strptr);
    halter.join();
    return 0;
}

int main(int argc, char **argv)
{
    if (argc >= 2 && strcmp(argv[1], "instance") == 0)
    {
        return instance_wait();
    }
    if (argc >= 2 && strcmp(argv[1], "terminate") == 0)
    {
        return terminate_wait();
    }
    if (argc >= 2 && strcmp(argv[1], "self") == 0)
    {
        return self_wait();
    }
    if (argc >= 2 && strcmp(argv[1], "selfclear") == 0)
    {
        return self_wait_clear();
    }
    if (argc >= 3 && strcmp(argv[1], "stall") == 0)
    {
        return stall(atoi(argv[2]));
    }
    if (argc >= 2 && strcmp(argv[1], "attached") == 0)
    {
        return attached_self_wait();
    }
    if (argc >= 2 && strcmp(argv[1], "nested") == 0)
    {
        return nested_self_wait();
    }
    if (argc >= 3 && strcmp(argv[1], "start") == 0)
    {
        return start_wait(argv[2]);
    }
    return 2;
}
