// A library whose loader prints `loader` and whose unloader prints `unloader`, and nothing else.
// It keeps nothing in static storage, whose state across loads is unspecified
// (`r[native.loading.static]`). Built as the extension `lifetime`.
//
// Built with -DDRIVER it is also a program, which does not link the extension: it creates
// interpreter instances that each load `lifetime` with LOAD_REQUIRED_LIBRARY, and prints a line
// after each creation and each `Terminate`, so that the extension's lines fall between them.
//   two          creates A and B, terminates A, then B
//   three        as `two`, then creates C and terminates it
//   attached     as `three`, with a thread of its own attached to A and detached before A is
//                terminated
#include <pthread.h>
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"

static void RexxEntry LifetimeLoader(RexxThreadContext *)
{
    printf("loader\n");
    fflush(stdout);
}

static void RexxEntry LifetimeUnloader(RexxThreadContext *)
{
    printf("unloader\n");
    fflush(stdout);
}

RexxPackageEntry lifetime_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "lifetime",
    "1.0",
    LifetimeLoader,
    LifetimeUnloader,
    NULL,
    NULL
};

OOREXX_GET_PACKAGE(lifetime);

#ifdef DRIVER
static RexxInstance *create(const char *name)
{
    RexxOption options[2];
    options[0].optionName = LOAD_REQUIRED_LIBRARY;
    options[0].option = "lifetime";
    options[1].optionName = NULL;
    RexxInstance *instance;
    RexxThreadContext *thread;
    if (!RexxCreateInterpreter(&instance, &thread, options))
    {
        printf("create %s failed\n", name);
        return NULL;
    }
    printf("created %s\n", name);
    return instance;
}

// Attaches the calling thread to the instance `argument` and detaches it, answering whether the
// attach answered a context.
static void *attach_and_detach(void *argument)
{
    RexxThreadContext *attached;
    if (!((RexxInstance *)argument)->AttachThread(&attached))
    {
        return NULL;
    }
    attached->DetachThread();
    return argument;
}

static void terminate(RexxInstance *instance, const char *name)
{
    instance->Terminate();
    printf("terminated %s\n", name);
}

int main(int argc, char **argv)
{
    setvbuf(stdout, NULL, _IONBF, 0);
    if (argc < 2)
    {
        return 2;
    }
    const char *scenario = argv[1];
    RexxInstance *a = create("A");
    RexxInstance *b = create("B");
    if (a == NULL || b == NULL)
    {
        return 1;
    }
    if (strcmp(scenario, "attached") == 0)
    {
        pthread_t thread;
        void *answered = NULL;
        if (pthread_create(&thread, NULL, attach_and_detach, a) != 0
            || pthread_join(thread, &answered) != 0 || answered == NULL)
        {
            printf("attach failed\n");
            return 1;
        }
    }
    terminate(a, "A");
    terminate(b, "B");
    if (strcmp(scenario, "two") != 0)
    {
        RexxInstance *c = create("C");
        if (c == NULL)
        {
            return 1;
        }
        terminate(c, "C");
    }
    return 0;
}
#endif
