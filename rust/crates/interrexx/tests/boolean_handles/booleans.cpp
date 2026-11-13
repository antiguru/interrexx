#include "oorexxapi.h"
#include <stdio.h>
#include <string.h>

// Answers "<case> <T|F|->": T when the handle under test is the one
// context->True() answers, F for context->False(), - for neither.
static RexxObjectPtr mark(RexxCallContext *context, const char *name, RexxObjectPtr h)
{
    char line[64];
    const char *m = h == context->True() ? "T" : h == context->False() ? "F" : "-";
    snprintf(line, sizeof line, "%s %s", name, m);
    return context->String(line);
}

RexxRoutine3(RexxObjectPtr, BoolHandles, CSTRING, which, RexxObjectPtr, a, RexxObjectPtr, b)
{
    if (strcmp(which, "hasentry_missing") == 0) {
        RexxObjectPtr dir = context->SendMessage0(context->FindClass("DIRECTORY"), "NEW");
        return mark(context, which,
                    context->SendMessage1(dir, "HASENTRY", context->String("K")));
    }
    if (strcmp(which, "hasentry_present") == 0) {
        RexxObjectPtr dir = context->SendMessage0(context->FindClass("DIRECTORY"), "NEW");
        context->SendMessage2(dir, "PUT", context->Nil(), context->String("K"));
        return mark(context, which,
                    context->SendMessage1(dir, "HASENTRY", context->String("K")));
    }
    if (strcmp(which, "compare") == 0) {
        return mark(context, which,
                    context->SendMessage1(context->String("1"), "=", context->String("1")));
    }
    if (strcmp(which, "isa") == 0) {
        return mark(context, which, context->SendMessage1(a, "ISA", b));
    }
    if (strcmp(which, "rexx_method") == 0) {
        return mark(context, which, context->SendMessage0(a, "YES"));
    }
    if (strcmp(which, "argument_true") == 0 || strcmp(which, "argument_false") == 0) {
        return mark(context, which, a);
    }
    char line[64];
    snprintf(line, sizeof line, "%s unknown", which);
    return context->String(line);
}

RexxRoutineEntry routines[] = {
    REXX_TYPED_ROUTINE(BoolHandles, BoolHandles),
    REXX_LAST_ROUTINE()
};
RexxPackageEntry booleans_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "BOOLEANS", "1.0.0", NULL, NULL, routines, NULL
};
OOREXX_GET_PACKAGE(booleans);
