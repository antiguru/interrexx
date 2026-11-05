// Review Focus 5: two instances in one process. The first sets an .environment and a .local
// entry, the second reports whether it sees them; then both run a long loop on threads of
// their own while the process is sent SIGINT, and each reports the condition that ended it.
//
// Usage: two <set.rex> <see.rex> <loop.rex>
#include <atomic>
#include <chrono>
#include <csignal>
#include <cstdio>
#include <thread>
#include <unistd.h>
#include "oorexxapi.h"

static std::atomic<int> started(0);

static const char *run(RexxThreadContext *context, const char *program, char *buffer, size_t size)
{
    RexxObjectPtr result = context->CallProgram(program, context->NewArray(0));
    if (context->CheckCondition())
    {
        RexxCondition condition;
        context->DecodeConditionInfo(context->GetConditionInfo(), &condition);
        snprintf(buffer, size, "condition %s", context->CString(condition.conditionName));
        context->ClearCondition();
        return buffer;
    }
    return result == NULLOBJECT ? "(none)" : context->CString(result);
}

static void looping(RexxInstance *instance, const char *program, const char *name)
{
    RexxThreadContext *context = NULL;
    instance->AttachThread(&context);
    started++;
    char buffer[100];
    const char *answer = run(context, program, buffer, sizeof(buffer));
    printf("%s %s\n", name, answer);
    fflush(stdout);
    context->DetachThread();
}

int main(int argc, char **argv)
{
    RexxInstance *first = NULL, *second = NULL;
    RexxThreadContext *one = NULL, *two = NULL;
    if (argc < 4 || RexxCreateInterpreter(&first, &one, NULL) != 1 ||
        RexxCreateInterpreter(&second, &two, NULL) != 1)
    {
        return 1;
    }
    char buffer[100];
    printf("first %s\n", run(one, argv[1], buffer, sizeof(buffer)));
    printf("second %s\n", run(two, argv[2], buffer, sizeof(buffer)));
    printf("first again %s\n", run(one, argv[2], buffer, sizeof(buffer)));
    fflush(stdout);
    std::thread a(looping, first, argv[3], "first loop");
    std::thread b(looping, second, argv[3], "second loop");
    while (started < 2)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    std::this_thread::sleep_for(std::chrono::milliseconds(300));
    kill(getpid(), SIGINT);
    a.join();
    b.join();
    first->Terminate();
    second->Terminate();
    printf("done\n");
    return 0;
}
