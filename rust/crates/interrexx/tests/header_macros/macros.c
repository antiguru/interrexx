/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * An extension written against include/ alone, which uses every family of the
 * header's declaration machinery: RexxRoutine0 to RexxRoutine10 and
 * RexxMethod0 to RexxMethod10, every value code as a parameter and as a
 * result, the special arguments, the optional forms, the routine and method
 * table rows and their prototypes, the package entry macros, the classic
 * function declaration, the RXSTRING macros and the descriptor helpers.
 *
 * The same file compiles as C and as C++. The C++ build adds the entries
 * that only C++ can write (argumentExists, argumentOmitted, and the assignments
 * to and conversions of a ValueDescriptor). header_macros.rs builds it both ways and runs
 * header_macros.rex against each library.
 */

#include "oorexxapi.h"

#include <stdio.h>
#include <string.h>

/* A classic function, with C linkage in either language. It answers its
 * name, its argument count, and one word per argument: N for an empty
 * RXSTRING, Z for the null string, and V and the length for one with
 * characters. HmClassic below calls it. */
BEGIN_EXTERN_C()
REXX_CLASSIC_ROUTINE_PROTOTYPE(HmClassicInfo);

size_t RexxEntry HmClassicInfo(CSTRING name, size_t argc, CONSTRXSTRING *argv, CSTRING queue,
                               RXSTRING *result)
{
    char *out = RXSTRPTR(*result);
    size_t room = RXSTRLEN(*result);
    int used;
    size_t i;
    (void)queue;
    if (!RXVALIDSTRING(*result))
    {
        return 1;
    }
    used = snprintf(out, room, "%s:%zu", name, argc);
    for (i = 0; i < argc && used > 0 && (size_t)used < room; i++)
    {
        if (RXNULLSTRING(argv[i]))
        {
            used += snprintf(out + used, room - (size_t)used, " N");
        }
        else if (RXZEROLENSTRING(argv[i]))
        {
            used += snprintf(out + used, room - (size_t)used, " Z");
        }
        else
        {
            used += snprintf(out + used, room - (size_t)used, " V%zu", (size_t)RXSTRLEN(argv[i]));
        }
    }
    if (used < 0 || (size_t)used >= room)
    {
        return 1;
    }
    MAKERXSTRING(*result, out, (size_t)used);
    return 0;
}
END_EXTERN_C()

/* Prototypes ahead of the declarations, as a table in another source file
 * would use them. */
REXX_TYPED_ROUTINE_PROTOTYPE(HmSum0);
REXX_TYPED_ROUTINE_PROTOTYPE(HmSum10);
REXX_METHOD_PROTOTYPE(HmMSum0);
REXX_METHOD_PROTOTYPE(HmMSum10);
REXX_TYPED_ROUTINE_PROTOTYPE(HmTableRows);
REXX_TYPED_ROUTINE_PROTOTYPE(HmPackage);

/* The classic function called as the interpreter calls one, with an argument
 * of three characters, an empty one and the null string, and a result buffer
 * of 256 characters. */
RexxRoutine0(CSTRING, HmClassic)
{
    static char buffer[256];
    static char out[256];
    CONSTRXSTRING argv[3];
    RXSTRING result;
    size_t rc;
    MAKERXSTRING(argv[0], "abc", 3);
    MAKERXSTRING(argv[1], NULL, 0);
    MAKERXSTRING(argv[2], "", 0);
    MAKERXSTRING(result, buffer, sizeof buffer);
    rc = HmClassicInfo("HMINFO", 3, argv, NULL, &result);
    snprintf(out, sizeof out, "%zu %.*s", rc, (int)RXSTRLEN(result), RXSTRPTR(result));
    return out;
}

