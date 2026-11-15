// A routine that writes a line through C's standard output, for a program
// `rexx` runs with the library required.
#include <stdio.h>
#include "oorexxapi.h"
RexxRoutine0(int, CPrint) {
    printf("c\n");
    return 0;
}
static RexxRoutineEntry routines[] = { REXX_TYPED_ROUTINE(CPrint, CPrint), REXX_LAST_ROUTINE() };
static RexxPackageEntry cprintlib_package_entry = {
    STANDARD_PACKAGE_HEADER REXX_INTERPRETER_4_0_0, "cprintlib", "1.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(cprintlib);
