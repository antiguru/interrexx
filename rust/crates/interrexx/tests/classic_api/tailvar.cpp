// A native method that assigns its receiver's object variable K, sent as the `+` of
// `t.k = t.k + 1` in a method of the same class that exposes k: the write resolves its tail
// after the native ran, so it lands on the tail the native named.
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"
RexxMethod1(int, AdderPlus, RexxObjectPtr, other) {
    context->SetObjectVariable("K", context->String("B"));
    return 1;
}
static RexxMethodEntry methods[] = { REXX_METHOD(AdderPlus, AdderPlus), REXX_LAST_METHOD() };
static RexxPackageEntry package = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "tailvar", "1.0", NULL, NULL, NULL, methods
};
static const char *source =
    "return .adder~new~run\n"
    "::requires 'tailvar' LIBRARY\n"
    "::class adder\n"
    "::method '+' external 'LIBRARY tailvar AdderPlus'\n"
    "::method run\n"
    "  expose k t.\n"
    "  k = 'A'\n"
    "  t. = self\n"
    "  t.k = t.k + 1\n"
    "  return t.A~class~id t.B k\n";
int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    RexxInstance *instance;
    RexxThreadContext *thread;
    RexxLibraryPackage library = { "tailvar", &package };
    RexxOption options[2];
    options[0].optionName = REGISTER_LIBRARY;
    options[0].option = (POINTER)&library;
    options[1].optionName = NULL;
    if (!RexxCreateInterpreter(&instance, &thread, options)) {
        printf("create failed\n");
        return 1;
    }
    RexxRoutineObject routine = thread->NewRoutine("tailvar", source, strlen(source));
    RexxObjectPtr result = routine == NULL ? NULL : thread->CallRoutine(routine, NULL);
    printf("%s\n", result == NULL ? "(none)" : thread->CString(result));
    if (thread->CheckCondition()) {
        RexxCondition condition;
        thread->DecodeConditionInfo(thread->GetConditionInfo(), &condition);
        printf("condition %d.%d\n", (int)condition.code / 1000, (int)condition.code % 1000);
        thread->ClearCondition();
    }
    instance->Terminate();
    return 0;
}
