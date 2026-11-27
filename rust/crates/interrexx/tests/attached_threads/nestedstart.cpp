// A classic function registered by the host calls RexxStart while the outer
// RexxStart is still running. Both calls execute a file supplied by the test.
#include <cstdio>
#include <cstring>
#include "rexx.h"

static const char *inner_program;

static size_t RexxEntry nested(const char *, size_t, CONSTRXSTRING *, const char *, RXSTRING *result)
{
    RXSTRING inner = {0, NULL};
    short inner_rc = 0;
    int code = RexxStart(0, NULL, inner_program, NULL, "CMD", RXCOMMAND, NULL, &inner_rc, &inner);
    if (code != 0 || inner.strptr == NULL || inner.strlength >= RXAUTOBUFLEN)
    {
        fprintf(stderr, "nested RexxStart: code %d, result length %zu\n", code, inner.strlength);
        RexxFreeMemory(inner.strptr);
        return 1;
    }
    memcpy(result->strptr, inner.strptr, inner.strlength);
    result->strptr[inner.strlength] = '\0';
    result->strlength = inner.strlength;
    RexxFreeMemory(inner.strptr);
    return 0;
}

int main(int argc, char **argv)
{
    if (argc != 3)
    {
        return 1;
    }
    inner_program = argv[2];
    auto registered = RexxRegisterFunctionExe("NESTEDSTART", (REXXPFN)nested);
    if (registered != 0)
    {
        fprintf(stderr, "register classic function: %zu\n", (size_t)registered);
        return 2;
    }
    RXSTRING outer = {0, NULL};
    short outer_rc = 0;
    int code = RexxStart(0, NULL, argv[1], NULL, "CMD", RXCOMMAND, NULL, &outer_rc, &outer);
    if (outer.strptr)
    {
        printf("outer code %d result %.*s\n", code, (int)outer.strlength, outer.strptr);
    }
    else
    {
        printf("outer code %d no result\n", code);
    }
    RexxFreeMemory(outer.strptr);
    return RexxDeregisterFunction("NESTEDSTART") == 0 && code == 0 ? 0 : 3;
}
