// RexxSetHalt naming the thread a native function runs on, from another thread the function
// starts and joins, in a program RexxCreateInterpreter's CallProgram runs (no RexxStart): the
// halt reaches the function's activity, which is halted at its next clause. Its standard output
// is the oracle's, line for line.
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <oorexxapi.h>

static pthread_t target;

static void *halter(void *)
{
    usleep(100000);
    printf("halt from another thread=%d\n", RexxSetHalt(getpid(), (thread_id_t)target));
    return NULL;
}

static size_t OtherHalt(const char *, size_t, PCONSTRXSTRING, const char *, PRXSTRING ret)
{
    target = pthread_self();
    pthread_t t;
    pthread_create(&t, NULL, halter, NULL);
    pthread_join(t, NULL);
    strcpy(ret->strptr, "back");
    ret->strlength = 4;
    return 0;
}

int main()
{
    setvbuf(stdout, NULL, _IONBF, 0);
    RexxRegisterFunctionExe("OTHERHALT", (REXXPFN)OtherHalt);
    RexxInstance *instance;
    RexxThreadContext *context;
    if (!RexxCreateInterpreter(&instance, &context, NULL))
    {
        printf("no interpreter\n");
        return 1;
    }
    RexxObjectPtr result = context->CallProgram("other.rex", NULL);
    printf("result %s\n", result ? context->CString(result) : "(none)");
    instance->Terminate();
    RexxDeregisterFunction("OTHERHALT");
    return 0;
}
