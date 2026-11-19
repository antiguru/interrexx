// An embedding's thread that attaches and sends a message, then stays attached, idle, while the
// creating thread calls Terminate, and sends again afterwards before it detaches.
//
// Usage: embed. Prints each step; the order is the observable.
#include <atomic>
#include <chrono>
#include <cstdio>
#include <thread>
#include "oorexxapi.h"

static std::atomic<int> stage(0);

static void attached(RexxInstance *instance)
{
    RexxThreadContext *context = NULL;
    if (!instance->AttachThread(&context))
    {
        printf("attach failed\n");
        fflush(stdout);
        stage = 1;
        return;
    }
    RexxObjectPtr text = context->String("abc");
    RexxObjectPtr first = context->SendMessage0(text, "LENGTH");
    printf("before %s\n", first == NULLOBJECT ? "null" : context->CString(first));
    fflush(stdout);
    stage = 1;
    while (stage != 2)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    RexxObjectPtr late = context->SendMessage0(text, "LENGTH");
    printf("after %s\n", late == NULLOBJECT ? "null" : "answered");
    fflush(stdout);
    context->DetachThread();
    printf("detached\n");
    fflush(stdout);
}

int main()
{
    RexxInstance *instance = NULL;
    RexxThreadContext *context = NULL;
    if (RexxCreateInterpreter(&instance, &context, NULL) != 1)
    {
        return 1;
    }
    std::thread other(attached, instance);
    while (stage == 0)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(10));
    }
    instance->Terminate();
    printf("terminated\n");
    fflush(stdout);
    stage = 2;
    other.join();
    return 0;
}
