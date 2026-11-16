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
// unsets it and does what LIFETIME_NESTED makes the loader do. Where LIFETIME_UNLOADER_OTHER is
// set, the unloader, after printing `unloader`, unsets it, loads `lifetime2` through the thread
// context it is handed and prints `unloader other 1` or `unloader other 0`. Where
// LIFETIME_UNLOADER_KEEPS is set, the unloader, after printing `unloader`, unsets it, creates an
// instance that loads `lifetime`, prints `created inner` and leaves the instance live, with its
// address in the environment variable LIFETIME_KEPT, and adds the command environment `LIFETIME`,
// whose handler is the library's own and sets `RC` to 7, through the instance's thread context.
// Where LIFETIME_UNLOADER_THREAD is set, the
// unloader, after printing `unloader`, unsets it, creates and keeps an instance as
// LIFETIME_UNLOADER_KEEPS makes it do, sets LIFETIME_SLOW to `loader` and starts a thread of its
// own. The thread attaches to the kept instance, loads `lifetime2` through it, prints `thread
// other 1` or `thread other 0` for the answer, detaches and creates the file `thread-done` in the
// current directory. The unloader returns once that slow loader has started.
//
// The extension has one routine, `LifetimeProbe`, which answers 1 and prints nothing.
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
//   unloaderother
//                with LIFETIME_UNLOADER_OTHER set, creates A and terminates it, then creates C,
//                loads `lifetime2` through C's thread context, prints `C other` with the answer
//                and terminates C
//   unloaderkeep with LIFETIME_UNLOADER_KEEPS set, creates A and terminates it, then terminates
//                the instance the unloader left live and prints `terminated inner`, then creates
//                C and terminates it
//   unloaderprobe
//                with LIFETIME_UNLOADER_KEEPS set, creates A and terminates it. Then it attaches to
//                the instance the unloader left live, calls `LifetimeProbe` by name there and
//                prints `probe after` with the answer, or -1 where the call raised a condition,
//                detaches, and terminates that instance and prints `terminated inner`
//   unloadercommand
//                as `unloaderprobe`, but where that calls `LifetimeProbe` it runs the command `x`
//                to `LIFETIME` and prints `command after` with `RC`, or with -1 where the command
//                raised a condition
//   unloaderthread
//                with LIFETIME_UNLOADER_THREAD set, creates A and terminates it, waits until the
//                unloader's thread has created `thread-done`, then terminates the instance the
//                unloader left live and prints `terminated inner`
//   callrace     with a slow loader, creates A without loading `lifetime`. A thread of its own
//                attaches to A, loads `lifetime` there, prints `thread lifetime` with the answer
//                and detaches. Once the loader has started, the main thread calls `LifetimeProbe`
//                by name through A, from a routine it makes with `NewRoutine`, again and again
//                until that load has returned, and prints `race` with how many calls answered and
//                how many raised a condition. Then it calls `LifetimeProbe` by name once more,
//                prints `after` with the answer, or `after none` where the call raised a
//                condition, and terminates A
//   unloadercall with a slow unloader, creates A, and a thread of its own attaches to A. The main
//                thread terminates A. Once the unloader has started, the thread sends `LENGTH` to
//                the string `abc` through A, prints `thread callback` with the answer, or
//                `thread callback none` where the send answered no object, and detaches. The
//                program prints `joined` once the thread has ended
#include <atomic>
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

// Creates an instance that loads `lifetime`, from inside a hook, prints `created inner` and
// answers the instance, with its thread context in `thread` where that is not null.
static RexxInstance *create_inner(RexxThreadContext **thread = NULL)
{
    RexxOption options[2];
    options[0].optionName = LOAD_REQUIRED_LIBRARY;
    options[0].option = "lifetime";
    options[1].optionName = NULL;
    RexxInstance *instance;
    RexxThreadContext *created;
    if (thread == NULL)
    {
        thread = &created;
    }
    if (!RexxCreateInterpreter(&instance, thread, options))
    {
        printf("create inner failed\n");
        return NULL;
    }
    printf("created inner\n");
    return instance;
}

