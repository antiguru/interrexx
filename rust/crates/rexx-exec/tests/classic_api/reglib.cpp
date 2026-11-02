// RexxCreateInterpreter with REGISTER_LIBRARY: a package table the embedder registers under
// "mylib" serves a program's ::requires 'mylib' LIBRARY and, without the directive, nothing.
// A second instance lists an unknown option before INITIAL_ADDRESS_ENVIRONMENT, a third that
// option alone; each runs address.rex, which returns address().
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"
RexxRoutine0(RexxStringObject, MyFunc) { return context->String("fromlib"); }
static RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(MyFunc, MyFunc), REXX_LAST_ROUTINE() };
static RexxPackageEntry package = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "mylib", "1.0", NULL, NULL, routines, NULL
};
static void run(RexxThreadContext *thread, const char *label, const char *source) {
    RexxRoutineObject routine = thread->NewRoutine(label, source, strlen(source));
    if (routine == NULL) {
        printf("%s: no routine\n", label);
    } else {
        RexxObjectPtr result = thread->CallRoutine(routine, NULL);
        printf("%s: %s\n", label, result == NULL ? "(none)" : thread->CString(result));
    }
    if (thread->CheckCondition()) {
        RexxCondition condition;
        thread->DecodeConditionInfo(thread->GetConditionInfo(), &condition);
        printf("%s: condition %d\n", label, (int)condition.code);
        thread->ClearCondition();
    }
}
static void program(RexxThreadContext *thread, const char *label, const char *file) {
    RexxObjectPtr result = thread->CallProgram(file, NULL);
    printf("%s: %s\n", label, result == NULL ? "(none)" : thread->CString(result));
}
int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    RexxInstance *instance;
    RexxThreadContext *thread;
    RexxLibraryPackage library = { "mylib", &package };
    RexxOption options[2];
    options[0].optionName = REGISTER_LIBRARY;
    options[0].option = (POINTER)&library;
    options[1].optionName = NULL;
    if (!RexxCreateInterpreter(&instance, &thread, options)) {
        printf("create failed\n");
        return 1;
    }
    run(thread, "requires", "return myfunc()\n::requires 'mylib' LIBRARY");
    run(thread, "plain", "return myfunc()");
    instance->Terminate();
    RexxOption unknown[3];
    unknown[0].optionName = "NoSuchOption";
    unknown[0].option = (POINTER)NULL;
    unknown[1].optionName = INITIAL_ADDRESS_ENVIRONMENT;
    unknown[1].option = "ELSEWHERE";
    unknown[2].optionName = NULL;
    if (!RexxCreateInterpreter(&instance, &thread, unknown)) {
        printf("second create failed\n");
        return 1;
    }
    program(thread, "after unknown", "address.rex");
    instance->Terminate();
    if (!RexxCreateInterpreter(&instance, &thread, unknown + 1)) {
        printf("third create failed\n");
        return 1;
    }
    program(thread, "known", "address.rex");
    instance->Terminate();
    return 0;
}
