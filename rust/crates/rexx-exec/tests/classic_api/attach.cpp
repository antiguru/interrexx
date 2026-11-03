// Review Focus 4: a thread the instance never saw attaches, calls a program, and detaches;
// Terminate, called while it is attached, returns once it has detached.
//
// Usage: attach <program.rex>. Prints each step; the order is the observable.
#include <atomic>
#include <chrono>
#include <cstdio>
#include <thread>
#include "oorexxapi.h"

static std::atomic<int> stage(0);

static void attached(RexxInstance *instance, const char *program)
{
    RexxThreadContext *context = NULL;
    logical_t ok = instance->AttachThread(&context);
    RexxArrayObject args = context->NewArray(1);
    context->ArrayPut(args, context->String("21"), 1);
    RexxObjectPtr result = context->CallProgram(program, args);
    printf("attach %d result %s\n", (int)ok, result == NULLOBJECT ? "(none)" : context->CString(result));
    fflush(stdout);
    stage = 1;
    std::this_thread::sleep_for(std::chrono::milliseconds(500));
    printf("detaching\n");
    fflush(stdout);
    context->DetachThread();
}

int main(int argc, char **argv)
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (argc < 2 || RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    std::thread other(attached, instance, argv[1]);
    while (stage == 0)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    printf("terminating\n");
    fflush(stdout);
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    other.join();
    return 0;
}