// Creates an instance that loads `lifetime`, from inside a hook, and terminates it.
static void inner(void)
{
    RexxInstance *instance = create_inner();
    if (instance == NULL)
    {
        return;
    }
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

#ifndef OTHER
// Attaches to the instance `argument`, loads `lifetime2` there, prints the answer, detaches and
// creates `thread-done`.
static void *load_other_and_finish(void *argument)
{
    RexxThreadContext *context;
    if (((RexxInstance *)argument)->AttachThread(&context))
    {
        load_through(context, "lifetime2", "thread other");
        context->DetachThread();
    }
    else
    {
        printf("thread did not attach\n");
    }
    FILE *done = fopen("thread-done", "w");
    if (done != NULL)
    {
        fclose(done);
    }
    return argument;
}

// Waits, for at most ten seconds, until the file `name` exists, answering whether it does.
static int appeared(const char *name)
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

// The handler of the command environment `LIFETIME`, which sets `RC` to 7.
static RexxObjectPtr RexxEntry lifetime_handler(RexxExitContext *context, RexxStringObject,
                                                RexxStringObject)
{
    return context->String("7");
}

// Creates an instance that loads `lifetime`, from inside a hook, adds the command environment
// `LIFETIME` through its thread context, leaves it live with its address in LIFETIME_KEPT, and
// answers it.
static RexxInstance *keep_inner(void)
{
    RexxThreadContext *thread;
    RexxInstance *instance = create_inner(&thread);
    if (instance != NULL)
    {
        thread->AddCommandEnvironment("LIFETIME", (REXXPFN)lifetime_handler,
                                      DIRECT_COMMAND_ENVIRONMENT);
        char address[32];
        snprintf(address, sizeof address, "%p", (void *)instance);
        setenv("LIFETIME_KEPT", address, 1);
    }
    return instance;
}
#endif

static void RexxEntry LifetimeUnloader(RexxThreadContext *context)
{
    hook("unloader");
#ifndef OTHER
    if (getenv("LIFETIME_UNLOADER_CREATES") != NULL)
    {
        // Once, so that the instance this creates does not create another in its own unloader.
        unsetenv("LIFETIME_UNLOADER_CREATES");
        inner();
    }
    if (getenv("LIFETIME_UNLOADER_OTHER") != NULL)
    {
        // Once, so that the later instance's unloader loads nothing.
        unsetenv("LIFETIME_UNLOADER_OTHER");
        load_through(context, "lifetime2", "unloader other");
    }
    if (getenv("LIFETIME_UNLOADER_KEEPS") != NULL)
    {
        // Once, so that the kept instance does not keep another in its own unloader.
        unsetenv("LIFETIME_UNLOADER_KEEPS");
        keep_inner();
    }
    if (getenv("LIFETIME_UNLOADER_THREAD") != NULL)
    {
        // Once, so that the kept instance does not start another thread in its own unloader.
        unsetenv("LIFETIME_UNLOADER_THREAD");
        RexxInstance *instance = keep_inner();
        if (instance == NULL)
        {
            return;
        }
        setenv("LIFETIME_SLOW", "loader", 1);
        unlink("hook-started");
        pthread_t thread;
        if (pthread_create(&thread, NULL, load_other_and_finish, instance) != 0)
        {
            printf("no thread\n");
            return;
        }
        pthread_detach(thread);
        appeared("hook-started");
    }
#endif
}

#ifndef OTHER
RexxRoutine0(int, LifetimeProbe)
{
    return 1;
}

static RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(LifetimeProbe, LifetimeProbe),
    REXX_LAST_ROUTINE()
};
#define ROUTINES routines
#else
#define ROUTINES NULL
#endif

RexxPackageEntry ENTRY = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    PACKAGE_NAME,
    "1.0",
    LifetimeLoader,
    LifetimeUnloader,
    ROUTINES,
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

// Creates the instance `name`, which loads no library, and answers it with its thread context in
// `thread`.
static RexxInstance *create_bare(const char *name, RexxThreadContext **thread)
{
    RexxInstance *instance;
    if (!RexxCreateInterpreter(&instance, thread, NULL))
    {
        printf("create %s failed\n", name);
        return NULL;
    }
    printf("created %s\n", name);
    return instance;
}

// Set once the thread `callrace` starts has finished its load of `lifetime`.
static std::atomic<bool> race_loaded(false);

// Attaches to the instance `argument`, loads `lifetime` there, prints the answer and detaches.
static void *load_lifetime(void *argument)
{
    RexxThreadContext *context;
    if (!((RexxInstance *)argument)->AttachThread(&context))
    {
        printf("thread did not attach\n");
        race_loaded = true;
        return NULL;
    }
    printf("thread lifetime %d\n", context->LoadLibrary("lifetime") ? 1 : 0);
    race_loaded = true;
    context->DetachThread();
    return argument;
}

// Runs `source` as a routine made with `NewRoutine` through `context`, answering the whole number
// it returned, or -1 where it raised a condition, which it clears.
static int call_source(RexxThreadContext *context, const char *source)
{
    RexxRoutineObject routine = context->NewRoutine("probe", source, strlen(source));
    if (routine == NULLOBJECT)
    {
        context->ClearCondition();
        return -1;
    }
    RexxObjectPtr answer = context->CallRoutine(routine, context->NewArray(0));
    if (context->CheckCondition() || answer == NULLOBJECT)
    {
        context->ClearCondition();
        return -1;
    }
    wholenumber_t number = 0;
    return context->WholeNumber(answer, &number) ? (int)number : -1;
}

// Calls `LifetimeProbe` by name through `context`, answering what it answered, or -1 where the call
// raised a condition, which it clears.
static int call_probe(RexxThreadContext *context)
{
    return call_source(context, "return LifetimeProbe()");
}

// Set once the thread `unloadercall` starts has attached.
static std::atomic<bool> callback_attached(false);

static int hook_started(void);