/* Each arity: the digits given, read as one number in the order declared. */
RexxRoutine0(wholenumber_t, HmSum0)
{
    return 0;
}
RexxRoutine1(wholenumber_t, HmSum1, int, a1)
{
    return a1;
}
RexxRoutine2(wholenumber_t, HmSum2, int, a1, int, a2)
{
    return a1 * 10 + a2;
}
RexxRoutine3(wholenumber_t, HmSum3, int, a1, int, a2, int, a3)
{
    return (a1 * 10 + a2) * 10 + a3;
}
RexxRoutine4(wholenumber_t, HmSum4, int, a1, int, a2, int, a3, int, a4)
{
    return ((a1 * 10 + a2) * 10 + a3) * 10 + a4;
}
RexxRoutine5(wholenumber_t, HmSum5, int, a1, int, a2, int, a3, int, a4, int, a5)
{
    return (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
}
RexxRoutine6(wholenumber_t, HmSum6, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6)
{
    return ((((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5) * 10 + a6;
}
RexxRoutine7(wholenumber_t, HmSum7, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
             a7)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return (n * 10 + a6) * 10 + a7;
}
RexxRoutine8(wholenumber_t, HmSum8, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
             a7, int, a8)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return ((n * 10 + a6) * 10 + a7) * 10 + a8;
}
RexxRoutine9(wholenumber_t, HmSum9, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
             a7, int, a8, int, a9)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return (((n * 10 + a6) * 10 + a7) * 10 + a8) * 10 + a9;
}
RexxRoutine10(wholenumber_t, HmSum10, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
              a7, int, a8, int, a9, int, a10)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return ((((n * 10 + a6) * 10 + a7) * 10 + a8) * 10 + a9) * 10 + a10;
}

