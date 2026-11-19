// Three libraries and a program that loads them from two instances at once, where the first
// library's loader calls into the second instance.
//
// Built with -DSLOW it is the extension `crossslow`: its loader prints `slow loader start`, creates
// the file `hook-started` in the current directory and sleeps for a second. It then attaches to the
// instance whose address the environment variable CROSSLOAD_SECOND holds, sends `LENGTH` to the
// string `abc` there and prints `slow loader callback` with the answer.
//
// Built with -DQUICK it is the extension `crossquick`, whose loader prints `quick loader`.
//
// Built with -DOUTER it is the extension `crossouter`. Its loader prints `outer loader start`,
// attaches to the instance whose address the environment variable CROSSLOAD_FIRST holds, loads
// `crossslow` there, prints `outer loaded crossslow` with the answer, detaches and prints
// `outer loader end`.
//
// Built with -DDRIVER it is a program, which links none of the extensions. It creates the instances
// `first` and `second` and prints a line after each. A thread of its own attaches to `first` and
// loads `crossslow` there. Once that loader has started, the main thread loads a library through
// `second`'s thread context and prints `main` with the library's name and the answer. The thread
// prints `thread crossslow` with its answer, and once it is joined the program terminates both
// instances and prints `terminated`.
//   other  the main thread loads `crossquick`
//   same   the main thread loads `crossslow`
//   hook   the main thread loads `crossouter`, whose loader, running on `second`, waits for the
//          loader of `crossslow`, which calls into `second`
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "oorexxapi.h"

#ifdef SLOW
static void RexxEntry SlowLoader(RexxThreadContext *)
{
    printf("slow loader start\n");
    fflush(stdout);
    FILE *started = fopen("hook-started", "w");
    if (started != NULL)
    {
        fclose(started);
    }
    sleep(1);
    const char *address = getenv("CROSSLOAD_SECOND");
    RexxInstance *second = NULL;
    if (address != NULL)
    {
        sscanf(address, "%p", (void **)&second);
    }
    RexxThreadContext *context = NULL;
    if (second == NULL || !second->AttachThread(&context))
    {
        printf("slow loader did not attach\n");
        return;
    }
    RexxObjectPtr length = context->SendMessage0(context->NewStringFromAsciiz("abc"), "LENGTH");
    printf("slow loader callback %s\n",
           length != NULLOBJECT ? context->ObjectToStringValue(length) : "none");
    fflush(stdout);
}

RexxPackageEntry crossslow_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "crossslow",
    "1.0",
    SlowLoader,
    NULL,
    NULL,
    NULL
};

OOREXX_GET_PACKAGE(crossslow);
#endif

#ifdef QUICK
static void RexxEntry QuickLoader(RexxThreadContext *)
{
    printf("quick loader\n");
    fflush(stdout);
}

RexxPackageEntry crossquick_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "crossquick",
    "1.0",
    QuickLoader,
    NULL,
    NULL,
    NULL
};

OOREXX_GET_PACKAGE(crossquick);
#endif

#ifdef OUTER
static void RexxEntry OuterLoader(RexxThreadContext *)
{
    printf("outer loader start\n");
    fflush(stdout);
    const char *address = getenv("CROSSLOAD_FIRST");
    RexxInstance *first = NULL;
    if (address != NULL)
    {
        sscanf(address, "%p", (void **)&first);
    }
    RexxThreadContext *context = NULL;
    if (first == NULL || !first->AttachThread(&context))
    {
        printf("outer loader did not attach\n");
        return;
    }
    printf("outer loaded crossslow %d\n", context->LoadLibrary("crossslow") ? 1 : 0);
    context->DetachThread();
    printf("outer loader end\n");
    fflush(stdout);
}

RexxPackageEntry crossouter_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "crossouter",
    "1.0",
    OuterLoader,
    NULL,
    NULL,
    NULL
};

OOREXX_GET_PACKAGE(crossouter);
#endif

#ifdef DRIVER
static RexxInstance *first;

// Attaches to `first`, loads `crossslow` there and prints the answer.
static void *load_slow(void *)
{
    RexxThreadContext *context;
    if (!first->AttachThread(&context))
    {
        printf("thread did not attach\n");
        return NULL;
    }
    printf("thread crossslow %d\n", context->LoadLibrary("crossslow") ? 1 : 0);
    context->DetachThread();
    return NULL;
}

// Waits, for at most ten seconds, until the slow loader has created `hook-started`.
static int hook_started(void)
{
    for (int tries = 0; tries < 1000; tries++)
    {
        if (access("hook-started", F_OK) == 0)
        {
            return 1;
        }
        usleep(10000);
    }
    printf("the hook did not start\n");
    return 0;
}

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    if (argc < 2)
    {
        return 2;
    }
    const char *library = strcmp(argv[1], "same") == 0   ? "crossslow"
                          : strcmp(argv[1], "hook") == 0 ? "crossouter"
                                                         : "crossquick";
    unlink("hook-started");
    RexxThreadContext *first_context;
    if (!RexxCreateInterpreter(&first, &first_context, NULL))
    {
        return 1;
    }
    printf("created first\n");
    RexxInstance *second;
    RexxThreadContext *second_context;
    if (!RexxCreateInterpreter(&second, &second_context, NULL))
    {
        return 1;
    }
    printf("created second\n");
    char address[32];
    snprintf(address, sizeof address, "%p", (void *)second);
    setenv("CROSSLOAD_SECOND", address, 1);
    snprintf(address, sizeof address, "%p", (void *)first);
    setenv("CROSSLOAD_FIRST", address, 1);
    pthread_t thread;
    if (pthread_create(&thread, NULL, load_slow, NULL) != 0 || !hook_started())
    {
        return 1;
    }
    printf("main %s %d\n", library, second_context->LoadLibrary(library) ? 1 : 0);
    if (pthread_join(thread, NULL) != 0)
    {
        return 1;
    }
    first->Terminate();
    second->Terminate();
    printf("terminated\n");
    return 0;
}
#endif