// Attaches to the instance `argument`, and once a slow hook has started sends `LENGTH` to `abc`
// through it, prints the answer and detaches.
static void *call_during_unloader(void *argument)
{
    RexxThreadContext *context;
    if (!((RexxInstance *)argument)->AttachThread(&context))
    {
        printf("thread did not attach\n");
        callback_attached = true;
        return NULL;
    }
    callback_attached = true;
    hook_started();
    RexxObjectPtr length = context->SendMessage0(context->NewStringFromAsciiz("abc"), "LENGTH");
    printf("thread callback %s\n",
           length != NULLOBJECT ? context->ObjectToStringValue(length) : "none");
    context->DetachThread();
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
    if (strcmp(scenario, "callrace") == 0)
    {
        setenv("LIFETIME_SLOW", "loader", 1);
        unlink("hook-started");
        RexxThreadContext *context;
        RexxInstance *a = create_bare("A", &context);
        pthread_t thread;
        if (a == NULL || pthread_create(&thread, NULL, load_lifetime, a) != 0 || !hook_started())
        {
            return 1;
        }
        int answered = 0;
        int raised = 0;
        do
        {
            if (call_probe(context) < 0)
            {
                raised++;
            }
            else
            {
                answered++;
            }
        } while (!race_loaded);
        void *loaded = NULL;
        if (pthread_join(thread, &loaded) != 0 || loaded == NULL)
        {
            return 1;
        }
        printf("race %d %d\n", answered, raised);
        int after = call_probe(context);
        if (after < 0)
        {
            printf("after none\n");
        }
        else
        {
            printf("after %d\n", after);
        }
        terminate(a, "A");
        return 0;
    }
    if (strcmp(scenario, "unloadercall") == 0)
    {
        setenv("LIFETIME_SLOW", "unloader", 1);
        unlink("hook-started");
        RexxInstance *a = create("A");
        pthread_t thread;
        if (a == NULL || pthread_create(&thread, NULL, call_during_unloader, a) != 0)
        {
            return 1;
        }
        while (!callback_attached)
        {
            usleep(1000);
        }
        terminate(a, "A");
        if (pthread_join(thread, NULL) != 0)
        {
            return 1;
        }
        printf("joined\n");
        return 0;
    }
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
    if (strcmp(scenario, "unloaderother") == 0)
    {
        setenv("LIFETIME_UNLOADER_OTHER", "1", 1);
        RexxInstance *a = create("A");
        if (a == NULL)
        {
            return 1;
        }
        terminate(a, "A");
        RexxOption options[2];
        options[0].optionName = LOAD_REQUIRED_LIBRARY;
        options[0].option = "lifetime";
        options[1].optionName = NULL;
        RexxInstance *c;
        RexxThreadContext *thread;
        if (!RexxCreateInterpreter(&c, &thread, options))
        {
            printf("create C failed\n");
            return 1;
        }
        printf("created C\n");
        load_through(thread, "lifetime2", "C other");
        terminate(c, "C");
        return 0;
    }
    if (strcmp(scenario, "unloaderkeep") == 0)
    {
        setenv("LIFETIME_UNLOADER_KEEPS", "1", 1);
        RexxInstance *a = create("A");
        if (a == NULL)
        {
            return 1;
        }
        terminate(a, "A");
        const char *address = getenv("LIFETIME_KEPT");
        RexxInstance *kept = NULL;
        if (address == NULL || sscanf(address, "%p", (void **)&kept) != 1 || kept == NULL)
        {
            printf("nothing kept\n");
            return 1;
        }
        terminate(kept, "inner");
        RexxInstance *c = create("C");
        if (c == NULL)
        {
            return 1;
        }
        terminate(c, "C");
        return 0;
    }
    if (strcmp(scenario, "unloaderprobe") == 0 || strcmp(scenario, "unloadercommand") == 0)
    {
        setenv("LIFETIME_UNLOADER_KEEPS", "1", 1);
        RexxInstance *a = create("A");
        if (a == NULL)
        {
            return 1;
        }
        terminate(a, "A");
        const char *address = getenv("LIFETIME_KEPT");
        RexxInstance *kept = NULL;
        RexxThreadContext *context;
        if (address == NULL || sscanf(address, "%p", (void **)&kept) != 1 || kept == NULL
            || !kept->AttachThread(&context))
        {
            printf("nothing kept\n");
            return 1;
        }
        if (strcmp(scenario, "unloaderprobe") == 0)
        {
            printf("probe after %d\n", call_probe(context));
        }
        else
        {
            printf("command after %d\n", call_source(context, "address lifetime 'x'; return rc"));
        }
        context->DetachThread();
        terminate(kept, "inner");
        return 0;
    }
    if (strcmp(scenario, "unloaderthread") == 0)
    {
        setenv("LIFETIME_UNLOADER_THREAD", "1", 1);
        unlink("thread-done");
        RexxInstance *a = create("A");
        if (a == NULL)
        {
            return 1;
        }
        terminate(a, "A");
        if (!appeared("thread-done"))
        {
            return 1;
        }
        const char *address = getenv("LIFETIME_KEPT");
        RexxInstance *kept = NULL;
        if (address == NULL || sscanf(address, "%p", (void **)&kept) != 1 || kept == NULL)
        {
            printf("nothing kept\n");
            return 1;
        }
        terminate(kept, "inner");
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