RexxMethod0(wholenumber_t, HmMSum0)
{
    return 0;
}
RexxMethod1(wholenumber_t, HmMSum1, int, a1)
{
    return a1;
}
RexxMethod2(wholenumber_t, HmMSum2, int, a1, int, a2)
{
    return a1 * 10 + a2;
}
RexxMethod3(wholenumber_t, HmMSum3, int, a1, int, a2, int, a3)
{
    return (a1 * 10 + a2) * 10 + a3;
}
RexxMethod4(wholenumber_t, HmMSum4, int, a1, int, a2, int, a3, int, a4)
{
    return ((a1 * 10 + a2) * 10 + a3) * 10 + a4;
}
RexxMethod5(wholenumber_t, HmMSum5, int, a1, int, a2, int, a3, int, a4, int, a5)
{
    return (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
}
RexxMethod6(wholenumber_t, HmMSum6, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6)
{
    return ((((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5) * 10 + a6;
}
RexxMethod7(wholenumber_t, HmMSum7, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
            a7)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return (n * 10 + a6) * 10 + a7;
}
RexxMethod8(wholenumber_t, HmMSum8, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
            a7, int, a8)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return ((n * 10 + a6) * 10 + a7) * 10 + a8;
}
RexxMethod9(wholenumber_t, HmMSum9, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
            a7, int, a8, int, a9)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return (((n * 10 + a6) * 10 + a7) * 10 + a8) * 10 + a9;
}
RexxMethod10(wholenumber_t, HmMSum10, int, a1, int, a2, int, a3, int, a4, int, a5, int, a6, int,
             a7, int, a8, int, a9, int, a10)
{
    wholenumber_t n = (((a1 * 10 + a2) * 10 + a3) * 10 + a4) * 10 + a5;
    return ((((n * 10 + a6) * 10 + a7) * 10 + a8) * 10 + a9) * 10 + a10;
}

/* Every value code that takes an argument, as the parameter and the result:
 * the routine answers its argument. */
#define HM_ECHO(t) \
    RexxRoutine1(t, HmEcho_##t, t, v) \
    { \
        return v; \
    }

HM_ECHO(RexxObjectPtr)
HM_ECHO(int)
HM_ECHO(wholenumber_t)
HM_ECHO(double)
HM_ECHO(CSTRING)
HM_ECHO(POINTER)
HM_ECHO(RexxStringObject)
HM_ECHO(stringsize_t)
HM_ECHO(float)
HM_ECHO(int8_t)
HM_ECHO(int16_t)
HM_ECHO(int32_t)
HM_ECHO(int64_t)
HM_ECHO(__int32_t)
HM_ECHO(__int64_t)
HM_ECHO(uint8_t)
HM_ECHO(uint16_t)
HM_ECHO(uint32_t)
HM_ECHO(uint64_t)
HM_ECHO(__uint32_t)
HM_ECHO(__uint64_t)
HM_ECHO(intptr_t)
HM_ECHO(uintptr_t)
HM_ECHO(__uintptr_t)
HM_ECHO(logical_t)
HM_ECHO(RexxArrayObject)
HM_ECHO(RexxStemObject)
HM_ECHO(size_t)
HM_ECHO(ssize_t)
HM_ECHO(POINTERSTRING)
HM_ECHO(RexxClassObject)
HM_ECHO(RexxMutableBufferObject)
HM_ECHO(RexxVariableReferenceObject)

/* The two codes that restrict a whole number, as parameters. */
RexxRoutine1(wholenumber_t, HmEcho_positive_wholenumber_t, positive_wholenumber_t, v)
{
    return v;
}
RexxRoutine1(wholenumber_t, HmEcho_nonnegative_wholenumber_t, nonnegative_wholenumber_t, v)
{
    return v;
}

/* The optional forms of the numeric codes: an omitted argument is 0. */
#define HM_OPTIONAL(t) \
    RexxRoutine1(t, HmOpt_##t, OPTIONAL_##t, v) \
    { \
        return v; \
    }

HM_OPTIONAL(int)
HM_OPTIONAL(wholenumber_t)
HM_OPTIONAL(double)
HM_OPTIONAL(stringsize_t)
HM_OPTIONAL(float)
HM_OPTIONAL(int8_t)
HM_OPTIONAL(int16_t)
HM_OPTIONAL(int32_t)
HM_OPTIONAL(int64_t)
HM_OPTIONAL(uint8_t)
HM_OPTIONAL(uint16_t)
HM_OPTIONAL(uint32_t)
HM_OPTIONAL(uint64_t)
HM_OPTIONAL(intptr_t)
HM_OPTIONAL(uintptr_t)
HM_OPTIONAL(logical_t)
HM_OPTIONAL(size_t)
HM_OPTIONAL(ssize_t)

RexxRoutine1(wholenumber_t, HmOpt_positive_wholenumber_t, OPTIONAL_positive_wholenumber_t, v)
{
    return v;
}
RexxRoutine1(wholenumber_t, HmOpt_nonnegative_wholenumber_t, OPTIONAL_nonnegative_wholenumber_t,
             v)
{
    return v;
}

/* The optional forms of the pointer and object codes: an omitted argument is
 * a null pointer, which the routine reports. */
#define HM_OPTIONAL_NULL(t) \
    RexxRoutine1(logical_t, HmOptNull_##t, OPTIONAL_##t, v) \
    { \
        return v == NULL; \
    }

HM_OPTIONAL_NULL(RexxObjectPtr)
HM_OPTIONAL_NULL(CSTRING)
HM_OPTIONAL_NULL(POINTER)
HM_OPTIONAL_NULL(RexxStringObject)
HM_OPTIONAL_NULL(RexxArrayObject)
HM_OPTIONAL_NULL(RexxStemObject)
HM_OPTIONAL_NULL(POINTERSTRING)
HM_OPTIONAL_NULL(RexxClassObject)
HM_OPTIONAL_NULL(RexxMutableBufferObject)

/* An address the pointer codes carry, and the routines that make it and
 * recognise it. */
static int hmMarker;

RexxRoutine0(POINTER, HmPointer)
{
    return &hmMarker;
}
RexxRoutine1(logical_t, HmIsMarker, POINTER, p)
{
    return p == &hmMarker;
}
RexxRoutine0(POINTERSTRING, HmPointerString)
{
    return &hmMarker;
}
RexxRoutine1(logical_t, HmIsMarkerString, POINTERSTRING, p)
{
    return p == &hmMarker;
}

/* Ten parameters of ten codes, each written back in its place. */
RexxRoutine10(CSTRING, HmMixed, int8_t, a, uint16_t, b, int32_t, c, uint64_t, d, double, e,
              CSTRING, f, logical_t, g, size_t, h, ssize_t, i, wholenumber_t, j)
{
    static char out[256];
    snprintf(out, sizeof out, "%d|%u|%d|%llu|%g|%s|%zu|%zu|%zd|%ld", (int)a, (unsigned)b, (int)c,
             (unsigned long long)d, e, f, (size_t)g, h, i, (long)j);
    return out;
}

/* The special arguments of a routine. */
RexxRoutine1(CSTRING, HmName, NAME, name)
{
    return name;
}
RexxRoutine2(RexxArrayObject, HmArgs, NAME, name, ARGLIST, args)
{
    (void)name;
    return args;
}

/* The special arguments of a method. */
RexxMethod1(RexxObjectPtr, HmSelf, OSELF, self)
{
    return self;
}
RexxMethod1(RexxClassObject, HmSuper, SUPER, super)
{
    return super;
}
RexxMethod1(RexxObjectPtr, HmScope, SCOPE, scope)
{
    return scope;
}
RexxMethod1(logical_t, HmCself, CSELF, p)
{
    return p == &hmMarker;
}
RexxMethod1(CSTRING, HmMessage, NAME, name)
{
    return name;
}
RexxMethod2(RexxArrayObject, HmMArgs, OSELF, self, ARGLIST, args)
{
    (void)self;
    return args;
}

/* The descriptor and signature-word helpers and the legacy names, outside a
 * call: each answers 1 where it holds. */
RexxRoutine0(CSTRING, HmHelpers)
{
    static char out[64];
    ValueDescriptor d;
    CONSTANT_STRING s = "x";
    CONSTANT_RXSTRING r;
    d.flags = 5;
    SET_REXX_VALUE(d, wholenumber_t, 42);
    MAKERXSTRING(r, s, 1);
    snprintf(out, sizeof out, "%d %d %d %d %d %d %d %d", d.type == REXX_VALUE_wholenumber_t,
             d.flags == 5, GET_REXX_VALUE(d, wholenumber_t) == 42,
             ARGUMENT_TYPE(REXX_VALUE_OPTIONAL_int) == REXX_VALUE_int,
             IS_OPTIONAL_ARGUMENT(REXX_VALUE_OPTIONAL_int) && !IS_OPTIONAL_ARGUMENT(REXX_VALUE_int),
             NULLOBJECT == NULL && NO_CSTRING == NULL, RXSTRPTR(r) == s,
             NO_VERSION_YET == NULL);
    return out;
}

/* The platform macros: whether __REXX64__ is defined exactly where a pointer
 * has 64 bits, and whether SSIZE_MIN is the least ssize_t. */
RexxRoutine0(CSTRING, HmPlatform)
{
#ifdef __REXX64__
    int rexx64 = sizeof(void *) == 8;
#else
    int rexx64 = sizeof(void *) != 8;
#endif
    ssize_t least = SSIZE_MIN;
    return rexx64 && least < 0 && least + SSIZE_MAX == -1 ? "1 1" : "0";
}

#ifdef __cplusplus

/* A handle converts to a wider one without a cast. */
static RexxStringObject hmNarrowest(RexxBufferStringObject b)
{
    return b;
}
RexxRoutine1(RexxObjectPtr, HmWiden, RexxStringObject, s)
{
    RexxObjectPtr o = s;
    (void)hmNarrowest;
    return o;
}

/* Which of the arguments were given, through argumentExists for the first
 * parameter and argumentOmitted for the others: E for given, O for omitted.
 * The second routine's first parameter is NAME, so its argument is
 * descriptor 2. */
RexxRoutine3(CSTRING, HmExists, OPTIONAL_int, a, OPTIONAL_CSTRING, b, OPTIONAL_RexxObjectPtr, c)
{
    static char out[4];
    (void)a;
    (void)b;
    (void)c;
    out[0] = argumentExists(1) ? 'E' : 'O';
    out[1] = argumentOmitted(2) ? 'O' : 'E';
    out[2] = argumentOmitted(3) ? 'O' : 'E';
    out[3] = '\0';
    return out;
}
RexxRoutine2(CSTRING, HmExistsAfterName, NAME, name, OPTIONAL_int, a)
{
    (void)name;
    (void)a;
    return argumentExists(2) ? "E" : "O";
}

/* The type code each C++ assignment to a ValueDescriptor sets, and whether
 * every one left the flags. */
RexxRoutine0(CSTRING, HmAssign)
{
    static char out[64];
    ValueDescriptor d;
    int kept = 1;
    int codes[10];
    d.flags = 3;
    d = (RexxObjectPtr)NULLOBJECT;
    codes[0] = d.type;
    d = (RexxStringObject)NULLOBJECT;
    codes[1] = d.type;
    d = (RexxArrayObject)NULLOBJECT;
    codes[2] = d.type;
    d = (RexxStemObject)NULLOBJECT;
    codes[3] = d.type;
    d = (RexxMutableBufferObject)NULLOBJECT;
    codes[4] = d.type;
    d = (CSTRING) "s";
    codes[5] = d.type;
    d = (POINTER)&hmMarker;
    codes[6] = d.type;
    d = (wholenumber_t)7;
    codes[7] = d.type;
    kept = kept && d.value.value_wholenumber_t == 7;
    d = 1.5f;
    codes[8] = d.type;
    d = 2.5;
    codes[9] = d.type;
    kept = kept && d.flags == 3 && d.value.value_double == 2.5;
    snprintf(out, sizeof out, "%d %d %d %d %d %d %d %d %d %d %d", codes[0], codes[1], codes[2],
             codes[3], codes[4], codes[5], codes[6], codes[7], codes[8], codes[9], kept);
    return out;
}

/* Whether each C++ conversion of a ValueDescriptor answers the union member of
 * its type. Each initialisation names one type, so overload resolution picks
 * the conversion to exactly that type among the ones that could serve it.
 * Before each value is stored, the union is filled with a byte pattern, and
 * each value differs from what a narrower member, or float read as double and
 * double as float, would give. The pointer members, and wholenumber_t and
 * stringsize_t, share all their bits, so a mix-up among them reads the same
 * value: the header generator checks that each conversion reads a member of
 * its own type. POINTER is read into a const void pointer, which CSTRING and
 * every object pointer convert to as well, so without its own conversion the
 * initialisation is ambiguous. */
RexxRoutine0(CSTRING, HmConvert)
{
    static char out[32];
    static char marks[7];
    static const char text[] = "c";
    ValueDescriptor d;
    int same[11];
    d.type = 0;
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_RexxObjectPtr = (RexxObjectPtr)&marks[0];
    RexxObjectPtr o = d;
    same[0] = o == (RexxObjectPtr)&marks[0];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_RexxStringObject = (RexxStringObject)&marks[1];
    RexxStringObject s = d;
    same[1] = s == (RexxStringObject)&marks[1];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_RexxArrayObject = (RexxArrayObject)&marks[2];
    RexxArrayObject a = d;
    same[2] = a == (RexxArrayObject)&marks[2];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_RexxStemObject = (RexxStemObject)&marks[3];
    RexxStemObject t = d;
    same[3] = t == (RexxStemObject)&marks[3];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_RexxMutableBufferObject = (RexxMutableBufferObject)&marks[4];
    RexxMutableBufferObject b = d;
    same[4] = b == (RexxMutableBufferObject)&marks[4];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_CSTRING = text;
    CSTRING c = d;
    same[5] = c == text;
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_POINTER = &marks[6];
    const void *p = d;
    same[6] = p == &marks[6];
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_wholenumber_t = -(wholenumber_t)0x123456789;
    wholenumber_t w = d;
    same[7] = w == -(wholenumber_t)0x123456789;
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_stringsize_t = (stringsize_t)0x987654321;
    stringsize_t z = d;
    same[8] = z == (stringsize_t)0x987654321;
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_float = 1.5f;
    float f = d;
    same[9] = f == 1.5f;
    memset(&d.value, 0xA5, sizeof d.value);
    d.value.value_double = 2.5;
    double x = d;
    same[10] = x == 2.5;
    snprintf(out, sizeof out, "%d %d %d %d %d %d %d %d %d %d %d", same[0], same[1], same[2],
             same[3], same[4], same[5], same[6], same[7], same[8], same[9], same[10]);
    return out;
}

#endif

static RexxRoutineEntry hmRoutines[] = {
    REXX_TYPED_ROUTINE(HmSum0, HmSum0),
    REXX_TYPED_ROUTINE(HmSum1, HmSum1),
    REXX_TYPED_ROUTINE(HmSum2, HmSum2),
    REXX_TYPED_ROUTINE(HmSum3, HmSum3),
    REXX_TYPED_ROUTINE(HmSum4, HmSum4),
    REXX_TYPED_ROUTINE(HmSum5, HmSum5),
    REXX_TYPED_ROUTINE(HmSum6, HmSum6),
    REXX_TYPED_ROUTINE(HmSum7, HmSum7),
    REXX_TYPED_ROUTINE(HmSum8, HmSum8),
    REXX_TYPED_ROUTINE(HmSum9, HmSum9),
    REXX_TYPED_ROUTINE(HmSum10, HmSum10),
    /* A row written with the general form, under a name of its own. */
    REXX_ROUTINE(ROUTINE_TYPED_STYLE, HmRow, HmSum2),
    REXX_TYPED_ROUTINE(HmEcho_RexxObjectPtr, HmEcho_RexxObjectPtr),
    REXX_TYPED_ROUTINE(HmEcho_int, HmEcho_int),
    REXX_TYPED_ROUTINE(HmEcho_wholenumber_t, HmEcho_wholenumber_t),
    REXX_TYPED_ROUTINE(HmEcho_double, HmEcho_double),
    REXX_TYPED_ROUTINE(HmEcho_CSTRING, HmEcho_CSTRING),
    REXX_TYPED_ROUTINE(HmEcho_POINTER, HmEcho_POINTER),
    REXX_TYPED_ROUTINE(HmEcho_RexxStringObject, HmEcho_RexxStringObject),
    REXX_TYPED_ROUTINE(HmEcho_stringsize_t, HmEcho_stringsize_t),
    REXX_TYPED_ROUTINE(HmEcho_float, HmEcho_float),
    REXX_TYPED_ROUTINE(HmEcho_int8_t, HmEcho_int8_t),
    REXX_TYPED_ROUTINE(HmEcho_int16_t, HmEcho_int16_t),
    REXX_TYPED_ROUTINE(HmEcho_int32_t, HmEcho_int32_t),
    REXX_TYPED_ROUTINE(HmEcho_int64_t, HmEcho_int64_t),
    REXX_TYPED_ROUTINE(HmEcho___int32_t, HmEcho___int32_t),
    REXX_TYPED_ROUTINE(HmEcho___int64_t, HmEcho___int64_t),
    REXX_TYPED_ROUTINE(HmEcho_uint8_t, HmEcho_uint8_t),
    REXX_TYPED_ROUTINE(HmEcho_uint16_t, HmEcho_uint16_t),
    REXX_TYPED_ROUTINE(HmEcho_uint32_t, HmEcho_uint32_t),
    REXX_TYPED_ROUTINE(HmEcho_uint64_t, HmEcho_uint64_t),
    REXX_TYPED_ROUTINE(HmEcho___uint32_t, HmEcho___uint32_t),
    REXX_TYPED_ROUTINE(HmEcho___uint64_t, HmEcho___uint64_t),
    REXX_TYPED_ROUTINE(HmEcho_intptr_t, HmEcho_intptr_t),
    REXX_TYPED_ROUTINE(HmEcho_uintptr_t, HmEcho_uintptr_t),
    REXX_TYPED_ROUTINE(HmEcho___uintptr_t, HmEcho___uintptr_t),
    REXX_TYPED_ROUTINE(HmEcho_logical_t, HmEcho_logical_t),
    REXX_TYPED_ROUTINE(HmEcho_RexxArrayObject, HmEcho_RexxArrayObject),
    REXX_TYPED_ROUTINE(HmEcho_RexxStemObject, HmEcho_RexxStemObject),
    REXX_TYPED_ROUTINE(HmEcho_size_t, HmEcho_size_t),
    REXX_TYPED_ROUTINE(HmEcho_ssize_t, HmEcho_ssize_t),
    REXX_TYPED_ROUTINE(HmEcho_POINTERSTRING, HmEcho_POINTERSTRING),
    REXX_TYPED_ROUTINE(HmEcho_RexxClassObject, HmEcho_RexxClassObject),
    REXX_TYPED_ROUTINE(HmEcho_RexxMutableBufferObject, HmEcho_RexxMutableBufferObject),
    REXX_TYPED_ROUTINE(HmEcho_RexxVariableReferenceObject, HmEcho_RexxVariableReferenceObject),
    REXX_TYPED_ROUTINE(HmEcho_positive_wholenumber_t, HmEcho_positive_wholenumber_t),
    REXX_TYPED_ROUTINE(HmEcho_nonnegative_wholenumber_t, HmEcho_nonnegative_wholenumber_t),
    REXX_TYPED_ROUTINE(HmOpt_int, HmOpt_int),
    REXX_TYPED_ROUTINE(HmOpt_wholenumber_t, HmOpt_wholenumber_t),
    REXX_TYPED_ROUTINE(HmOpt_double, HmOpt_double),
    REXX_TYPED_ROUTINE(HmOpt_stringsize_t, HmOpt_stringsize_t),
    REXX_TYPED_ROUTINE(HmOpt_float, HmOpt_float),
    REXX_TYPED_ROUTINE(HmOpt_int8_t, HmOpt_int8_t),
    REXX_TYPED_ROUTINE(HmOpt_int16_t, HmOpt_int16_t),
    REXX_TYPED_ROUTINE(HmOpt_int32_t, HmOpt_int32_t),
    REXX_TYPED_ROUTINE(HmOpt_int64_t, HmOpt_int64_t),
    REXX_TYPED_ROUTINE(HmOpt_uint8_t, HmOpt_uint8_t),
    REXX_TYPED_ROUTINE(HmOpt_uint16_t, HmOpt_uint16_t),
    REXX_TYPED_ROUTINE(HmOpt_uint32_t, HmOpt_uint32_t),
    REXX_TYPED_ROUTINE(HmOpt_uint64_t, HmOpt_uint64_t),
    REXX_TYPED_ROUTINE(HmOpt_intptr_t, HmOpt_intptr_t),
    REXX_TYPED_ROUTINE(HmOpt_uintptr_t, HmOpt_uintptr_t),
    REXX_TYPED_ROUTINE(HmOpt_logical_t, HmOpt_logical_t),
    REXX_TYPED_ROUTINE(HmOpt_size_t, HmOpt_size_t),
    REXX_TYPED_ROUTINE(HmOpt_ssize_t, HmOpt_ssize_t),
    REXX_TYPED_ROUTINE(HmOpt_positive_wholenumber_t, HmOpt_positive_wholenumber_t),
    REXX_TYPED_ROUTINE(HmOpt_nonnegative_wholenumber_t, HmOpt_nonnegative_wholenumber_t),
    REXX_TYPED_ROUTINE(HmOptNull_RexxObjectPtr, HmOptNull_RexxObjectPtr),
    REXX_TYPED_ROUTINE(HmOptNull_CSTRING, HmOptNull_CSTRING),
    REXX_TYPED_ROUTINE(HmOptNull_POINTER, HmOptNull_POINTER),
    REXX_TYPED_ROUTINE(HmOptNull_RexxStringObject, HmOptNull_RexxStringObject),
    REXX_TYPED_ROUTINE(HmOptNull_RexxArrayObject, HmOptNull_RexxArrayObject),
    REXX_TYPED_ROUTINE(HmOptNull_RexxStemObject, HmOptNull_RexxStemObject),
    REXX_TYPED_ROUTINE(HmOptNull_POINTERSTRING, HmOptNull_POINTERSTRING),
    REXX_TYPED_ROUTINE(HmOptNull_RexxClassObject, HmOptNull_RexxClassObject),
    REXX_TYPED_ROUTINE(HmOptNull_RexxMutableBufferObject, HmOptNull_RexxMutableBufferObject),
    REXX_TYPED_ROUTINE(HmPointer, HmPointer),
    REXX_TYPED_ROUTINE(HmIsMarker, HmIsMarker),
    REXX_TYPED_ROUTINE(HmPointerString, HmPointerString),
    REXX_TYPED_ROUTINE(HmIsMarkerString, HmIsMarkerString),
    REXX_TYPED_ROUTINE(HmMixed, HmMixed),
    REXX_TYPED_ROUTINE(HmName, HmName),
    REXX_TYPED_ROUTINE(HmArgs, HmArgs),
    REXX_TYPED_ROUTINE(HmHelpers, HmHelpers),
    REXX_TYPED_ROUTINE(HmClassic, HmClassic),
    REXX_TYPED_ROUTINE(HmTableRows, HmTableRows),
    REXX_TYPED_ROUTINE(HmPackage, HmPackage),
    REXX_TYPED_ROUTINE(HmPlatform, HmPlatform),
#ifdef __cplusplus
    REXX_TYPED_ROUTINE(HmExists, HmExists),
    REXX_TYPED_ROUTINE(HmExistsAfterName, HmExistsAfterName),
    REXX_TYPED_ROUTINE(HmAssign, HmAssign),
    REXX_TYPED_ROUTINE(HmConvert, HmConvert),
    REXX_TYPED_ROUTINE(HmWiden, HmWiden),
#endif
    /* A classic row: the package loads with it. */
    REXX_CLASSIC_ROUTINE(HmClassicRow, HmClassicInfo),
    REXX_LAST_ROUTINE()
};

static RexxMethodEntry hmMethods[] = {
    REXX_METHOD(HmMSum0, HmMSum0),
    REXX_METHOD(HmMSum1, HmMSum1),
    REXX_METHOD(HmMSum2, HmMSum2),
    REXX_METHOD(HmMSum3, HmMSum3),
    REXX_METHOD(HmMSum4, HmMSum4),
    REXX_METHOD(HmMSum5, HmMSum5),
    REXX_METHOD(HmMSum6, HmMSum6),
    REXX_METHOD(HmMSum7, HmMSum7),
    REXX_METHOD(HmMSum8, HmMSum8),
    REXX_METHOD(HmMSum9, HmMSum9),
    REXX_METHOD(HmMSum10, HmMSum10),
    /* A row written with the entry form, under a name of its own. */
    REXX_METHOD_ENTRY(HmMRow, HmMSum2),
    REXX_METHOD(HmSelf, HmSelf),
    REXX_METHOD(HmSuper, HmSuper),
    REXX_METHOD(HmScope, HmScope),
    REXX_METHOD(HmCself, HmCself),
    REXX_METHOD(HmMessage, HmMessage),
    REXX_METHOD(HmMArgs, HmMArgs),
    REXX_LAST_METHOD()
};

static RexxPackageEntry hm_package_entry = {
    STANDARD_PACKAGE_HEADER
    REXX_INTERPRETER_4_0_0,
    "headermacros",
    NO_VERSION_YET,
    NULL,
    NULL,
    hmRoutines,
    hmMethods
};

OOREXX_GET_PACKAGE(hm);

/* The classic row is the last before the terminator. */
#define HM_CLASSIC_ROW (sizeof hmRoutines / sizeof hmRoutines[0] - 2)

/* The routine table's classic row and one typed row, as the table holds them:
 * style, name and whether the entry is the function given. */
RexxRoutine0(CSTRING, HmTableRows)
{
    static char out[96];
    const RexxRoutineEntry *classic = &hmRoutines[HM_CLASSIC_ROW];
    const RexxRoutineEntry *last = &hmRoutines[HM_CLASSIC_ROW + 1];
    snprintf(out, sizeof out, "%d %s %d %d %s %d %d %s %d | %d %s %d", hmRoutines[0].style,
             hmRoutines[0].name, hmRoutines[0].entryPoint == (void *)HmSum0, classic->style,
             classic->name, classic->entryPoint == (void *)HmClassicInfo, last->style,
             last->name == NULL ? "null" : last->name, last->entryPoint == NULL,
             hmMethods[0].style, hmMethods[0].name, hmMethods[0].entryPoint == (void *)HmMSum0);
    return out;
}

/* The package entry's header and version fields. */
RexxRoutine0(CSTRING, HmPackage)
{
    static char out[96];
    snprintf(out, sizeof out, "%d %d %d %s %d", hm_package_entry.size == (int)sizeof(RexxPackageEntry),
             hm_package_entry.apiVersion == REXX_PACKAGE_API_NO,
             hm_package_entry.requiredVersion == REXX_INTERPRETER_4_0_0,
             hm_package_entry.packageName, hm_package_entry.packageVersion == NULL);
    return out;
}
