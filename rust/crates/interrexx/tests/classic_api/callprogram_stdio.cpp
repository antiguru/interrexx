// C stdio output and the SAY output of programs CallProgram runs on an
// instance's thread context, with standard output not a terminal: each
// program's lines land before the C line printed after its call returns, and
// a native routine the program calls prints after the program's earlier lines,
// as does the C program after a SendMessage that wrote a line. A second
// instance does the same through a classic function, RexxCallBack, a context
// command handler, a termination exit and the loader of a package a program
// requires; LoadPackage and LoadPackageFromData land a prologue's lines too; and
// a partial line on standard error lands before the error line native code
// writes after it.
#include <stdio.h>
#include <string.h>
#include "oorexxapi.h"
RexxRoutine0(int, CPrint) {
    printf("c inside\n");
    return 0;
}
static RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(CPrint, CPrint), REXX_LAST_ROUTINE() };
static RexxPackageEntry package = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "clib", "1.0", NULL, NULL, routines, NULL
};
static size_t REXXENTRY CFunc(CSTRING, size_t, CONSTRXSTRING *, CSTRING, RXSTRING *ret) {
    printf("c classic\n");
    ret->strlength = 0;
    return 0;
}
static size_t REXXENTRY CErr(CSTRING, size_t, CONSTRXSTRING *, CSTRING, RXSTRING *ret) {
    fprintf(stderr, "c err\n");
    ret->strlength = 0;
    return 0;
}
static size_t REXXENTRY CBack(CSTRING, size_t, CONSTRXSTRING *, CSTRING, RXSTRING *ret) {
    short rc = 0;
    RXSTRING result = { 0, NULL };
    RexxCallBack("LBL", 0, NULL, &rc, &result);
    printf("c after callback\n");
    ret->strlength = 0;
    return 0;
}
static RexxObjectPtr REXXENTRY CEnv(RexxExitContext *, RexxStringObject, RexxStringObject) {
    printf("c command\n");
    return NULLOBJECT;
}
static int REXXENTRY CExit(RexxExitContext *, int, int, PEXIT) {
    printf("c exit\n");
    return RXEXIT_NOT_HANDLED;
}
static int second() {
    RexxFunctionHandler *cfunc = CFunc, *cerr = CErr, *cback = CBack;
    RexxRegisterFunctionExe("CFUNC", (REXXPFN)cfunc);
    RexxRegisterFunctionExe("CERR", (REXXPFN)cerr);
    RexxRegisterFunctionExe("CBACK", (REXXPFN)cback);
    RexxContextEnvironment environments[2] = { { CEnv, "CENV" }, { NULL, NULL } };
    RexxContextExit exits[2] = { { CExit, RXTER }, { NULL, 0 } };
    RexxOption options[3];
    options[0].optionName = DIRECT_ENVIRONMENTS;
    options[0].option = (POINTER)environments;
    options[1].optionName = DIRECT_EXITS;
    options[1].option = (POINTER)exits;
    options[2].optionName = NULL;
    RexxInstance *instance;
    RexxThreadContext *context;
    if (!RexxCreateInterpreter(&instance, &context, options)) {
        return 1;
    }
    context->CallProgram("s.rex", NULL);
    printf("c after program\n");
    instance->Terminate();
    return 0;
}
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
    context->LoadPackage("lp.rex");
    printf("c after load\n");
    const char *data = "say 'rexx data prologue'\n";
    context->LoadPackageFromData("lpd", data, strlen(data));
    printf("c after data\n");
    instance->Terminate();
    printf("c four\n");
    return second();
}
