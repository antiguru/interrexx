// A library whose loader prints `loader` and whose unloader prints `unloader`, and nothing else.
// It keeps nothing in static storage, whose state across loads is unspecified
// (`r[native.loading.static]`). Built as the extension `lifetime`.
//
// Where the environment variable LIFETIME_SLOW names `loader` or `unloader`, that hook is slow
// instead: it prints `loader start` (or `unloader start`), creates the file `hook-started` in the
// current directory, sleeps for a second and prints `loader end` (or `unloader end`). Where
// LIFETIME_NESTED is set, the loader, after printing `loader`, creates an instance of its own that
// loads `lifetime` too, prints `created inner`, terminates it and prints `terminated inner`.
// Where LIFETIME_RELOAD is set, the loader, after printing `loader`, loads `lifetime` through the
// thread context it is handed and prints `reloaded 1` or `reloaded 0` for the answer. Where
// LIFETIME_OTHER is set, it loads the library `lifetime2` the same way and prints `other 1` or
// `other 0`. Where LIFETIME_UNLOADER_CREATES is set, the unloader, after printing `unloader`,
// unsets it and does what LIFETIME_NESTED makes the loader do.
//
// Built with -DOTHER it is the extension `lifetime2` instead, a library whose hooks print `other
// loader` and `other unloader`, which does nothing else.
//
// Built with -DDRIVER it is also a program, which does not link the extension: it creates
// interpreter instances that each load `lifetime` with LOAD_REQUIRED_LIBRARY, and prints a line
// after each creation and each `Terminate`, so that the extension's lines fall between them.
//   two          creates A and B, terminates A, then B
//   three        as `two`, then creates C and terminates it
//   attached     as `three`, with a thread of its own attached to A and detached before A is
//                terminated
//   loading      with a slow loader, creates A on a thread of its own and, once the loader has
//                started, creates B; then terminates A and B
//   unloading    with a slow unloader, creates A, terminates it on a thread of its own and, once
//                the unloader has started, creates B; then terminates B, whose unloader is slow
//                too
//   nested       with LIFETIME_NESTED set, creates A and terminates it
//   reload       with LIFETIME_RELOAD set, creates A and terminates it
//   other        with LIFETIME_OTHER set, creates A and terminates it
//   unloadernew  with LIFETIME_UNLOADER_CREATES set, creates A and terminates it
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "oorexxapi.h"

#ifdef OTHER
#define PREFIX "other "
#define ENTRY lifetime2_package_entry
#define PACKAGE_NAME "lifetime2"
#else
#define PREFIX ""
#define ENTRY lifetime_package_entry
#define PACKAGE_NAME "lifetime"
#endif

// Runs the hook `name`: prints its name, or, where LIFETIME_SLOW names it, runs it slowly.
static void hook(const char *name)
{
    const char *slow = getenv("LIFETIME_SLOW");
    if (slow == NULL || strcmp(slow, name) != 0)
    {
        printf(PREFIX "%s\n", name);
        fflush(stdout);
        return;
    }
    printf(PREFIX "%s start\n", name);
    fflush(stdout);
    FILE *started = fopen("hook-started", "w");
    if (started != NULL)
    {
        fclose(started);
    }
    sleep(1);
    printf(PREFIX "%s end\n", name);
    fflush(stdout);
}

// Creates an instance that loads `lifetime`, from inside the loader, and terminates it.
static void inner(void)
{
    RexxOption options[2];
    options[0].optionName = LOAD_REQUIRED_LIBRARY;
    options[0].option = "lifetime";
    options[1].optionName = NULL;
    RexxInstance *instance;
    RexxThreadContext *thread;
    if (!RexxCreateInterpreter(&instance, &thread, options))
    {
        printf("create inner failed\n");
        return;
    }
    printf("created inner\n");
    instance->Terminate();
    printf("terminated inner\n");
}

// Loads the library `name` through `context` and prints `label` with the answer.
static void load_through(RexxThreadContext *context, const char *name, const char *label)
{
    printf("%s %d\n", label, context->LoadLibrary(name) ? 1 : 0);
    fflush(stdout);
}

static void RexxEntry LifetimeLoader(RexxThreadContext *context)
{
    hook("loader");
#ifndef OTHER
    if (getenv("LIFETIME_NESTED") != NULL)
    {
        inner();
    }
    if (getenv("LIFETIME_RELOAD") != NULL)
    {
        load_through(context, "lifetime", "reloaded");
    }
    if (getenv("LIFETIME_OTHER") != NULL)
    {
        load_through(context, "lifetime2", "other");
    }
#endif
}

static void RexxEntry LifetimeUnloader(RexxThreadContext *)
{
    hook("unloader");
#ifndef OTHER
    if (getenv("LIFETIME_UNLOADER_CREATES") != NULL)
    {
        // Once, so that the instance this creates does not create another in its own unloader.
        unsetenv("LIFETIME_UNLOADER_CREATES");
        inner();
    }
#endif
}

RexxPackageEntry ENTRY = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    PACKAGE_NAME,
    "1.0",
    LifetimeLoader,
    LifetimeUnloader,
    NULL,
    NULL
};

#ifdef OTHER
OOREXX_GET_PACKAGE(lifetime2);
#else
OOREXX_GET_PACKAGE(lifetime);
#endif

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

static void *create_a(void *)
{
    return create("A");
}

static void *terminate_a(void *instance)
{
    terminate((RexxInstance *)instance, "A");
    return instance;
}

// Waits, for at most ten seconds, until a slow hook has created `hook-started`.
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
    const char *scenario = argv[1];
    if (strcmp(scenario, "loading") == 0)
    {
        setenv("LIFETIME_SLOW", "loader", 1);
        pthread_t thread;
        void *a = NULL;
        if (pthread_create(&thread, NULL, create_a, NULL) != 0 || !hook_started())
        {
            return 1;
        }
        RexxInstance *b = create("B");
        if (pthread_join(thread, &a) != 0 || a == NULL || b == NULL)
        {
            return 1;
        }
        terminate((RexxInstance *)a, "A");
        terminate(b, "B");
        return 0;
    }
    if (strcmp(scenario, "nested") == 0 || strcmp(scenario, "reload") == 0
        || strcmp(scenario, "other") == 0 || strcmp(scenario, "unloadernew") == 0)
    {
        const char *variable = strcmp(scenario, "nested") == 0   ? "LIFETIME_NESTED"
                               : strcmp(scenario, "reload") == 0 ? "LIFETIME_RELOAD"
                               : strcmp(scenario, "other") == 0  ? "LIFETIME_OTHER"
                                                                 : "LIFETIME_UNLOADER_CREATES";
        setenv(variable, "1", 1);
        RexxInstance *a = create("A");
        if (a == NULL)
        {
            return 1;
        }
        terminate(a, "A");
        return 0;
    }
    if (strcmp(scenario, "unloading") == 0)
    {
        setenv("LIFETIME_SLOW", "unloader", 1);
        RexxInstance *a = create("A");
        pthread_t thread;
        if (a == NULL || pthread_create(&thread, NULL, terminate_a, a) != 0 || !hook_started())
        {
            return 1;
        }
        RexxInstance *b = create("B");
        if (pthread_join(thread, NULL) != 0 || b == NULL)
        {
            return 1;
        }
        terminate(b, "B");
        return 0;
    }
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
