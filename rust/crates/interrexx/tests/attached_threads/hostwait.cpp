// An embedding host whose program waits on something only the host can end.
//
// Usage: hostwait instance | hostwait terminate | hostwait self | hostwait start <program>.
//
// * `instance`: the host creates an instance, makes an event semaphore in it, and starts a thread
//   that waits a while, attaches and posts the semaphore. The creating thread meanwhile calls a
//   routine that waits on the semaphore, and prints what the routine answered.
// * `terminate`: the host creates an instance, calls a routine that starts an activity waiting on
//   an event semaphore nothing posts, prints what the routine answered, and calls `Terminate`.
// * `self`: the host creates an instance and calls a routine that sends a message whose method
//   waits for that message's own result before it replies. It prints what the routine answered
//   and whether the call left a condition, and calls `Terminate`.
// * `start`: the host starts a thread that waits a while and halts the thread `RexxStart` runs
//   `<program>` on, then runs it there and prints what it answered.
#include <chrono>
#include <cstdio>
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

static int self_wait()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    const char *source = "m = .message~new(.k~new, 'RUN')\n"
                         ".environment~mm = m\n"
                         "return m~send\n"
                         "::class k\n"
                         "::method run\n"
                         "  say 'in run'\n"
                         "  return .environment~mm~result\n";
    RexxRoutineObject routine = context->NewRoutine("SELFWAIT", source, strlen(source));
    RexxObjectPtr result = context->CallRoutine(routine, context->NewArray(0));
    printf("routine %s condition %d\n", result == NULLOBJECT ? "(none)" : context->CString(result),
           context->CheckCondition() ? 1 : 0);
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
    if (argc >= 3 && strcmp(argv[1], "start") == 0)
    {
        return start_wait(argv[2]);
    }
    return 2;
}
