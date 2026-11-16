// Two libraries whose loaders, running at once on two threads, each load the other's library.
//
// Built with -DFIRST it is the extension `cyclefirst`, built with -DSECOND the extension
// `cyclesecond`. Each loader prints `<name> loader start`, creates the file `<name>-started` in the
// current directory and waits, for at most ten seconds, until the other loader has created its
// file. The loader of `cyclesecond` then sleeps for a second, so that the loader of `cyclefirst`
// asks first. Each then loads the other's library through the thread context it is handed, prints
// `<name> loaded` with the answer, and prints `<name> loader end`.
//
// Built with -DDRIVER it is a program, which links neither extension. It creates the instances
// `one` and `two`. A thread of its own attaches to `one` and loads `cyclefirst` there. Once that
// loader has started, the main thread loads `cyclesecond` through `two`'s thread context. Each
// prints the answer, and once the thread is joined the program terminates both instances and
// prints `terminated`.
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include "oorexxapi.h"

// Waits, for at most ten seconds, until the file `name` exists.
static int exists_soon(const char *name)
{
    for (int tries = 0; tries < 1000; tries++)
    {
        if (access(name, F_OK) == 0)
        {
            return 1;
        }
        usleep(10000);
    }
    printf("%s did not appear\n", name);
    return 0;
}

#if defined(FIRST) || defined(SECOND)
#ifdef FIRST
#define NAME "cyclefirst"
#define OTHER "cyclesecond"
#define ENTRY cyclefirst_package_entry
#else
#define NAME "cyclesecond"
#define OTHER "cyclefirst"
#define ENTRY cyclesecond_package_entry
#endif

static void RexxEntry CycleLoader(RexxThreadContext *context)
{
    printf(NAME " loader start\n");
    fflush(stdout);
    FILE *started = fopen(NAME "-started", "w");
    if (started != NULL)
    {
        fclose(started);
    }
    if (!exists_soon(OTHER "-started"))
    {
        return;
    }
#ifdef SECOND
    sleep(1);
#endif
    printf(NAME " loaded " OTHER " %d\n", context->LoadLibrary(OTHER) ? 1 : 0);
    printf(NAME " loader end\n");
    fflush(stdout);
}

RexxPackageEntry ENTRY = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    NAME,
    "1.0",
    CycleLoader,
    NULL,
    NULL,
    NULL
};

#ifdef FIRST
OOREXX_GET_PACKAGE(cyclefirst);
#else
OOREXX_GET_PACKAGE(cyclesecond);
#endif
#endif

#ifdef DRIVER
static RexxInstance *one;

// Attaches to `one`, loads `cyclefirst` there and prints the answer.
static void *load_first(void *)
{
    RexxThreadContext *context;
    if (!one->AttachThread(&context))
    {
        printf("thread did not attach\n");
        return NULL;
    }
    printf("thread cyclefirst %d\n", context->LoadLibrary("cyclefirst") ? 1 : 0);
    context->DetachThread();
    return NULL;
}

int main()
{
    setvbuf(stdout, NULL, _IONBF, 0);
    unlink("cyclefirst-started");
    unlink("cyclesecond-started");
    RexxThreadContext *one_context;
    if (!RexxCreateInterpreter(&one, &one_context, NULL))
    {
        return 1;
    }
    RexxInstance *two;
    RexxThreadContext *two_context;
    if (!RexxCreateInterpreter(&two, &two_context, NULL))
    {
        return 1;
    }
    pthread_t thread;
    if (pthread_create(&thread, NULL, load_first, NULL) != 0 || !exists_soon("cyclefirst-started"))
    {
        return 1;
    }
    printf("main cyclesecond %d\n", two_context->LoadLibrary("cyclesecond") ? 1 : 0);
    if (pthread_join(thread, NULL) != 0)
    {
        return 1;
    }
    one->Terminate();
    two->Terminate();
    printf("terminated\n");
    return 0;
}
#endif
