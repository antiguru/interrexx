// A package whose loader prints a line, loaded by a program's ::requires
// after the program has written output of its own.
#include <stdio.h>
#include "oorexxapi.h"
static void REXXENTRY HookLoad(RexxThreadContext *) { printf("c loader\n"); }
static RexxPackageEntry hooklib_package_entry = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "hooklib", "1.0", HookLoad, NULL, NULL, NULL
};
OOREXX_GET_PACKAGE(hooklib);
