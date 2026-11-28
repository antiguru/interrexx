/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include <cstring>
#include <oorexxapi.h>

static size_t RexxEntry registered(const char *, size_t, CONSTRXSTRING *,
                                   const char *, PRXSTRING result)
{
    const char answer[] = "shared registration";
    std::memcpy(result->strptr, answer, sizeof(answer) - 1);
    result->strlength = sizeof(answer) - 1;
    return 0;
}

RexxRoutine0(int, installCliFunction)
{
    (void)context;
    return RexxRegisterFunctionExe("CLIREGISTERED", (REXXPFN)registered);
}

static RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(installCliFunction, installCliFunction),
    REXX_LAST_ROUTINE()
};

static RexxPackageEntry cli_registration_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "cli-registration",
    "1.0",
    nullptr,
    nullptr,
    routines,
    nullptr
};
OOREXX_GET_PACKAGE(cli_registration);
