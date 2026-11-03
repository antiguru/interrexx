// What a thread still attached when its instance terminates may do afterwards: the thread
// context and instance it was handed outlive Terminate. The heap is churned between Terminate
// and the late calls, so a context or table Terminate freed is overwritten before it is read.
//
// Usage: attach_after <program.rex>. Prints each step; the order is the observable.
#include <atomic>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <thread>
#include "oorexxapi.h"

static std::atomic<int> stage(0);

// Allocates, overwrites and frees blocks of every small size, so freed memory is reused.
static void churn()
{
    for (int round = 0; round < 4; round++)
    {
        void *blocks[512];
        for (int i = 0; i < 512; i++)
        {
            size_t size = 8 + (size_t)i * 8;
            blocks[i] = malloc(size);
            memset(blocks[i], 0xa5, size);
        }
        for (int i = 0; i < 512; i++)
        {
            free(blocks[i]);
        }
    }
}

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
    while (stage != 2)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    printf("detaching\n");
    fflush(stdout);
    context->DetachThread();
    printf("detached\n");
    fflush(stdout);
    printf("version through the instance %s\n", instance->InterpreterVersion() >= REXX_INTERPRETER_5_0_0 ? "ok" : "low");
    fflush(stdout);
    stage = 3;
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
    churn();
    stage = 2;
    other.join();
    churn();
    instance->Terminate();
    printf("terminated again\n");
    fflush(stdout);
    return 0;
}
