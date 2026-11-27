// Run two programs under the instance's REXX_HEAP_LIMIT. The first makes an
// oversized request. The second collects while its array arguments are live.
// Its only assignment must be visible to the halt exit before 5.0 unwinds it.
#include <stdio.h>
#include "oorexxapi.h"

static bool sawAssignment;

static int REXXENTRY haltExit(RexxExitContext *context, int, int, PEXIT)
{
    if (context->GetContextVariable("HELD") != NULLOBJECT)
        sawAssignment = true;
    return RXEXIT_NOT_HANDLED;
}

static void run(RexxThreadContext *thread, const char *name, const char *file)
{
    thread->CallProgram(file, NULL);
    if (thread->CheckCondition())
    {
        RexxCondition condition;
        thread->DecodeConditionInfo(thread->GetConditionInfo(), &condition);
        printf("%s condition %ld.%ld\n", name, (long)(condition.code / 1000),
               (long)(condition.code % 1000));
        thread->ClearCondition();
    }
}

int main()
{
    RexxContextExit exits[] = { { haltExit, RXHLT }, { NULL, 0 } };
    RexxOption options[2] = {};
    options[0].optionName = DIRECT_EXITS;
    options[0].option = (POINTER)exits;
    RexxInstance *instance;
    RexxThreadContext *thread;
    if (!RexxCreateInterpreter(&instance, &thread, options))
        return 1;
    run(thread, "request", "request.rex");
    run(thread, "collect", "collect.rex");
    printf("exit saw assignment %d\n", sawAssignment);
    instance->Terminate();
    return 0;
}
