// The interpreter instance a native routine's thread context links, and the application data that
// instance carries, as one line: "<instance> <applicationData>". Built as the extension `inst`,
// which `inst.rex` loads with `::requires 'inst' LIBRARY`.
//
// Built with -DDRIVER it is also a program: it creates an interpreter with APPLICATION_DATA set
// and the package registered as `inst`, runs `inst.rex data` (argv[1]) through the creating
// thread's context, and prints whether the main activity's answer carries the data it set.
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"

RexxRoutine0(RexxObjectPtr, InstProbe)
{
    char text[64];
    RexxInstance *instance = context->threadContext->instance;
    snprintf(text, sizeof text, "%p %p", (void *)instance, instance->applicationData);
    return context->String(text);
}

static RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(InstProbe, InstProbe),
    REXX_LAST_ROUTINE()
};

RexxPackageEntry inst_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "inst",
    "1.0",
    NULL,
    NULL,
    routines,
    NULL
};

OOREXX_GET_PACKAGE(inst);

#ifdef DRIVER
static int marker;

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    if (argc < 2)
    {
        return 2;
    }
    RexxLibraryPackage library = { "inst", &inst_package_entry };
    RexxOption options[3];
    options[0].optionName = APPLICATION_DATA;
    options[0].option = (POINTER)&marker;
    options[1].optionName = REGISTER_LIBRARY;
    options[1].option = (POINTER)&library;
    options[2].optionName = NULL;
    RexxInstance *instance;
    RexxThreadContext *thread;
    if (!RexxCreateInterpreter(&instance, &thread, options))
    {
        printf("create failed\n");
        return 1;
    }
    RexxArrayObject args = thread->NewArray(1);
    thread->ArrayPut(args, thread->String("data"), 1);
    RexxObjectPtr result = thread->CallProgram(argv[1], args);
    char data[32];
    snprintf(data, sizeof data, "%p", (void *)&marker);
    const char *answer = result == NULLOBJECT ? "(none)" : thread->CString(result);
    const char *space = strchr(answer, ' ');
    if (space != NULL && strcmp(space + 1, data) == 0)
    {
        printf("data set\n");
    }
    else
    {
        printf("data %s\n", answer);
    }
    instance->Terminate();
    return 0;
}
#endif
