// C stdio output and the SAY output of programs CallProgram runs on an
// instance's thread context, with standard output not a terminal: each
// program's lines land before the C line printed after its call returns, and
// a native routine the program calls prints after the program's earlier lines,
// as does the C program after a SendMessage that wrote a line.
#include <stdio.h>
#include "oorexxapi.h"
RexxRoutine0(int, CPrint) {
    printf("c inside\n");
    return 0;
}
static RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(CPrint, CPrint), REXX_LAST_ROUTINE() };
static RexxPackageEntry package = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "clib", "1.0", NULL, NULL, routines, NULL
};
int main() {
    RexxInstance *instance;
    RexxThreadContext *context;
    RexxLibraryPackage library = { "clib", &package };
    RexxOption options[2];
    options[0].optionName = REGISTER_LIBRARY;
    options[0].option = (POINTER)&library;
    options[1].optionName = NULL;
    if (!RexxCreateInterpreter(&instance, &context, options)) {
        return 1;
    }
    printf("c one\n");
    context->CallProgram("p.rex", NULL);
    printf("c two\n");
    context->CallProgram("q.rex", NULL);
    printf("c three\n");
    RexxObjectPtr output = context->SendMessage1(
        context->GetLocalEnvironment(), "AT", context->String("OUTPUT"));
    context->SendMessage1(output, "LINEOUT", context->String("rexx send"));
    printf("c send\n");
    instance->Terminate();
    printf("c four\n");
    return 0;
}
