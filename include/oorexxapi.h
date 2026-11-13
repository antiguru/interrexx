/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The object-oriented Rexx programming interface: the context structures an
 * extension or an embedding program receives, the tables of functions each
 * context reaches, and the structures those functions take.
 *
 * Each section between a BEGIN GENERATED and an END GENERATED line is written
 * from interrexx's Rust definitions by rust/crates/interrexx-api/src/header.rs:
 * change those, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1. The rest of this file is written by hand.
 */

#ifndef INTERREXX_OOREXXAPI_H
#define INTERREXX_OOREXXAPI_H

#include "rexx.h"

#ifdef __cplusplus
extern "C" {
#endif

/* BEGIN GENERATED contexts */
#ifdef __cplusplus
struct RexxInstance_;
typedef RexxInstance_ RexxInstance;
struct RexxThreadContext_;
typedef RexxThreadContext_ RexxThreadContext;
struct RexxMethodContext_;
typedef RexxMethodContext_ RexxMethodContext;
struct RexxCallContext_;
typedef RexxCallContext_ RexxCallContext;
struct RexxExitContext_;
typedef RexxExitContext_ RexxExitContext;
struct RexxIORedirectorContext_;
typedef RexxIORedirectorContext_ RexxIORedirectorContext;
#else
typedef const struct RexxInstance_ *RexxInstance;
typedef const struct RexxThreadContext_ *RexxThreadContext;
typedef const struct RexxMethodContext_ *RexxMethodContext;
typedef const struct RexxCallContext_ *RexxCallContext;
typedef const struct RexxExitContext_ *RexxExitContext;
typedef const struct RexxIORedirectorContext_ *RexxIORedirectorContext;
#endif
/* END GENERATED contexts */

/* The hooks a package runs when it is loaded and when it is unloaded. */
typedef void (RexxEntry *RexxPackageLoader)(RexxThreadContext *);
typedef void (RexxEntry *RexxPackageUnloader)(RexxThreadContext *);

/* A context exit handler: it receives the exit context, the exit's function
 * and subfunction codes and the parameter block, and answers whether it
 * handled the exit. */
typedef int REXXENTRY RexxContextExitHandler(RexxExitContext *, int, int, PEXIT);

/* BEGIN GENERATED structs */
typedef struct _ValueDescriptor ValueDescriptor;
typedef struct _RexxContextExit RexxContextExit;
typedef struct _RexxRoutineEntry RexxRoutineEntry;
typedef struct _RexxMethodEntry RexxMethodEntry;
typedef struct _RexxPackageEntry RexxPackageEntry;

struct _ValueDescriptor {
    union {
        RexxArrayObject value_ARGLIST;
        CSTRING value_NAME;
        RexxObjectPtr value_SCOPE;
        POINTER value_CSELF;
        RexxClassObject value_OSELF;
        RexxClassObject value_SUPER;
        RexxObjectPtr value_RexxObjectPtr;
        RexxClassObject value_RexxClassObject;
        int value_int;
        wholenumber_t value_wholenumber_t;
        stringsize_t value_stringsize_t;
        logical_t value_logical_t;
        double value_double;
        CSTRING value_CSTRING;
        POINTER value_POINTER;
        RexxStringObject value_RexxStringObject;
        float value_float;
        int8_t value_int8_t;
        int16_t value_int16_t;
        int32_t value_int32_t;
        int32_t value___int32_t;
        int64_t value_int64_t;
        int64_t value___int64_t;
        uint8_t value_uint8_t;
        uint16_t value_uint16_t;
        uint32_t value_uint32_t;
        uint32_t value___uint32_t;
        uint64_t value_uint64_t;
        uint64_t value___uint64_t;
        intptr_t value_intptr_t;
        uintptr_t value_uintptr_t;
        uintptr_t value___uintptr_t;
        size_t value_size_t;
        ssize_t value_ssize_t;
        RexxArrayObject value_RexxArrayObject;
        RexxStemObject value_RexxStemObject;
        POINTER value_POINTERSTRING;
        RexxMutableBufferObject value_RexxMutableBufferObject;
        RexxVariableReferenceObject value_RexxVariableReferenceObject;
        wholenumber_t value_positive_wholenumber_t;
        wholenumber_t value_nonnegative_wholenumber_t;
        RexxObjectPtr value_OPTIONAL_RexxObjectPtr;
        int value_OPTIONAL_int;
        wholenumber_t value_OPTIONAL_wholenumber_t;
        stringsize_t value_OPTIONAL_stringsize_t;
        logical_t value_OPTIONAL_logical_t;
        double value_OPTIONAL_double;
        CSTRING value_OPTIONAL_CSTRING;
        RexxClassObject value_OPTIONAL_RexxClassObject;
        POINTER value_OPTIONAL_POINTER;
        RexxStringObject value_OPTIONAL_RexxStringObject;
        float value_OPTIONAL_float;
        int8_t value_OPTIONAL_int8_t;
        int16_t value_OPTIONAL_int16_t;
        int32_t value_OPTIONAL_int32_t;
        int64_t value_OPTIONAL_int64_t;
        uint8_t value_OPTIONAL_uint8_t;
        uint16_t value_OPTIONAL_uint16_t;
        uint32_t value_OPTIONAL_uint32_t;
        uint64_t value_OPTIONAL_uint64_t;
        intptr_t value_OPTIONAL_intptr_t;
        uintptr_t value_OPTIONAL_uintptr_t;
        ssize_t value_OPTIONAL_ssize_t;
        size_t value_OPTIONAL_size_t;
        RexxArrayObject value_OPTIONAL_RexxArrayObject;
        RexxStemObject value_OPTIONAL_RexxStemObject;
        POINTER value_OPTIONAL_POINTERSTRING;
        RexxMutableBufferObject value_OPTIONAL_RexxMutableBufferObject;
        wholenumber_t value_OPTIONAL_positive_wholenumber_t;
        wholenumber_t value_OPTIONAL_nonnegative_wholenumber_t;
    } value;
    uint16_t type;
    uint16_t flags;
};

typedef struct {
    wholenumber_t code;
    wholenumber_t rc;
    size_t position;
    RexxStringObject conditionName;
    RexxStringObject message;
    RexxStringObject errortext;
    RexxStringObject program;
    RexxStringObject description;
    RexxArrayObject additional;
} RexxCondition;

struct _RexxContextExit {
    RexxContextExitHandler *handler;
    int sysexit_code;
};

struct _RexxRoutineEntry {
    int style;
    int reserved1;
    const char *name;
    void *entryPoint;
    int reserved2;
    int reserved3;
};

struct _RexxMethodEntry {
    int style;
    int reserved1;
    const char *name;
    void *entryPoint;
    int reserved2;
    int reserved3;
};

struct _RexxPackageEntry {
    int size;
    int apiVersion;
    int requiredVersion;
    const char *packageName;
    const char *packageVersion;
    RexxPackageLoader loader;
    RexxPackageUnloader unloader;
    RexxRoutineEntry *routines;
    RexxMethodEntry *methods;
};
/* END GENERATED structs */

/* BEGIN GENERATED tables */
typedef struct {
    wholenumber_t interfaceVersion;
    void (RexxEntry *Terminate)(RexxInstance *);
    logical_t (RexxEntry *AttachThread)(RexxInstance *, RexxThreadContext **);
    size_t (RexxEntry *InterpreterVersion)(RexxInstance *);
    size_t (RexxEntry *LanguageLevel)(RexxInstance *);
    void (RexxEntry *Halt)(RexxInstance *);
    void (RexxEntry *SetTrace)(RexxInstance *, logical_t);
    void (RexxEntry *AddCommandEnvironment)(RexxInstance *, const char *, REXXPFN, int);
    logical_t (RexxEntry *StackCommandEnvironment)(RexxInstance *, const char *, REXXPFN, int);
} RexxInstanceInterface;

typedef struct {
    wholenumber_t interfaceVersion;
    void (RexxEntry *DetachThread)(RexxThreadContext *);
    void (RexxEntry *HaltThread)(RexxThreadContext *);
    void (RexxEntry *SetThreadTrace)(RexxThreadContext *, logical_t);
    RexxObjectPtr (RexxEntry *RequestGlobalReference)(RexxThreadContext *, RexxObjectPtr);
    void (RexxEntry *ReleaseGlobalReference)(RexxThreadContext *, RexxObjectPtr);
    void (RexxEntry *ReleaseLocalReference)(RexxThreadContext *, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *SendMessage)(RexxThreadContext *, RexxObjectPtr, CSTRING, RexxArrayObject);
    RexxObjectPtr (RexxEntry *SendMessage0)(RexxThreadContext *, RexxObjectPtr, CSTRING);
    RexxObjectPtr (RexxEntry *SendMessage1)(RexxThreadContext *, RexxObjectPtr, CSTRING, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *SendMessage2)(RexxThreadContext *, RexxObjectPtr, CSTRING, RexxObjectPtr, RexxObjectPtr);
    RexxDirectoryObject (RexxEntry *GetLocalEnvironment)(RexxThreadContext *);
    RexxDirectoryObject (RexxEntry *GetGlobalEnvironment)(RexxThreadContext *);
    logical_t (RexxEntry *IsInstanceOf)(RexxThreadContext *, RexxObjectPtr, RexxClassObject);
    logical_t (RexxEntry *IsOfType)(RexxThreadContext *, RexxObjectPtr, CSTRING);
    logical_t (RexxEntry *HasMethod)(RexxThreadContext *, RexxObjectPtr, CSTRING);
    RexxPackageObject (RexxEntry *LoadPackage)(RexxThreadContext *, CSTRING);
    RexxPackageObject (RexxEntry *LoadPackageFromData)(RexxThreadContext *, CSTRING, CSTRING, size_t);
    logical_t (RexxEntry *LoadLibrary)(RexxThreadContext *, CSTRING);
    logical_t (RexxEntry *RegisterLibrary)(RexxThreadContext *, CSTRING, RexxPackageEntry *);
    RexxClassObject (RexxEntry *FindClass)(RexxThreadContext *, CSTRING);
    RexxClassObject (RexxEntry *FindPackageClass)(RexxThreadContext *, RexxPackageObject, CSTRING);
    RexxDirectoryObject (RexxEntry *GetPackageRoutines)(RexxThreadContext *, RexxPackageObject);
    RexxDirectoryObject (RexxEntry *GetPackagePublicRoutines)(RexxThreadContext *, RexxPackageObject);
    RexxDirectoryObject (RexxEntry *GetPackageClasses)(RexxThreadContext *, RexxPackageObject);
    RexxDirectoryObject (RexxEntry *GetPackagePublicClasses)(RexxThreadContext *, RexxPackageObject);
    RexxDirectoryObject (RexxEntry *GetPackageMethods)(RexxThreadContext *, RexxPackageObject);
    RexxObjectPtr (RexxEntry *CallRoutine)(RexxThreadContext *, RexxRoutineObject, RexxArrayObject);
    RexxObjectPtr (RexxEntry *CallProgram)(RexxThreadContext *, CSTRING, RexxArrayObject);
    RexxMethodObject (RexxEntry *NewMethod)(RexxThreadContext *, CSTRING, CSTRING, size_t);
    RexxRoutineObject (RexxEntry *NewRoutine)(RexxThreadContext *, CSTRING, CSTRING, size_t);
    logical_t (RexxEntry *IsRoutine)(RexxThreadContext *, RexxObjectPtr);
    logical_t (RexxEntry *IsMethod)(RexxThreadContext *, RexxObjectPtr);
    RexxPackageObject (RexxEntry *GetRoutinePackage)(RexxThreadContext *, RexxRoutineObject);
    RexxPackageObject (RexxEntry *GetMethodPackage)(RexxThreadContext *, RexxMethodObject);
    POINTER (RexxEntry *ObjectToCSelf)(RexxThreadContext *, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *WholeNumberToObject)(RexxThreadContext *, wholenumber_t);
    RexxObjectPtr (RexxEntry *UintptrToObject)(RexxThreadContext *, uintptr_t);
    RexxObjectPtr (RexxEntry *IntptrToObject)(RexxThreadContext *, intptr_t);
    RexxObjectPtr (RexxEntry *ValueToObject)(RexxThreadContext *, ValueDescriptor *);
    RexxArrayObject (RexxEntry *ValuesToObject)(RexxThreadContext *, ValueDescriptor *, size_t);
    logical_t (RexxEntry *ObjectToValue)(RexxThreadContext *, RexxObjectPtr, ValueDescriptor *);
    RexxObjectPtr (RexxEntry *StringSizeToObject)(RexxThreadContext *, stringsize_t);
    logical_t (RexxEntry *ObjectToWholeNumber)(RexxThreadContext *, RexxObjectPtr, wholenumber_t *);
    logical_t (RexxEntry *ObjectToStringSize)(RexxThreadContext *, RexxObjectPtr, stringsize_t *);
    RexxObjectPtr (RexxEntry *Int64ToObject)(RexxThreadContext *, int64_t);
    RexxObjectPtr (RexxEntry *UnsignedInt64ToObject)(RexxThreadContext *, uint64_t);
    logical_t (RexxEntry *ObjectToInt64)(RexxThreadContext *, RexxObjectPtr, int64_t *);
    logical_t (RexxEntry *ObjectToUnsignedInt64)(RexxThreadContext *, RexxObjectPtr, uint64_t *);
    RexxObjectPtr (RexxEntry *Int32ToObject)(RexxThreadContext *, int32_t);
    RexxObjectPtr (RexxEntry *UnsignedInt32ToObject)(RexxThreadContext *, uint32_t);
    logical_t (RexxEntry *ObjectToInt32)(RexxThreadContext *, RexxObjectPtr, int32_t *);
    logical_t (RexxEntry *ObjectToUnsignedInt32)(RexxThreadContext *, RexxObjectPtr, uint32_t *);
    logical_t (RexxEntry *ObjectToUintptr)(RexxThreadContext *, RexxObjectPtr, uintptr_t *);
    logical_t (RexxEntry *ObjectToIntptr)(RexxThreadContext *, RexxObjectPtr, intptr_t *);
    logical_t (RexxEntry *ObjectToLogical)(RexxThreadContext *, RexxObjectPtr, logical_t *);
    RexxObjectPtr (RexxEntry *LogicalToObject)(RexxThreadContext *, logical_t);
    RexxObjectPtr (RexxEntry *DoubleToObject)(RexxThreadContext *, double);
    RexxObjectPtr (RexxEntry *DoubleToObjectWithPrecision)(RexxThreadContext *, double, size_t);
    logical_t (RexxEntry *ObjectToDouble)(RexxThreadContext *, RexxObjectPtr, double *);
    RexxStringObject (RexxEntry *ObjectToString)(RexxThreadContext *, RexxObjectPtr);
    CSTRING (RexxEntry *ObjectToStringValue)(RexxThreadContext *, RexxObjectPtr);
    size_t (RexxEntry *StringGet)(RexxThreadContext *, RexxStringObject, size_t, POINTER, size_t);
    size_t (RexxEntry *StringLength)(RexxThreadContext *, RexxStringObject);
    CSTRING (RexxEntry *StringData)(RexxThreadContext *, RexxStringObject);
    RexxStringObject (RexxEntry *NewString)(RexxThreadContext *, CSTRING, size_t);
    RexxStringObject (RexxEntry *NewStringFromAsciiz)(RexxThreadContext *, CSTRING);
    RexxStringObject (RexxEntry *StringUpper)(RexxThreadContext *, RexxStringObject);
    RexxStringObject (RexxEntry *StringLower)(RexxThreadContext *, RexxStringObject);
    logical_t (RexxEntry *IsString)(RexxThreadContext *, RexxObjectPtr);
    RexxBufferStringObject (RexxEntry *NewBufferString)(RexxThreadContext *, size_t);
    size_t (RexxEntry *BufferStringLength)(RexxThreadContext *, RexxBufferStringObject);
    POINTER (RexxEntry *BufferStringData)(RexxThreadContext *, RexxBufferStringObject);
    RexxStringObject (RexxEntry *FinishBufferString)(RexxThreadContext *, RexxBufferStringObject, size_t);
    void (RexxEntry *DirectoryPut)(RexxThreadContext *, RexxDirectoryObject, RexxObjectPtr, CSTRING);
    RexxObjectPtr (RexxEntry *DirectoryAt)(RexxThreadContext *, RexxDirectoryObject, CSTRING);
    RexxObjectPtr (RexxEntry *DirectoryRemove)(RexxThreadContext *, RexxDirectoryObject, CSTRING);
    RexxDirectoryObject (RexxEntry *NewDirectory)(RexxThreadContext *);
    logical_t (RexxEntry *IsDirectory)(RexxThreadContext *, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *ArrayAt)(RexxThreadContext *, RexxArrayObject, size_t);
    void (RexxEntry *ArrayPut)(RexxThreadContext *, RexxArrayObject, RexxObjectPtr, size_t);
    size_t (RexxEntry *ArrayAppend)(RexxThreadContext *, RexxArrayObject, RexxObjectPtr);
    size_t (RexxEntry *ArrayAppendString)(RexxThreadContext *, RexxArrayObject, CSTRING, size_t);
    size_t (RexxEntry *ArraySize)(RexxThreadContext *, RexxArrayObject);
    size_t (RexxEntry *ArrayItems)(RexxThreadContext *, RexxArrayObject);
    size_t (RexxEntry *ArrayDimension)(RexxThreadContext *, RexxArrayObject);
    RexxArrayObject (RexxEntry *NewArray)(RexxThreadContext *, size_t);
    RexxArrayObject (RexxEntry *ArrayOfOne)(RexxThreadContext *, RexxObjectPtr);
    RexxArrayObject (RexxEntry *ArrayOfTwo)(RexxThreadContext *, RexxObjectPtr, RexxObjectPtr);
    RexxArrayObject (RexxEntry *ArrayOfThree)(RexxThreadContext *, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr);
    RexxArrayObject (RexxEntry *ArrayOfFour)(RexxThreadContext *, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr);
    logical_t (RexxEntry *IsArray)(RexxThreadContext *, RexxObjectPtr);
    POINTER (RexxEntry *BufferData)(RexxThreadContext *, RexxBufferObject);
    size_t (RexxEntry *BufferLength)(RexxThreadContext *, RexxBufferObject);
    RexxBufferObject (RexxEntry *NewBuffer)(RexxThreadContext *, size_t);
    logical_t (RexxEntry *IsBuffer)(RexxThreadContext *, RexxObjectPtr);
    POINTER (RexxEntry *PointerValue)(RexxThreadContext *, RexxPointerObject);
    RexxPointerObject (RexxEntry *NewPointer)(RexxThreadContext *, POINTER);
    logical_t (RexxEntry *IsPointer)(RexxThreadContext *, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *SupplierItem)(RexxThreadContext *, RexxSupplierObject);
    RexxObjectPtr (RexxEntry *SupplierIndex)(RexxThreadContext *, RexxSupplierObject);
    logical_t (RexxEntry *SupplierAvailable)(RexxThreadContext *, RexxSupplierObject);
    void (RexxEntry *SupplierNext)(RexxThreadContext *, RexxSupplierObject);
    RexxSupplierObject (RexxEntry *NewSupplier)(RexxThreadContext *, RexxArrayObject, RexxArrayObject);
    RexxStemObject (RexxEntry *NewStem)(RexxThreadContext *, CSTRING);
    void (RexxEntry *SetStemElement)(RexxThreadContext *, RexxStemObject, CSTRING, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *GetStemElement)(RexxThreadContext *, RexxStemObject, CSTRING);
    void (RexxEntry *DropStemElement)(RexxThreadContext *, RexxStemObject, CSTRING);
    void (RexxEntry *SetStemArrayElement)(RexxThreadContext *, RexxStemObject, size_t, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *GetStemArrayElement)(RexxThreadContext *, RexxStemObject, size_t);
    void (RexxEntry *DropStemArrayElement)(RexxThreadContext *, RexxStemObject, size_t);
    RexxDirectoryObject (RexxEntry *GetAllStemElements)(RexxThreadContext *, RexxStemObject);
    RexxObjectPtr (RexxEntry *GetStemValue)(RexxThreadContext *, RexxStemObject);
    logical_t (RexxEntry *IsStem)(RexxThreadContext *, RexxObjectPtr);
    void (RexxEntry *RaiseException0)(RexxThreadContext *, size_t);
    void (RexxEntry *RaiseException1)(RexxThreadContext *, size_t, RexxObjectPtr);
    void (RexxEntry *RaiseException2)(RexxThreadContext *, size_t, RexxObjectPtr, RexxObjectPtr);
    void (RexxEntry *RaiseException)(RexxThreadContext *, size_t, RexxArrayObject);
    void (RexxEntry *RaiseCondition)(RexxThreadContext *, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr);
    logical_t (RexxEntry *CheckCondition)(RexxThreadContext *);
    RexxDirectoryObject (RexxEntry *GetConditionInfo)(RexxThreadContext *);
    void (RexxEntry *DecodeConditionInfo)(RexxThreadContext *, RexxDirectoryObject, RexxCondition *);
    void (RexxEntry *ClearCondition)(RexxThreadContext *);
    RexxObjectPtr RexxNil;
    RexxObjectPtr RexxTrue;
    RexxObjectPtr RexxFalse;
    RexxStringObject RexxNullString;
    POINTER (RexxEntry *ObjectToCSelfScoped)(RexxThreadContext *, RexxObjectPtr, RexxObjectPtr);
    wholenumber_t (RexxEntry *DisplayCondition)(RexxThreadContext *);
    POINTER (RexxEntry *MutableBufferData)(RexxThreadContext *, RexxMutableBufferObject);
    size_t (RexxEntry *MutableBufferLength)(RexxThreadContext *, RexxMutableBufferObject);
    size_t (RexxEntry *SetMutableBufferLength)(RexxThreadContext *, RexxMutableBufferObject, size_t);
    RexxMutableBufferObject (RexxEntry *NewMutableBuffer)(RexxThreadContext *, size_t);
    logical_t (RexxEntry *IsMutableBuffer)(RexxThreadContext *, RexxObjectPtr);
    size_t (RexxEntry *MutableBufferCapacity)(RexxThreadContext *, RexxMutableBufferObject);
    POINTER (RexxEntry *SetMutableBufferCapacity)(RexxThreadContext *, RexxMutableBufferObject, size_t);
    RexxStringObject (RexxEntry *VariableReferenceName)(RexxThreadContext *, RexxVariableReferenceObject);
    RexxObjectPtr (RexxEntry *VariableReferenceValue)(RexxThreadContext *, RexxVariableReferenceObject);
    void (RexxEntry *SetVariableReferenceValue)(RexxThreadContext *, RexxVariableReferenceObject, RexxObjectPtr);
    logical_t (RexxEntry *IsVariableReference)(RexxThreadContext *, RexxObjectPtr);
    void (RexxEntry *StringTablePut)(RexxThreadContext *, RexxStringTableObject, RexxObjectPtr, CSTRING);
    RexxObjectPtr (RexxEntry *StringTableAt)(RexxThreadContext *, RexxStringTableObject, CSTRING);
    RexxObjectPtr (RexxEntry *StringTableRemove)(RexxThreadContext *, RexxStringTableObject, CSTRING);
    RexxStringTableObject (RexxEntry *NewStringTable)(RexxThreadContext *);
    logical_t (RexxEntry *IsStringTable)(RexxThreadContext *, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *SendMessageScoped)(RexxThreadContext *, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject);
    RexxInstance * (RexxEntry *GetInterpreterInstance)(RexxThreadContext *);
    void (RexxEntry *RaiseConditionObject)(RexxThreadContext *, RexxDirectoryObject);
} RexxThreadInterface;

typedef struct {
    wholenumber_t interfaceVersion;
    RexxArrayObject (RexxEntry *GetArguments)(RexxMethodContext *);
    RexxObjectPtr (RexxEntry *GetArgument)(RexxMethodContext *, size_t);
    CSTRING (RexxEntry *GetMessageName)(RexxMethodContext *);
    RexxMethodObject (RexxEntry *GetMethod)(RexxMethodContext *);
    RexxObjectPtr (RexxEntry *GetSelf)(RexxMethodContext *);
    RexxClassObject (RexxEntry *GetSuper)(RexxMethodContext *);
    RexxObjectPtr (RexxEntry *GetScope)(RexxMethodContext *);
    void (RexxEntry *SetObjectVariable)(RexxMethodContext *, CSTRING, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *GetObjectVariable)(RexxMethodContext *, CSTRING);
    void (RexxEntry *DropObjectVariable)(RexxMethodContext *, CSTRING);
    RexxObjectPtr (RexxEntry *ForwardMessage)(RexxMethodContext *, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject);
    void (RexxEntry *SetGuardOn)(RexxMethodContext *);
    void (RexxEntry *SetGuardOff)(RexxMethodContext *);
    RexxClassObject (RexxEntry *FindContextClass)(RexxMethodContext *, CSTRING);
    POINTER (RexxEntry *GetCSelf)(RexxMethodContext *);
    POINTER (RexxEntry *AllocateObjectMemory)(RexxMethodContext *, size_t);
    void (RexxEntry *FreeObjectMemory)(RexxMethodContext *, POINTER);
    POINTER (RexxEntry *ReallocateObjectMemory)(RexxMethodContext *, POINTER, size_t);
    RexxVariableReferenceObject (RexxEntry *GetObjectVariableReference)(RexxMethodContext *, CSTRING);
    RexxObjectPtr (RexxEntry *SetGuardOnWhenUpdated)(RexxMethodContext *, CSTRING);
    RexxObjectPtr (RexxEntry *SetGuardOffWhenUpdated)(RexxMethodContext *, CSTRING);
    void (RexxEntry *ThrowException0)(RexxMethodContext *, size_t);
    void (RexxEntry *ThrowException1)(RexxMethodContext *, size_t, RexxObjectPtr);
    void (RexxEntry *ThrowException2)(RexxMethodContext *, size_t, RexxObjectPtr, RexxObjectPtr);
    void (RexxEntry *ThrowException)(RexxMethodContext *, size_t, RexxArrayObject);
    void (RexxEntry *ThrowCondition)(RexxMethodContext *, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr);
} MethodContextInterface;

typedef struct {
    wholenumber_t interfaceVersion;
    RexxArrayObject (RexxEntry *GetArguments)(RexxCallContext *);
    RexxObjectPtr (RexxEntry *GetArgument)(RexxCallContext *, size_t);
    CSTRING (RexxEntry *GetRoutineName)(RexxCallContext *);
    RexxRoutineObject (RexxEntry *GetRoutine)(RexxCallContext *);
    void (RexxEntry *SetContextVariable)(RexxCallContext *, CSTRING, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *GetContextVariable)(RexxCallContext *, CSTRING);
    void (RexxEntry *DropContextVariable)(RexxCallContext *, CSTRING);
    RexxDirectoryObject (RexxEntry *GetAllContextVariables)(RexxCallContext *);
    RexxStemObject (RexxEntry *ResolveStemVariable)(RexxCallContext *, RexxObjectPtr);
    void (RexxEntry *InvalidRoutine)(RexxCallContext *);
    stringsize_t (RexxEntry *GetContextDigits)(RexxCallContext *);
    stringsize_t (RexxEntry *GetContextFuzz)(RexxCallContext *);
    logical_t (RexxEntry *GetContextForm)(RexxCallContext *);
    RexxObjectPtr (RexxEntry *GetCallerContext)(RexxCallContext *);
    RexxClassObject (RexxEntry *FindContextClass)(RexxCallContext *, CSTRING);
    RexxVariableReferenceObject (RexxEntry *GetContextVariableReference)(RexxCallContext *, CSTRING);
    void (RexxEntry *ThrowException0)(RexxCallContext *, size_t);
    void (RexxEntry *ThrowException1)(RexxCallContext *, size_t, RexxObjectPtr);
    void (RexxEntry *ThrowException2)(RexxCallContext *, size_t, RexxObjectPtr, RexxObjectPtr);
    void (RexxEntry *ThrowException)(RexxCallContext *, size_t, RexxArrayObject);
    void (RexxEntry *ThrowCondition)(RexxCallContext *, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr);
} CallContextInterface;

typedef struct {
    wholenumber_t interfaceVersion;
    void (RexxEntry *SetContextVariable)(RexxExitContext *, CSTRING, RexxObjectPtr);
    RexxObjectPtr (RexxEntry *GetContextVariable)(RexxExitContext *, CSTRING);
    void (RexxEntry *DropContextVariable)(RexxExitContext *, CSTRING);
    RexxDirectoryObject (RexxEntry *GetAllContextVariables)(RexxExitContext *);
    RexxObjectPtr (RexxEntry *GetCallerContext)(RexxExitContext *);
    RexxVariableReferenceObject (RexxEntry *GetContextVariableReference)(RexxExitContext *, CSTRING);
    void (RexxEntry *ThrowException0)(RexxExitContext *, size_t);
    void (RexxEntry *ThrowException1)(RexxExitContext *, size_t, RexxObjectPtr);
    void (RexxEntry *ThrowException2)(RexxExitContext *, size_t, RexxObjectPtr, RexxObjectPtr);
    void (RexxEntry *ThrowException)(RexxExitContext *, size_t, RexxArrayObject);
    void (RexxEntry *ThrowCondition)(RexxExitContext *, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr);
} ExitContextInterface;

typedef struct {
    wholenumber_t interfaceVersion;
    void (RexxEntry *ReadInput)(RexxIORedirectorContext *, CSTRING *, size_t *);
    void (RexxEntry *ReadInputBuffer)(RexxIORedirectorContext *, CSTRING *, size_t *);
    void (RexxEntry *WriteOutput)(RexxIORedirectorContext *, CSTRING, size_t);
    void (RexxEntry *WriteError)(RexxIORedirectorContext *, CSTRING, size_t);
    void (RexxEntry *WriteOutputBuffer)(RexxIORedirectorContext *, CSTRING, size_t);
    void (RexxEntry *WriteErrorBuffer)(RexxIORedirectorContext *, CSTRING, size_t);
    logical_t (RexxEntry *IsInputRedirected)(RexxIORedirectorContext *);
    logical_t (RexxEntry *IsOutputRedirected)(RexxIORedirectorContext *);
    logical_t (RexxEntry *IsErrorRedirected)(RexxIORedirectorContext *);
    logical_t (RexxEntry *AreOutputAndErrorSameTarget)(RexxIORedirectorContext *);
    logical_t (RexxEntry *IsRedirectionRequested)(RexxIORedirectorContext *);
} IORedirectorInterface;
/* END GENERATED tables */

/* BEGIN GENERATED context-structs */
struct RexxInstance_ {
    RexxInstanceInterface *functions;
    void *applicationData;
#ifdef __cplusplus
    void Terminate() { functions->Terminate(this); }
    logical_t AttachThread(RexxThreadContext **a1) { return functions->AttachThread(this, a1); }
    size_t InterpreterVersion() { return functions->InterpreterVersion(this); }
    size_t LanguageLevel() { return functions->LanguageLevel(this); }
    void Halt() { functions->Halt(this); }
    void SetTrace(logical_t a1) { functions->SetTrace(this, a1); }
    void AddCommandEnvironment(const char *a1, REXXPFN a2, int a3) { functions->AddCommandEnvironment(this, a1, a2, a3); }
    logical_t StackCommandEnvironment(const char *a1, REXXPFN a2, int a3) { return functions->StackCommandEnvironment(this, a1, a2, a3); }
#endif
};

struct RexxThreadContext_ {
    RexxInstance *instance;
    RexxThreadInterface *functions;
#ifdef __cplusplus
    void DetachThread() { functions->DetachThread(this); }
    void HaltThread() { functions->HaltThread(this); }
    void SetThreadTrace(logical_t a1) { functions->SetThreadTrace(this, a1); }
    RexxObjectPtr RequestGlobalReference(RexxObjectPtr a1) { return functions->RequestGlobalReference(this, a1); }
    void ReleaseGlobalReference(RexxObjectPtr a1) { functions->ReleaseGlobalReference(this, a1); }
    void ReleaseLocalReference(RexxObjectPtr a1) { functions->ReleaseLocalReference(this, a1); }
    RexxObjectPtr SendMessage(RexxObjectPtr a1, CSTRING a2, RexxArrayObject a3) { return functions->SendMessage(this, a1, a2, a3); }
    RexxObjectPtr SendMessage0(RexxObjectPtr a1, CSTRING a2) { return functions->SendMessage0(this, a1, a2); }
    RexxObjectPtr SendMessage1(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3) { return functions->SendMessage1(this, a1, a2, a3); }
    RexxObjectPtr SendMessage2(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3, RexxObjectPtr a4) { return functions->SendMessage2(this, a1, a2, a3, a4); }
    RexxDirectoryObject GetLocalEnvironment() { return functions->GetLocalEnvironment(this); }
    RexxDirectoryObject GetGlobalEnvironment() { return functions->GetGlobalEnvironment(this); }
    logical_t IsInstanceOf(RexxObjectPtr a1, RexxClassObject a2) { return functions->IsInstanceOf(this, a1, a2); }
    logical_t IsOfType(RexxObjectPtr a1, CSTRING a2) { return functions->IsOfType(this, a1, a2); }
    logical_t HasMethod(RexxObjectPtr a1, CSTRING a2) { return functions->HasMethod(this, a1, a2); }
    RexxPackageObject LoadPackage(CSTRING a1) { return functions->LoadPackage(this, a1); }
    RexxPackageObject LoadPackageFromData(CSTRING a1, CSTRING a2, size_t a3) { return functions->LoadPackageFromData(this, a1, a2, a3); }
    logical_t LoadLibrary(CSTRING a1) { return functions->LoadLibrary(this, a1); }
    logical_t RegisterLibrary(CSTRING a1, RexxPackageEntry *a2) { return functions->RegisterLibrary(this, a1, a2); }
    RexxClassObject FindClass(CSTRING a1) { return functions->FindClass(this, a1); }
    RexxClassObject FindPackageClass(RexxPackageObject a1, CSTRING a2) { return functions->FindPackageClass(this, a1, a2); }
    RexxDirectoryObject GetPackageRoutines(RexxPackageObject a1) { return functions->GetPackageRoutines(this, a1); }
    RexxDirectoryObject GetPackagePublicRoutines(RexxPackageObject a1) { return functions->GetPackagePublicRoutines(this, a1); }
    RexxDirectoryObject GetPackageClasses(RexxPackageObject a1) { return functions->GetPackageClasses(this, a1); }
    RexxDirectoryObject GetPackagePublicClasses(RexxPackageObject a1) { return functions->GetPackagePublicClasses(this, a1); }
    RexxDirectoryObject GetPackageMethods(RexxPackageObject a1) { return functions->GetPackageMethods(this, a1); }
    RexxObjectPtr CallRoutine(RexxRoutineObject a1, RexxArrayObject a2) { return functions->CallRoutine(this, a1, a2); }
    RexxObjectPtr CallProgram(CSTRING a1, RexxArrayObject a2) { return functions->CallProgram(this, a1, a2); }
    RexxMethodObject NewMethod(CSTRING a1, CSTRING a2, size_t a3) { return functions->NewMethod(this, a1, a2, a3); }
    RexxRoutineObject NewRoutine(CSTRING a1, CSTRING a2, size_t a3) { return functions->NewRoutine(this, a1, a2, a3); }
    logical_t IsRoutine(RexxObjectPtr a1) { return functions->IsRoutine(this, a1); }
    logical_t IsMethod(RexxObjectPtr a1) { return functions->IsMethod(this, a1); }
    RexxPackageObject GetRoutinePackage(RexxRoutineObject a1) { return functions->GetRoutinePackage(this, a1); }
    RexxPackageObject GetMethodPackage(RexxMethodObject a1) { return functions->GetMethodPackage(this, a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1) { return functions->ObjectToCSelf(this, a1); }
    RexxObjectPtr WholeNumberToObject(wholenumber_t a1) { return functions->WholeNumberToObject(this, a1); }
    RexxObjectPtr UintptrToObject(uintptr_t a1) { return functions->UintptrToObject(this, a1); }
    RexxObjectPtr IntptrToObject(intptr_t a1) { return functions->IntptrToObject(this, a1); }
    RexxObjectPtr ValueToObject(ValueDescriptor *a1) { return functions->ValueToObject(this, a1); }
    RexxArrayObject ValuesToObject(ValueDescriptor *a1, size_t a2) { return functions->ValuesToObject(this, a1, a2); }
    logical_t ObjectToValue(RexxObjectPtr a1, ValueDescriptor *a2) { return functions->ObjectToValue(this, a1, a2); }
    RexxObjectPtr StringSizeToObject(size_t a1) { return functions->StringSizeToObject(this, a1); }
    logical_t ObjectToWholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return functions->ObjectToWholeNumber(this, a1, a2); }
    logical_t ObjectToStringSize(RexxObjectPtr a1, size_t *a2) { return functions->ObjectToStringSize(this, a1, a2); }
    RexxObjectPtr Int64ToObject(int64_t a1) { return functions->Int64ToObject(this, a1); }
    RexxObjectPtr UnsignedInt64ToObject(uint64_t a1) { return functions->UnsignedInt64ToObject(this, a1); }
    logical_t ObjectToInt64(RexxObjectPtr a1, int64_t *a2) { return functions->ObjectToInt64(this, a1, a2); }
    logical_t ObjectToUnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return functions->ObjectToUnsignedInt64(this, a1, a2); }
    RexxObjectPtr Int32ToObject(int32_t a1) { return functions->Int32ToObject(this, a1); }
    RexxObjectPtr UnsignedInt32ToObject(uint32_t a1) { return functions->UnsignedInt32ToObject(this, a1); }
    logical_t ObjectToInt32(RexxObjectPtr a1, int32_t *a2) { return functions->ObjectToInt32(this, a1, a2); }
    logical_t ObjectToUnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return functions->ObjectToUnsignedInt32(this, a1, a2); }
    logical_t ObjectToUintptr(RexxObjectPtr a1, uintptr_t *a2) { return functions->ObjectToUintptr(this, a1, a2); }
    logical_t ObjectToIntptr(RexxObjectPtr a1, intptr_t *a2) { return functions->ObjectToIntptr(this, a1, a2); }
    logical_t ObjectToLogical(RexxObjectPtr a1, logical_t *a2) { return functions->ObjectToLogical(this, a1, a2); }
    RexxObjectPtr LogicalToObject(logical_t a1) { return functions->LogicalToObject(this, a1); }
    RexxObjectPtr DoubleToObject(double a1) { return functions->DoubleToObject(this, a1); }
    RexxObjectPtr DoubleToObjectWithPrecision(double a1, size_t a2) { return functions->DoubleToObjectWithPrecision(this, a1, a2); }
    logical_t ObjectToDouble(RexxObjectPtr a1, double *a2) { return functions->ObjectToDouble(this, a1, a2); }
    RexxStringObject ObjectToString(RexxObjectPtr a1) { return functions->ObjectToString(this, a1); }
    CSTRING ObjectToStringValue(RexxObjectPtr a1) { return functions->ObjectToStringValue(this, a1); }
    size_t StringGet(RexxStringObject a1, size_t a2, POINTER a3, size_t a4) { return functions->StringGet(this, a1, a2, a3, a4); }
    size_t StringLength(RexxStringObject a1) { return functions->StringLength(this, a1); }
    CSTRING StringData(RexxStringObject a1) { return functions->StringData(this, a1); }
    RexxStringObject NewString(CSTRING a1, size_t a2) { return functions->NewString(this, a1, a2); }
    RexxStringObject NewStringFromAsciiz(CSTRING a1) { return functions->NewStringFromAsciiz(this, a1); }
    RexxStringObject StringUpper(RexxStringObject a1) { return functions->StringUpper(this, a1); }
    RexxStringObject StringLower(RexxStringObject a1) { return functions->StringLower(this, a1); }
    logical_t IsString(RexxObjectPtr a1) { return functions->IsString(this, a1); }
    RexxBufferStringObject NewBufferString(size_t a1) { return functions->NewBufferString(this, a1); }
    size_t BufferStringLength(RexxBufferStringObject a1) { return functions->BufferStringLength(this, a1); }
    POINTER BufferStringData(RexxBufferStringObject a1) { return functions->BufferStringData(this, a1); }
    RexxStringObject FinishBufferString(RexxBufferStringObject a1, size_t a2) { return functions->FinishBufferString(this, a1, a2); }
    void DirectoryPut(RexxDirectoryObject a1, RexxObjectPtr a2, CSTRING a3) { functions->DirectoryPut(this, a1, a2, a3); }
    RexxObjectPtr DirectoryAt(RexxDirectoryObject a1, CSTRING a2) { return functions->DirectoryAt(this, a1, a2); }
    RexxObjectPtr DirectoryRemove(RexxDirectoryObject a1, CSTRING a2) { return functions->DirectoryRemove(this, a1, a2); }
    RexxDirectoryObject NewDirectory() { return functions->NewDirectory(this); }
    logical_t IsDirectory(RexxObjectPtr a1) { return functions->IsDirectory(this, a1); }
    RexxObjectPtr ArrayAt(RexxArrayObject a1, size_t a2) { return functions->ArrayAt(this, a1, a2); }
    void ArrayPut(RexxArrayObject a1, RexxObjectPtr a2, size_t a3) { functions->ArrayPut(this, a1, a2, a3); }
    size_t ArrayAppend(RexxArrayObject a1, RexxObjectPtr a2) { return functions->ArrayAppend(this, a1, a2); }
    size_t ArrayAppendString(RexxArrayObject a1, CSTRING a2, size_t a3) { return functions->ArrayAppendString(this, a1, a2, a3); }
    size_t ArraySize(RexxArrayObject a1) { return functions->ArraySize(this, a1); }
    size_t ArrayItems(RexxArrayObject a1) { return functions->ArrayItems(this, a1); }
    size_t ArrayDimension(RexxArrayObject a1) { return functions->ArrayDimension(this, a1); }
    RexxArrayObject NewArray(size_t a1) { return functions->NewArray(this, a1); }
    RexxArrayObject ArrayOfOne(RexxObjectPtr a1) { return functions->ArrayOfOne(this, a1); }
    RexxArrayObject ArrayOfTwo(RexxObjectPtr a1, RexxObjectPtr a2) { return functions->ArrayOfTwo(this, a1, a2); }
    RexxArrayObject ArrayOfThree(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return functions->ArrayOfThree(this, a1, a2, a3); }
    RexxArrayObject ArrayOfFour(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return functions->ArrayOfFour(this, a1, a2, a3, a4); }
    logical_t IsArray(RexxObjectPtr a1) { return functions->IsArray(this, a1); }
    POINTER BufferData(RexxBufferObject a1) { return functions->BufferData(this, a1); }
    size_t BufferLength(RexxBufferObject a1) { return functions->BufferLength(this, a1); }
    RexxBufferObject NewBuffer(wholenumber_t a1) { return functions->NewBuffer(this, a1); }
    logical_t IsBuffer(RexxObjectPtr a1) { return functions->IsBuffer(this, a1); }
    POINTER PointerValue(RexxPointerObject a1) { return functions->PointerValue(this, a1); }
    RexxPointerObject NewPointer(POINTER a1) { return functions->NewPointer(this, a1); }
    logical_t IsPointer(RexxObjectPtr a1) { return functions->IsPointer(this, a1); }
    RexxObjectPtr SupplierItem(RexxSupplierObject a1) { return functions->SupplierItem(this, a1); }
    RexxObjectPtr SupplierIndex(RexxSupplierObject a1) { return functions->SupplierIndex(this, a1); }
    logical_t SupplierAvailable(RexxSupplierObject a1) { return functions->SupplierAvailable(this, a1); }
    void SupplierNext(RexxSupplierObject a1) { functions->SupplierNext(this, a1); }
    RexxSupplierObject NewSupplier(RexxArrayObject a1, RexxArrayObject a2) { return functions->NewSupplier(this, a1, a2); }
    RexxStemObject NewStem(CSTRING a1) { return functions->NewStem(this, a1); }
    void SetStemElement(RexxStemObject a1, CSTRING a2, RexxObjectPtr a3) { functions->SetStemElement(this, a1, a2, a3); }
    RexxObjectPtr GetStemElement(RexxStemObject a1, CSTRING a2) { return functions->GetStemElement(this, a1, a2); }
    void DropStemElement(RexxStemObject a1, CSTRING a2) { functions->DropStemElement(this, a1, a2); }
    void SetStemArrayElement(RexxStemObject a1, size_t a2, RexxObjectPtr a3) { functions->SetStemArrayElement(this, a1, a2, a3); }
    RexxObjectPtr GetStemArrayElement(RexxStemObject a1, size_t a2) { return functions->GetStemArrayElement(this, a1, a2); }
    void DropStemArrayElement(RexxStemObject a1, size_t a2) { functions->DropStemArrayElement(this, a1, a2); }
    RexxDirectoryObject GetAllStemElements(RexxStemObject a1) { return functions->GetAllStemElements(this, a1); }
    RexxObjectPtr GetStemValue(RexxStemObject a1) { return functions->GetStemValue(this, a1); }
    logical_t IsStem(RexxObjectPtr a1) { return functions->IsStem(this, a1); }
    void RaiseException0(size_t a1) { functions->RaiseException0(this, a1); }
    void RaiseException1(size_t a1, RexxObjectPtr a2) { functions->RaiseException1(this, a1, a2); }
    void RaiseException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { functions->RaiseException2(this, a1, a2, a3); }
    void RaiseException(size_t a1, RexxArrayObject a2) { functions->RaiseException(this, a1, a2); }
    void RaiseCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { functions->RaiseCondition(this, a1, a2, a3, a4); }
    logical_t CheckCondition() { return functions->CheckCondition(this); }
    RexxDirectoryObject GetConditionInfo() { return functions->GetConditionInfo(this); }
    void DecodeConditionInfo(RexxDirectoryObject a1, RexxCondition *a2) { functions->DecodeConditionInfo(this, a1, a2); }
    void ClearCondition() { functions->ClearCondition(this); }
    wholenumber_t DisplayCondition() { return functions->DisplayCondition(this); }
    POINTER MutableBufferData(RexxMutableBufferObject a1) { return functions->MutableBufferData(this, a1); }
    size_t MutableBufferLength(RexxMutableBufferObject a1) { return functions->MutableBufferLength(this, a1); }
    size_t SetMutableBufferLength(RexxMutableBufferObject a1, size_t a2) { return functions->SetMutableBufferLength(this, a1, a2); }
    RexxMutableBufferObject NewMutableBuffer(wholenumber_t a1) { return functions->NewMutableBuffer(this, a1); }
    logical_t IsMutableBuffer(RexxObjectPtr a1) { return functions->IsMutableBuffer(this, a1); }
    size_t MutableBufferCapacity(RexxMutableBufferObject a1) { return functions->MutableBufferCapacity(this, a1); }
    POINTER SetMutableBufferCapacity(RexxMutableBufferObject a1, size_t a2) { return functions->SetMutableBufferCapacity(this, a1, a2); }
    RexxStringObject VariableReferenceName(RexxVariableReferenceObject a1) { return functions->VariableReferenceName(this, a1); }
    RexxObjectPtr VariableReferenceValue(RexxVariableReferenceObject a1) { return functions->VariableReferenceValue(this, a1); }
    void SetVariableReferenceValue(RexxVariableReferenceObject a1, RexxObjectPtr a2) { functions->SetVariableReferenceValue(this, a1, a2); }
    logical_t IsVariableReference(RexxObjectPtr a1) { return functions->IsVariableReference(this, a1); }
    void StringTablePut(RexxStringTableObject a1, RexxObjectPtr a2, CSTRING a3) { functions->StringTablePut(this, a1, a2, a3); }
    RexxObjectPtr StringTableAt(RexxStringTableObject a1, CSTRING a2) { return functions->StringTableAt(this, a1, a2); }
    RexxObjectPtr StringTableRemove(RexxStringTableObject a1, CSTRING a2) { return functions->StringTableRemove(this, a1, a2); }
    RexxStringTableObject NewStringTable() { return functions->NewStringTable(this); }
    logical_t IsStringTable(RexxObjectPtr a1) { return functions->IsStringTable(this, a1); }
    RexxObjectPtr SendMessageScoped(RexxObjectPtr a1, CSTRING a2, RexxClassObject a3, RexxArrayObject a4) { return functions->SendMessageScoped(this, a1, a2, a3, a4); }
    RexxInstance *GetInterpreterInstance() { return functions->GetInterpreterInstance(this); }
    void RaiseConditionObject(RexxDirectoryObject a1) { functions->RaiseConditionObject(this, a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1, RexxObjectPtr a2) { return functions->ObjectToCSelfScoped(this, a1, a2); }
    RexxObjectPtr WholeNumber(wholenumber_t a1) { return functions->WholeNumberToObject(this, a1); }
    RexxObjectPtr Uintptr(uintptr_t a1) { return functions->UintptrToObject(this, a1); }
    RexxObjectPtr Intptr(intptr_t a1) { return functions->IntptrToObject(this, a1); }
    RexxObjectPtr StringSize(size_t a1) { return functions->StringSizeToObject(this, a1); }
    logical_t WholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return functions->ObjectToWholeNumber(this, a1, a2); }
    logical_t StringSize(RexxObjectPtr a1, size_t *a2) { return functions->ObjectToStringSize(this, a1, a2); }
    RexxObjectPtr Int64(int64_t a1) { return functions->Int64ToObject(this, a1); }
    RexxObjectPtr UnsignedInt64(uint64_t a1) { return functions->UnsignedInt64ToObject(this, a1); }
    logical_t Int64(RexxObjectPtr a1, int64_t *a2) { return functions->ObjectToInt64(this, a1, a2); }
    logical_t UnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return functions->ObjectToUnsignedInt64(this, a1, a2); }
    RexxObjectPtr Int32(int32_t a1) { return functions->Int32ToObject(this, a1); }
    RexxObjectPtr UnsignedInt32(uint32_t a1) { return functions->UnsignedInt32ToObject(this, a1); }
    logical_t Int32(RexxObjectPtr a1, int32_t *a2) { return functions->ObjectToInt32(this, a1, a2); }
    logical_t UnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return functions->ObjectToUnsignedInt32(this, a1, a2); }
    logical_t Uintptr(RexxObjectPtr a1, uintptr_t *a2) { return functions->ObjectToUintptr(this, a1, a2); }
    logical_t Intptr(RexxObjectPtr a1, intptr_t *a2) { return functions->ObjectToIntptr(this, a1, a2); }
    logical_t Logical(RexxObjectPtr a1, logical_t *a2) { return functions->ObjectToLogical(this, a1, a2); }
    RexxObjectPtr Logical(logical_t a1) { return functions->LogicalToObject(this, a1); }
    RexxObjectPtr Double(double a1) { return functions->DoubleToObject(this, a1); }
    logical_t Double(RexxObjectPtr a1, double *a2) { return functions->ObjectToDouble(this, a1, a2); }
    CSTRING CString(RexxObjectPtr a1) { return functions->ObjectToStringValue(this, a1); }
    RexxStringObject String(CSTRING a1, size_t a2) { return functions->NewString(this, a1, a2); }
    RexxStringObject String(CSTRING a1) { return functions->NewStringFromAsciiz(this, a1); }
    RexxStringObject CString(CSTRING a1) { return functions->NewStringFromAsciiz(this, a1); }
    RexxArrayObject Array(RexxObjectPtr a1) { return functions->ArrayOfOne(this, a1); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2) { return functions->ArrayOfTwo(this, a1, a2); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return functions->ArrayOfThree(this, a1, a2, a3); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return functions->ArrayOfFour(this, a1, a2, a3, a4); }
    RexxObjectPtr Nil() { return functions->RexxNil; }
    RexxObjectPtr True() { return functions->RexxTrue; }
    RexxObjectPtr False() { return functions->RexxFalse; }
    RexxStringObject NullString() { return functions->RexxNullString; }
    size_t InterpreterVersion() { return instance->InterpreterVersion(); }
    size_t LanguageLevel() { return instance->LanguageLevel(); }
    void AddCommandEnvironment(const char *a1, REXXPFN a2, int a3) { instance->AddCommandEnvironment(a1, a2, a3); }
    logical_t StackCommandEnvironment(const char *a1, REXXPFN a2, int a3) { return instance->StackCommandEnvironment(a1, a2, a3); }
    POINTER GetApplicationData() { return instance->applicationData; }
#endif
};

struct RexxMethodContext_ {
    RexxThreadContext *threadContext;
    MethodContextInterface *functions;
    ValueDescriptor *arguments;
#ifdef __cplusplus
    RexxArrayObject GetArguments() { return functions->GetArguments(this); }
    RexxObjectPtr GetArgument(size_t a1) { return functions->GetArgument(this, a1); }
    CSTRING GetMessageName() { return functions->GetMessageName(this); }
    RexxMethodObject GetMethod() { return functions->GetMethod(this); }
    RexxObjectPtr GetSelf() { return functions->GetSelf(this); }
    RexxClassObject GetSuper() { return functions->GetSuper(this); }
    RexxObjectPtr GetScope() { return functions->GetScope(this); }
    void SetObjectVariable(CSTRING a1, RexxObjectPtr a2) { functions->SetObjectVariable(this, a1, a2); }
    RexxObjectPtr GetObjectVariable(CSTRING a1) { return functions->GetObjectVariable(this, a1); }
    void DropObjectVariable(CSTRING a1) { functions->DropObjectVariable(this, a1); }
    RexxObjectPtr ForwardMessage(RexxObjectPtr a1, CSTRING a2, RexxClassObject a3, RexxArrayObject a4) { return functions->ForwardMessage(this, a1, a2, a3, a4); }
    void SetGuardOn() { functions->SetGuardOn(this); }
    void SetGuardOff() { functions->SetGuardOff(this); }
    RexxClassObject FindContextClass(CSTRING a1) { return functions->FindContextClass(this, a1); }
    POINTER GetCSelf() { return functions->GetCSelf(this); }
    POINTER AllocateObjectMemory(size_t a1) { return functions->AllocateObjectMemory(this, a1); }
    void FreeObjectMemory(POINTER a1) { functions->FreeObjectMemory(this, a1); }
    POINTER ReallocateObjectMemory(POINTER a1, size_t a2) { return functions->ReallocateObjectMemory(this, a1, a2); }
    RexxVariableReferenceObject GetObjectVariableReference(CSTRING a1) { return functions->GetObjectVariableReference(this, a1); }
    RexxObjectPtr SetGuardOnWhenUpdated(CSTRING a1) { return functions->SetGuardOnWhenUpdated(this, a1); }
    RexxObjectPtr SetGuardOffWhenUpdated(CSTRING a1) { return functions->SetGuardOffWhenUpdated(this, a1); }
    void ThrowException0(size_t a1) { functions->ThrowException0(this, a1); }
    void ThrowException1(size_t a1, RexxObjectPtr a2) { functions->ThrowException1(this, a1, a2); }
    void ThrowException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { functions->ThrowException2(this, a1, a2, a3); }
    void ThrowException(size_t a1, RexxArrayObject a2) { functions->ThrowException(this, a1, a2); }
    void ThrowCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { functions->ThrowCondition(this, a1, a2, a3, a4); }
    RexxObjectPtr RequestGlobalReference(RexxObjectPtr a1) { return threadContext->RequestGlobalReference(a1); }
    void ReleaseGlobalReference(RexxObjectPtr a1) { threadContext->ReleaseGlobalReference(a1); }
    void ReleaseLocalReference(RexxObjectPtr a1) { threadContext->ReleaseLocalReference(a1); }
    RexxObjectPtr SendMessage(RexxObjectPtr a1, CSTRING a2, RexxArrayObject a3) { return threadContext->SendMessage(a1, a2, a3); }
    RexxObjectPtr SendMessage0(RexxObjectPtr a1, CSTRING a2) { return threadContext->SendMessage0(a1, a2); }
    RexxObjectPtr SendMessage1(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3) { return threadContext->SendMessage1(a1, a2, a3); }
    RexxObjectPtr SendMessage2(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->SendMessage2(a1, a2, a3, a4); }
    RexxDirectoryObject GetLocalEnvironment() { return threadContext->GetLocalEnvironment(); }
    RexxDirectoryObject GetGlobalEnvironment() { return threadContext->GetGlobalEnvironment(); }
    logical_t IsInstanceOf(RexxObjectPtr a1, RexxClassObject a2) { return threadContext->IsInstanceOf(a1, a2); }
    logical_t IsOfType(RexxObjectPtr a1, CSTRING a2) { return threadContext->IsOfType(a1, a2); }
    logical_t HasMethod(RexxObjectPtr a1, CSTRING a2) { return threadContext->HasMethod(a1, a2); }
    RexxPackageObject LoadPackage(CSTRING a1) { return threadContext->LoadPackage(a1); }
    RexxPackageObject LoadPackageFromData(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->LoadPackageFromData(a1, a2, a3); }
    logical_t LoadLibrary(CSTRING a1) { return threadContext->LoadLibrary(a1); }
    logical_t RegisterLibrary(CSTRING a1, RexxPackageEntry *a2) { return threadContext->RegisterLibrary(a1, a2); }
    RexxClassObject FindClass(CSTRING a1) { return threadContext->FindClass(a1); }
    RexxClassObject FindPackageClass(RexxPackageObject a1, CSTRING a2) { return threadContext->FindPackageClass(a1, a2); }
    RexxDirectoryObject GetPackageRoutines(RexxPackageObject a1) { return threadContext->GetPackageRoutines(a1); }
    RexxDirectoryObject GetPackagePublicRoutines(RexxPackageObject a1) { return threadContext->GetPackagePublicRoutines(a1); }
    RexxDirectoryObject GetPackageClasses(RexxPackageObject a1) { return threadContext->GetPackageClasses(a1); }
    RexxDirectoryObject GetPackagePublicClasses(RexxPackageObject a1) { return threadContext->GetPackagePublicClasses(a1); }
    RexxDirectoryObject GetPackageMethods(RexxPackageObject a1) { return threadContext->GetPackageMethods(a1); }
    RexxObjectPtr CallRoutine(RexxRoutineObject a1, RexxArrayObject a2) { return threadContext->CallRoutine(a1, a2); }
    RexxObjectPtr CallProgram(CSTRING a1, RexxArrayObject a2) { return threadContext->CallProgram(a1, a2); }
    RexxMethodObject NewMethod(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewMethod(a1, a2, a3); }
    RexxRoutineObject NewRoutine(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewRoutine(a1, a2, a3); }
    logical_t IsRoutine(RexxObjectPtr a1) { return threadContext->IsRoutine(a1); }
    logical_t IsMethod(RexxObjectPtr a1) { return threadContext->IsMethod(a1); }
    RexxPackageObject GetRoutinePackage(RexxRoutineObject a1) { return threadContext->GetRoutinePackage(a1); }
    RexxPackageObject GetMethodPackage(RexxMethodObject a1) { return threadContext->GetMethodPackage(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1) { return threadContext->ObjectToCSelf(a1); }
    RexxObjectPtr WholeNumberToObject(wholenumber_t a1) { return threadContext->WholeNumberToObject(a1); }
    RexxObjectPtr UintptrToObject(uintptr_t a1) { return threadContext->UintptrToObject(a1); }
    RexxObjectPtr IntptrToObject(intptr_t a1) { return threadContext->IntptrToObject(a1); }
    RexxObjectPtr ValueToObject(ValueDescriptor *a1) { return threadContext->ValueToObject(a1); }
    RexxArrayObject ValuesToObject(ValueDescriptor *a1, size_t a2) { return threadContext->ValuesToObject(a1, a2); }
    logical_t ObjectToValue(RexxObjectPtr a1, ValueDescriptor *a2) { return threadContext->ObjectToValue(a1, a2); }
    RexxObjectPtr StringSizeToObject(size_t a1) { return threadContext->StringSizeToObject(a1); }
    logical_t ObjectToWholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->ObjectToWholeNumber(a1, a2); }
    logical_t ObjectToStringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->ObjectToStringSize(a1, a2); }
    RexxObjectPtr Int64ToObject(int64_t a1) { return threadContext->Int64ToObject(a1); }
    RexxObjectPtr UnsignedInt64ToObject(uint64_t a1) { return threadContext->UnsignedInt64ToObject(a1); }
    logical_t ObjectToInt64(RexxObjectPtr a1, int64_t *a2) { return threadContext->ObjectToInt64(a1, a2); }
    logical_t ObjectToUnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->ObjectToUnsignedInt64(a1, a2); }
    RexxObjectPtr Int32ToObject(int32_t a1) { return threadContext->Int32ToObject(a1); }
    RexxObjectPtr UnsignedInt32ToObject(uint32_t a1) { return threadContext->UnsignedInt32ToObject(a1); }
    logical_t ObjectToInt32(RexxObjectPtr a1, int32_t *a2) { return threadContext->ObjectToInt32(a1, a2); }
    logical_t ObjectToUnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->ObjectToUnsignedInt32(a1, a2); }
    logical_t ObjectToUintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->ObjectToUintptr(a1, a2); }
    logical_t ObjectToIntptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->ObjectToIntptr(a1, a2); }
    logical_t ObjectToLogical(RexxObjectPtr a1, logical_t *a2) { return threadContext->ObjectToLogical(a1, a2); }
    RexxObjectPtr LogicalToObject(logical_t a1) { return threadContext->LogicalToObject(a1); }
    RexxObjectPtr DoubleToObject(double a1) { return threadContext->DoubleToObject(a1); }
    RexxObjectPtr DoubleToObjectWithPrecision(double a1, size_t a2) { return threadContext->DoubleToObjectWithPrecision(a1, a2); }
    logical_t ObjectToDouble(RexxObjectPtr a1, double *a2) { return threadContext->ObjectToDouble(a1, a2); }
    RexxStringObject ObjectToString(RexxObjectPtr a1) { return threadContext->ObjectToString(a1); }
    CSTRING ObjectToStringValue(RexxObjectPtr a1) { return threadContext->ObjectToStringValue(a1); }
    size_t StringGet(RexxStringObject a1, size_t a2, POINTER a3, size_t a4) { return threadContext->StringGet(a1, a2, a3, a4); }
    size_t StringLength(RexxStringObject a1) { return threadContext->StringLength(a1); }
    CSTRING StringData(RexxStringObject a1) { return threadContext->StringData(a1); }
    RexxStringObject NewString(CSTRING a1, size_t a2) { return threadContext->NewString(a1, a2); }
    RexxStringObject NewStringFromAsciiz(CSTRING a1) { return threadContext->NewStringFromAsciiz(a1); }
    RexxStringObject StringUpper(RexxStringObject a1) { return threadContext->StringUpper(a1); }
    RexxStringObject StringLower(RexxStringObject a1) { return threadContext->StringLower(a1); }
    logical_t IsString(RexxObjectPtr a1) { return threadContext->IsString(a1); }
    RexxBufferStringObject NewBufferString(size_t a1) { return threadContext->NewBufferString(a1); }
    size_t BufferStringLength(RexxBufferStringObject a1) { return threadContext->BufferStringLength(a1); }
    POINTER BufferStringData(RexxBufferStringObject a1) { return threadContext->BufferStringData(a1); }
    RexxStringObject FinishBufferString(RexxBufferStringObject a1, size_t a2) { return threadContext->FinishBufferString(a1, a2); }
    void DirectoryPut(RexxDirectoryObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->DirectoryPut(a1, a2, a3); }
    RexxObjectPtr DirectoryAt(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryAt(a1, a2); }
    RexxObjectPtr DirectoryRemove(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryRemove(a1, a2); }
    RexxDirectoryObject NewDirectory() { return threadContext->NewDirectory(); }
    logical_t IsDirectory(RexxObjectPtr a1) { return threadContext->IsDirectory(a1); }
    RexxObjectPtr ArrayAt(RexxArrayObject a1, size_t a2) { return threadContext->ArrayAt(a1, a2); }
    void ArrayPut(RexxArrayObject a1, RexxObjectPtr a2, size_t a3) { threadContext->ArrayPut(a1, a2, a3); }
    size_t ArrayAppend(RexxArrayObject a1, RexxObjectPtr a2) { return threadContext->ArrayAppend(a1, a2); }
    size_t ArrayAppendString(RexxArrayObject a1, CSTRING a2, size_t a3) { return threadContext->ArrayAppendString(a1, a2, a3); }
    size_t ArraySize(RexxArrayObject a1) { return threadContext->ArraySize(a1); }
    size_t ArrayItems(RexxArrayObject a1) { return threadContext->ArrayItems(a1); }
    size_t ArrayDimension(RexxArrayObject a1) { return threadContext->ArrayDimension(a1); }
    RexxArrayObject NewArray(size_t a1) { return threadContext->NewArray(a1); }
    RexxArrayObject ArrayOfOne(RexxObjectPtr a1) { return threadContext->ArrayOfOne(a1); }
    RexxArrayObject ArrayOfTwo(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ArrayOfTwo(a1, a2); }
    RexxArrayObject ArrayOfThree(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->ArrayOfThree(a1, a2, a3); }
    RexxArrayObject ArrayOfFour(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->ArrayOfFour(a1, a2, a3, a4); }
    logical_t IsArray(RexxObjectPtr a1) { return threadContext->IsArray(a1); }
    POINTER BufferData(RexxBufferObject a1) { return threadContext->BufferData(a1); }
    size_t BufferLength(RexxBufferObject a1) { return threadContext->BufferLength(a1); }
    RexxBufferObject NewBuffer(wholenumber_t a1) { return threadContext->NewBuffer(a1); }
    logical_t IsBuffer(RexxObjectPtr a1) { return threadContext->IsBuffer(a1); }
    POINTER PointerValue(RexxPointerObject a1) { return threadContext->PointerValue(a1); }
    RexxPointerObject NewPointer(POINTER a1) { return threadContext->NewPointer(a1); }
    logical_t IsPointer(RexxObjectPtr a1) { return threadContext->IsPointer(a1); }
    RexxObjectPtr SupplierItem(RexxSupplierObject a1) { return threadContext->SupplierItem(a1); }
    RexxObjectPtr SupplierIndex(RexxSupplierObject a1) { return threadContext->SupplierIndex(a1); }
    logical_t SupplierAvailable(RexxSupplierObject a1) { return threadContext->SupplierAvailable(a1); }
    void SupplierNext(RexxSupplierObject a1) { threadContext->SupplierNext(a1); }
    RexxSupplierObject NewSupplier(RexxArrayObject a1, RexxArrayObject a2) { return threadContext->NewSupplier(a1, a2); }
    RexxStemObject NewStem(CSTRING a1) { return threadContext->NewStem(a1); }
    void SetStemElement(RexxStemObject a1, CSTRING a2, RexxObjectPtr a3) { threadContext->SetStemElement(a1, a2, a3); }
    RexxObjectPtr GetStemElement(RexxStemObject a1, CSTRING a2) { return threadContext->GetStemElement(a1, a2); }
    void DropStemElement(RexxStemObject a1, CSTRING a2) { threadContext->DropStemElement(a1, a2); }
    void SetStemArrayElement(RexxStemObject a1, size_t a2, RexxObjectPtr a3) { threadContext->SetStemArrayElement(a1, a2, a3); }
    RexxObjectPtr GetStemArrayElement(RexxStemObject a1, size_t a2) { return threadContext->GetStemArrayElement(a1, a2); }
    void DropStemArrayElement(RexxStemObject a1, size_t a2) { threadContext->DropStemArrayElement(a1, a2); }
    RexxDirectoryObject GetAllStemElements(RexxStemObject a1) { return threadContext->GetAllStemElements(a1); }
    RexxObjectPtr GetStemValue(RexxStemObject a1) { return threadContext->GetStemValue(a1); }
    logical_t IsStem(RexxObjectPtr a1) { return threadContext->IsStem(a1); }
    void RaiseException0(size_t a1) { threadContext->RaiseException0(a1); }
    void RaiseException1(size_t a1, RexxObjectPtr a2) { threadContext->RaiseException1(a1, a2); }
    void RaiseException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { threadContext->RaiseException2(a1, a2, a3); }
    void RaiseException(size_t a1, RexxArrayObject a2) { threadContext->RaiseException(a1, a2); }
    void RaiseCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { threadContext->RaiseCondition(a1, a2, a3, a4); }
    logical_t CheckCondition() { return threadContext->CheckCondition(); }
    RexxDirectoryObject GetConditionInfo() { return threadContext->GetConditionInfo(); }
    void DecodeConditionInfo(RexxDirectoryObject a1, RexxCondition *a2) { threadContext->DecodeConditionInfo(a1, a2); }
    void ClearCondition() { threadContext->ClearCondition(); }
    logical_t DisplayCondition() { return threadContext->DisplayCondition(); }
    POINTER MutableBufferData(RexxMutableBufferObject a1) { return threadContext->MutableBufferData(a1); }
    size_t MutableBufferLength(RexxMutableBufferObject a1) { return threadContext->MutableBufferLength(a1); }
    size_t SetMutableBufferLength(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferLength(a1, a2); }
    RexxMutableBufferObject NewMutableBuffer(wholenumber_t a1) { return threadContext->NewMutableBuffer(a1); }
    logical_t IsMutableBuffer(RexxObjectPtr a1) { return threadContext->IsMutableBuffer(a1); }
    size_t MutableBufferCapacity(RexxMutableBufferObject a1) { return threadContext->MutableBufferCapacity(a1); }
    POINTER SetMutableBufferCapacity(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferCapacity(a1, a2); }
    RexxStringObject VariableReferenceName(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceName(a1); }
    RexxObjectPtr VariableReferenceValue(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceValue(a1); }
    void SetVariableReferenceValue(RexxVariableReferenceObject a1, RexxObjectPtr a2) { threadContext->SetVariableReferenceValue(a1, a2); }
    logical_t IsVariableReference(RexxObjectPtr a1) { return threadContext->IsVariableReference(a1); }
    void StringTablePut(RexxStringTableObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->StringTablePut(a1, a2, a3); }
    RexxObjectPtr StringTableAt(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableAt(a1, a2); }
    RexxObjectPtr StringTableRemove(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableRemove(a1, a2); }
    RexxStringTableObject NewStringTable() { return threadContext->NewStringTable(); }
    logical_t IsStringTable(RexxObjectPtr a1) { return threadContext->IsStringTable(a1); }
    RexxObjectPtr SendMessageScoped(RexxObjectPtr a1, CSTRING a2, RexxClassObject a3, RexxArrayObject a4) { return threadContext->SendMessageScoped(a1, a2, a3, a4); }
    RexxInstance *GetInterpreterInstance() { return threadContext->GetInterpreterInstance(); }
    void RaiseConditionObject(RexxDirectoryObject a1) { threadContext->RaiseConditionObject(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ObjectToCSelf(a1, a2); }
    RexxObjectPtr WholeNumber(wholenumber_t a1) { return threadContext->WholeNumber(a1); }
    RexxObjectPtr Uintptr(uintptr_t a1) { return threadContext->Uintptr(a1); }
    RexxObjectPtr Intptr(intptr_t a1) { return threadContext->Intptr(a1); }
    RexxObjectPtr StringSize(size_t a1) { return threadContext->StringSize(a1); }
    logical_t WholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->WholeNumber(a1, a2); }
    logical_t StringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->StringSize(a1, a2); }
    RexxObjectPtr Int64(int64_t a1) { return threadContext->Int64(a1); }
    RexxObjectPtr UnsignedInt64(uint64_t a1) { return threadContext->UnsignedInt64(a1); }
    logical_t Int64(RexxObjectPtr a1, int64_t *a2) { return threadContext->Int64(a1, a2); }
    logical_t UnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->UnsignedInt64(a1, a2); }
    RexxObjectPtr Int32(int32_t a1) { return threadContext->Int32(a1); }
    RexxObjectPtr UnsignedInt32(uint32_t a1) { return threadContext->UnsignedInt32(a1); }
    logical_t Int32(RexxObjectPtr a1, int32_t *a2) { return threadContext->Int32(a1, a2); }
    logical_t UnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->UnsignedInt32(a1, a2); }
    logical_t Uintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->Uintptr(a1, a2); }
    logical_t Intptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->Intptr(a1, a2); }
    logical_t Logical(RexxObjectPtr a1, logical_t *a2) { return threadContext->Logical(a1, a2); }
    RexxObjectPtr Logical(logical_t a1) { return threadContext->Logical(a1); }
    RexxObjectPtr Double(double a1) { return threadContext->Double(a1); }
    logical_t Double(RexxObjectPtr a1, double *a2) { return threadContext->Double(a1, a2); }
    CSTRING CString(RexxObjectPtr a1) { return threadContext->CString(a1); }
    RexxStringObject String(CSTRING a1, size_t a2) { return threadContext->String(a1, a2); }
    RexxStringObject String(CSTRING a1) { return threadContext->String(a1); }
    RexxStringObject CString(CSTRING a1) { return threadContext->CString(a1); }
    RexxArrayObject Array(RexxObjectPtr a1) { return threadContext->Array(a1); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->Array(a1, a2); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->Array(a1, a2, a3); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->Array(a1, a2, a3, a4); }
    RexxObjectPtr Nil() { return threadContext->Nil(); }
    RexxObjectPtr True() { return threadContext->True(); }
    RexxObjectPtr False() { return threadContext->False(); }
    RexxStringObject NullString() { return threadContext->NullString(); }
    size_t InterpreterVersion() { return threadContext->InterpreterVersion(); }
    size_t LanguageLevel() { return threadContext->LanguageLevel(); }
    void AddCommandEnvironment(const char *a1, REXXPFN a2, int a3) { threadContext->AddCommandEnvironment(a1, a2, a3); }
    logical_t StackCommandEnvironment(const char *a1, REXXPFN a2, int a3) { return threadContext->StackCommandEnvironment(a1, a2, a3); }
    POINTER GetApplicationData() { return threadContext->GetApplicationData(); }
#endif
};

struct RexxCallContext_ {
    RexxThreadContext *threadContext;
    CallContextInterface *functions;
    ValueDescriptor *arguments;
#ifdef __cplusplus
    RexxArrayObject GetArguments() { return functions->GetArguments(this); }
    RexxObjectPtr GetArgument(size_t a1) { return functions->GetArgument(this, a1); }
    CSTRING GetRoutineName() { return functions->GetRoutineName(this); }
    RexxRoutineObject GetRoutine() { return functions->GetRoutine(this); }
    void SetContextVariable(CSTRING a1, RexxObjectPtr a2) { functions->SetContextVariable(this, a1, a2); }
    RexxObjectPtr GetContextVariable(CSTRING a1) { return functions->GetContextVariable(this, a1); }
    void DropContextVariable(CSTRING a1) { functions->DropContextVariable(this, a1); }
    RexxDirectoryObject GetAllContextVariables() { return functions->GetAllContextVariables(this); }
    RexxStemObject ResolveStemVariable(RexxObjectPtr a1) { return functions->ResolveStemVariable(this, a1); }
    void InvalidRoutine() { functions->InvalidRoutine(this); }
    stringsize_t GetContextDigits() { return functions->GetContextDigits(this); }
    stringsize_t GetContextFuzz() { return functions->GetContextFuzz(this); }
    logical_t GetContextForm() { return functions->GetContextForm(this); }
    RexxObjectPtr GetCallerContext() { return functions->GetCallerContext(this); }
    RexxClassObject FindContextClass(CSTRING a1) { return functions->FindContextClass(this, a1); }
    RexxVariableReferenceObject GetContextVariableReference(CSTRING a1) { return functions->GetContextVariableReference(this, a1); }
    void ThrowException0(size_t a1) { functions->ThrowException0(this, a1); }
    void ThrowException1(size_t a1, RexxObjectPtr a2) { functions->ThrowException1(this, a1, a2); }
    void ThrowException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { functions->ThrowException2(this, a1, a2, a3); }
    void ThrowException(size_t a1, RexxArrayObject a2) { functions->ThrowException(this, a1, a2); }
    void ThrowCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { functions->ThrowCondition(this, a1, a2, a3, a4); }
    RexxObjectPtr RequestGlobalReference(RexxObjectPtr a1) { return threadContext->RequestGlobalReference(a1); }
    void ReleaseGlobalReference(RexxObjectPtr a1) { threadContext->ReleaseGlobalReference(a1); }
    void ReleaseLocalReference(RexxObjectPtr a1) { threadContext->ReleaseLocalReference(a1); }
    RexxObjectPtr SendMessage(RexxObjectPtr a1, CSTRING a2, RexxArrayObject a3) { return threadContext->SendMessage(a1, a2, a3); }
    RexxObjectPtr SendMessage0(RexxObjectPtr a1, CSTRING a2) { return threadContext->SendMessage0(a1, a2); }
    RexxObjectPtr SendMessage1(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3) { return threadContext->SendMessage1(a1, a2, a3); }
    RexxObjectPtr SendMessage2(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->SendMessage2(a1, a2, a3, a4); }
    RexxDirectoryObject GetLocalEnvironment() { return threadContext->GetLocalEnvironment(); }
    RexxDirectoryObject GetGlobalEnvironment() { return threadContext->GetGlobalEnvironment(); }
    logical_t IsInstanceOf(RexxObjectPtr a1, RexxClassObject a2) { return threadContext->IsInstanceOf(a1, a2); }
    logical_t IsOfType(RexxObjectPtr a1, CSTRING a2) { return threadContext->IsOfType(a1, a2); }
    logical_t HasMethod(RexxObjectPtr a1, CSTRING a2) { return threadContext->HasMethod(a1, a2); }
    RexxPackageObject LoadPackage(CSTRING a1) { return threadContext->LoadPackage(a1); }
    RexxPackageObject LoadPackageFromData(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->LoadPackageFromData(a1, a2, a3); }
    logical_t LoadLibrary(CSTRING a1) { return threadContext->LoadLibrary(a1); }
    logical_t RegisterLibrary(CSTRING a1, RexxPackageEntry *a2) { return threadContext->RegisterLibrary(a1, a2); }
    RexxClassObject FindClass(CSTRING a1) { return threadContext->FindClass(a1); }
    RexxClassObject FindPackageClass(RexxPackageObject a1, CSTRING a2) { return threadContext->FindPackageClass(a1, a2); }
    RexxDirectoryObject GetPackageRoutines(RexxPackageObject a1) { return threadContext->GetPackageRoutines(a1); }
    RexxDirectoryObject GetPackagePublicRoutines(RexxPackageObject a1) { return threadContext->GetPackagePublicRoutines(a1); }
    RexxDirectoryObject GetPackageClasses(RexxPackageObject a1) { return threadContext->GetPackageClasses(a1); }
    RexxDirectoryObject GetPackagePublicClasses(RexxPackageObject a1) { return threadContext->GetPackagePublicClasses(a1); }
    RexxDirectoryObject GetPackageMethods(RexxPackageObject a1) { return threadContext->GetPackageMethods(a1); }
    RexxObjectPtr CallRoutine(RexxRoutineObject a1, RexxArrayObject a2) { return threadContext->CallRoutine(a1, a2); }
    RexxObjectPtr CallProgram(CSTRING a1, RexxArrayObject a2) { return threadContext->CallProgram(a1, a2); }
    RexxMethodObject NewMethod(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewMethod(a1, a2, a3); }
    RexxRoutineObject NewRoutine(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewRoutine(a1, a2, a3); }
    logical_t IsRoutine(RexxObjectPtr a1) { return threadContext->IsRoutine(a1); }
    logical_t IsMethod(RexxObjectPtr a1) { return threadContext->IsMethod(a1); }
    RexxPackageObject GetRoutinePackage(RexxRoutineObject a1) { return threadContext->GetRoutinePackage(a1); }
    RexxPackageObject GetMethodPackage(RexxMethodObject a1) { return threadContext->GetMethodPackage(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1) { return threadContext->ObjectToCSelf(a1); }
    RexxObjectPtr WholeNumberToObject(wholenumber_t a1) { return threadContext->WholeNumberToObject(a1); }
    RexxObjectPtr UintptrToObject(uintptr_t a1) { return threadContext->UintptrToObject(a1); }
    RexxObjectPtr IntptrToObject(intptr_t a1) { return threadContext->IntptrToObject(a1); }
    RexxObjectPtr ValueToObject(ValueDescriptor *a1) { return threadContext->ValueToObject(a1); }
    RexxArrayObject ValuesToObject(ValueDescriptor *a1, size_t a2) { return threadContext->ValuesToObject(a1, a2); }
    logical_t ObjectToValue(RexxObjectPtr a1, ValueDescriptor *a2) { return threadContext->ObjectToValue(a1, a2); }
    RexxObjectPtr StringSizeToObject(size_t a1) { return threadContext->StringSizeToObject(a1); }
    logical_t ObjectToWholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->ObjectToWholeNumber(a1, a2); }
    logical_t ObjectToStringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->ObjectToStringSize(a1, a2); }
    RexxObjectPtr Int64ToObject(int64_t a1) { return threadContext->Int64ToObject(a1); }
    RexxObjectPtr UnsignedInt64ToObject(uint64_t a1) { return threadContext->UnsignedInt64ToObject(a1); }
    logical_t ObjectToInt64(RexxObjectPtr a1, int64_t *a2) { return threadContext->ObjectToInt64(a1, a2); }
    logical_t ObjectToUnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->ObjectToUnsignedInt64(a1, a2); }
    RexxObjectPtr Int32ToObject(int32_t a1) { return threadContext->Int32ToObject(a1); }
    RexxObjectPtr UnsignedInt32ToObject(uint32_t a1) { return threadContext->UnsignedInt32ToObject(a1); }
    logical_t ObjectToInt32(RexxObjectPtr a1, int32_t *a2) { return threadContext->ObjectToInt32(a1, a2); }
    logical_t ObjectToUnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->ObjectToUnsignedInt32(a1, a2); }
    logical_t ObjectToUintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->ObjectToUintptr(a1, a2); }
    logical_t ObjectToIntptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->ObjectToIntptr(a1, a2); }
    logical_t ObjectToLogical(RexxObjectPtr a1, logical_t *a2) { return threadContext->ObjectToLogical(a1, a2); }
    RexxObjectPtr LogicalToObject(logical_t a1) { return threadContext->LogicalToObject(a1); }
    RexxObjectPtr DoubleToObject(double a1) { return threadContext->DoubleToObject(a1); }
    RexxObjectPtr DoubleToObjectWithPrecision(double a1, size_t a2) { return threadContext->DoubleToObjectWithPrecision(a1, a2); }
    logical_t ObjectToDouble(RexxObjectPtr a1, double *a2) { return threadContext->ObjectToDouble(a1, a2); }
    RexxStringObject ObjectToString(RexxObjectPtr a1) { return threadContext->ObjectToString(a1); }
    CSTRING ObjectToStringValue(RexxObjectPtr a1) { return threadContext->ObjectToStringValue(a1); }
    size_t StringGet(RexxStringObject a1, size_t a2, POINTER a3, size_t a4) { return threadContext->StringGet(a1, a2, a3, a4); }
    size_t StringLength(RexxStringObject a1) { return threadContext->StringLength(a1); }
    CSTRING StringData(RexxStringObject a1) { return threadContext->StringData(a1); }
    RexxStringObject NewString(CSTRING a1, size_t a2) { return threadContext->NewString(a1, a2); }
    RexxStringObject NewStringFromAsciiz(CSTRING a1) { return threadContext->NewStringFromAsciiz(a1); }
    RexxStringObject StringUpper(RexxStringObject a1) { return threadContext->StringUpper(a1); }
    RexxStringObject StringLower(RexxStringObject a1) { return threadContext->StringLower(a1); }
    logical_t IsString(RexxObjectPtr a1) { return threadContext->IsString(a1); }
    RexxBufferStringObject NewBufferString(size_t a1) { return threadContext->NewBufferString(a1); }
    size_t BufferStringLength(RexxBufferStringObject a1) { return threadContext->BufferStringLength(a1); }
    POINTER BufferStringData(RexxBufferStringObject a1) { return threadContext->BufferStringData(a1); }
    RexxStringObject FinishBufferString(RexxBufferStringObject a1, size_t a2) { return threadContext->FinishBufferString(a1, a2); }
    void DirectoryPut(RexxDirectoryObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->DirectoryPut(a1, a2, a3); }
    RexxObjectPtr DirectoryAt(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryAt(a1, a2); }
    RexxObjectPtr DirectoryRemove(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryRemove(a1, a2); }
    RexxDirectoryObject NewDirectory() { return threadContext->NewDirectory(); }
    logical_t IsDirectory(RexxObjectPtr a1) { return threadContext->IsDirectory(a1); }
    RexxObjectPtr ArrayAt(RexxArrayObject a1, size_t a2) { return threadContext->ArrayAt(a1, a2); }
    void ArrayPut(RexxArrayObject a1, RexxObjectPtr a2, size_t a3) { threadContext->ArrayPut(a1, a2, a3); }
    size_t ArrayAppend(RexxArrayObject a1, RexxObjectPtr a2) { return threadContext->ArrayAppend(a1, a2); }
    size_t ArrayAppendString(RexxArrayObject a1, CSTRING a2, size_t a3) { return threadContext->ArrayAppendString(a1, a2, a3); }
    size_t ArraySize(RexxArrayObject a1) { return threadContext->ArraySize(a1); }
    size_t ArrayItems(RexxArrayObject a1) { return threadContext->ArrayItems(a1); }
    size_t ArrayDimension(RexxArrayObject a1) { return threadContext->ArrayDimension(a1); }
    RexxArrayObject NewArray(size_t a1) { return threadContext->NewArray(a1); }
    RexxArrayObject ArrayOfOne(RexxObjectPtr a1) { return threadContext->ArrayOfOne(a1); }
    RexxArrayObject ArrayOfTwo(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ArrayOfTwo(a1, a2); }
    RexxArrayObject ArrayOfThree(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->ArrayOfThree(a1, a2, a3); }
    RexxArrayObject ArrayOfFour(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->ArrayOfFour(a1, a2, a3, a4); }
    logical_t IsArray(RexxObjectPtr a1) { return threadContext->IsArray(a1); }
    POINTER BufferData(RexxBufferObject a1) { return threadContext->BufferData(a1); }
    size_t BufferLength(RexxBufferObject a1) { return threadContext->BufferLength(a1); }
    RexxBufferObject NewBuffer(wholenumber_t a1) { return threadContext->NewBuffer(a1); }
    logical_t IsBuffer(RexxObjectPtr a1) { return threadContext->IsBuffer(a1); }
    POINTER PointerValue(RexxPointerObject a1) { return threadContext->PointerValue(a1); }
    RexxPointerObject NewPointer(POINTER a1) { return threadContext->NewPointer(a1); }
    logical_t IsPointer(RexxObjectPtr a1) { return threadContext->IsPointer(a1); }
    RexxObjectPtr SupplierItem(RexxSupplierObject a1) { return threadContext->SupplierItem(a1); }
    RexxObjectPtr SupplierIndex(RexxSupplierObject a1) { return threadContext->SupplierIndex(a1); }
    logical_t SupplierAvailable(RexxSupplierObject a1) { return threadContext->SupplierAvailable(a1); }
    void SupplierNext(RexxSupplierObject a1) { threadContext->SupplierNext(a1); }
    RexxSupplierObject NewSupplier(RexxArrayObject a1, RexxArrayObject a2) { return threadContext->NewSupplier(a1, a2); }
    RexxStemObject NewStem(CSTRING a1) { return threadContext->NewStem(a1); }
    void SetStemElement(RexxStemObject a1, CSTRING a2, RexxObjectPtr a3) { threadContext->SetStemElement(a1, a2, a3); }
    RexxObjectPtr GetStemElement(RexxStemObject a1, CSTRING a2) { return threadContext->GetStemElement(a1, a2); }
    void DropStemElement(RexxStemObject a1, CSTRING a2) { threadContext->DropStemElement(a1, a2); }
    void SetStemArrayElement(RexxStemObject a1, size_t a2, RexxObjectPtr a3) { threadContext->SetStemArrayElement(a1, a2, a3); }
    RexxObjectPtr GetStemArrayElement(RexxStemObject a1, size_t a2) { return threadContext->GetStemArrayElement(a1, a2); }
    void DropStemArrayElement(RexxStemObject a1, size_t a2) { threadContext->DropStemArrayElement(a1, a2); }
    RexxDirectoryObject GetAllStemElements(RexxStemObject a1) { return threadContext->GetAllStemElements(a1); }
    RexxObjectPtr GetStemValue(RexxStemObject a1) { return threadContext->GetStemValue(a1); }
    logical_t IsStem(RexxObjectPtr a1) { return threadContext->IsStem(a1); }
    void RaiseException0(size_t a1) { threadContext->RaiseException0(a1); }
    void RaiseException1(size_t a1, RexxObjectPtr a2) { threadContext->RaiseException1(a1, a2); }
    void RaiseException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { threadContext->RaiseException2(a1, a2, a3); }
    void RaiseException(size_t a1, RexxArrayObject a2) { threadContext->RaiseException(a1, a2); }
    void RaiseCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { threadContext->RaiseCondition(a1, a2, a3, a4); }
    logical_t CheckCondition() { return threadContext->CheckCondition(); }
    RexxDirectoryObject GetConditionInfo() { return threadContext->GetConditionInfo(); }
    void DecodeConditionInfo(RexxDirectoryObject a1, RexxCondition *a2) { threadContext->DecodeConditionInfo(a1, a2); }
    void ClearCondition() { threadContext->ClearCondition(); }
    logical_t DisplayCondition() { return threadContext->DisplayCondition(); }
    POINTER MutableBufferData(RexxMutableBufferObject a1) { return threadContext->MutableBufferData(a1); }
    size_t MutableBufferLength(RexxMutableBufferObject a1) { return threadContext->MutableBufferLength(a1); }
    size_t SetMutableBufferLength(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferLength(a1, a2); }
    RexxMutableBufferObject NewMutableBuffer(wholenumber_t a1) { return threadContext->NewMutableBuffer(a1); }
    logical_t IsMutableBuffer(RexxObjectPtr a1) { return threadContext->IsMutableBuffer(a1); }
    size_t MutableBufferCapacity(RexxMutableBufferObject a1) { return threadContext->MutableBufferCapacity(a1); }
    POINTER SetMutableBufferCapacity(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferCapacity(a1, a2); }
    RexxStringObject VariableReferenceName(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceName(a1); }
    RexxObjectPtr VariableReferenceValue(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceValue(a1); }
    void SetVariableReferenceValue(RexxVariableReferenceObject a1, RexxObjectPtr a2) { threadContext->SetVariableReferenceValue(a1, a2); }
    logical_t IsVariableReference(RexxObjectPtr a1) { return threadContext->IsVariableReference(a1); }
    void StringTablePut(RexxStringTableObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->StringTablePut(a1, a2, a3); }
    RexxObjectPtr StringTableAt(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableAt(a1, a2); }
    RexxObjectPtr StringTableRemove(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableRemove(a1, a2); }
    RexxStringTableObject NewStringTable() { return threadContext->NewStringTable(); }
    logical_t IsStringTable(RexxObjectPtr a1) { return threadContext->IsStringTable(a1); }
    RexxObjectPtr SendMessageScoped(RexxObjectPtr a1, CSTRING a2, RexxClassObject a3, RexxArrayObject a4) { return threadContext->SendMessageScoped(a1, a2, a3, a4); }
    RexxInstance *GetInterpreterInstance() { return threadContext->GetInterpreterInstance(); }
    void RaiseConditionObject(RexxDirectoryObject a1) { threadContext->RaiseConditionObject(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ObjectToCSelf(a1, a2); }
    RexxObjectPtr WholeNumber(wholenumber_t a1) { return threadContext->WholeNumber(a1); }
    RexxObjectPtr Uintptr(uintptr_t a1) { return threadContext->Uintptr(a1); }
    RexxObjectPtr Intptr(intptr_t a1) { return threadContext->Intptr(a1); }
    RexxObjectPtr StringSize(size_t a1) { return threadContext->StringSize(a1); }
    logical_t WholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->WholeNumber(a1, a2); }
    logical_t StringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->StringSize(a1, a2); }
    RexxObjectPtr Int64(int64_t a1) { return threadContext->Int64(a1); }
    RexxObjectPtr UnsignedInt64(uint64_t a1) { return threadContext->UnsignedInt64(a1); }
    logical_t Int64(RexxObjectPtr a1, int64_t *a2) { return threadContext->Int64(a1, a2); }
    logical_t UnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->UnsignedInt64(a1, a2); }
    RexxObjectPtr Int32(int32_t a1) { return threadContext->Int32(a1); }
    RexxObjectPtr UnsignedInt32(uint32_t a1) { return threadContext->UnsignedInt32(a1); }
    logical_t Int32(RexxObjectPtr a1, int32_t *a2) { return threadContext->Int32(a1, a2); }
    logical_t UnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->UnsignedInt32(a1, a2); }
    logical_t Uintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->Uintptr(a1, a2); }
    logical_t Intptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->Intptr(a1, a2); }
    logical_t Logical(RexxObjectPtr a1, logical_t *a2) { return threadContext->Logical(a1, a2); }
    RexxObjectPtr Logical(logical_t a1) { return threadContext->Logical(a1); }
    RexxObjectPtr Double(double a1) { return threadContext->Double(a1); }
    logical_t Double(RexxObjectPtr a1, double *a2) { return threadContext->Double(a1, a2); }
    CSTRING CString(RexxObjectPtr a1) { return threadContext->CString(a1); }
    RexxStringObject String(CSTRING a1, size_t a2) { return threadContext->String(a1, a2); }
    RexxStringObject String(CSTRING a1) { return threadContext->String(a1); }
    RexxStringObject CString(CSTRING a1) { return threadContext->CString(a1); }
    RexxArrayObject Array(RexxObjectPtr a1) { return threadContext->Array(a1); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->Array(a1, a2); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->Array(a1, a2, a3); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->Array(a1, a2, a3, a4); }
    RexxObjectPtr Nil() { return threadContext->Nil(); }
    RexxObjectPtr True() { return threadContext->True(); }
    RexxObjectPtr False() { return threadContext->False(); }
    RexxStringObject NullString() { return threadContext->NullString(); }
    size_t InterpreterVersion() { return threadContext->InterpreterVersion(); }
    size_t LanguageLevel() { return threadContext->LanguageLevel(); }
    void AddCommandEnvironment(const char *a1, REXXPFN a2, int a3) { threadContext->AddCommandEnvironment(a1, a2, a3); }
    logical_t StackCommandEnvironment(const char *a1, REXXPFN a2, int a3) { return threadContext->StackCommandEnvironment(a1, a2, a3); }
    POINTER GetApplicationData() { return threadContext->GetApplicationData(); }
#endif
};

struct RexxExitContext_ {
    RexxThreadContext *threadContext;
    ExitContextInterface *functions;
    ValueDescriptor *arguments;
#ifdef __cplusplus
    void SetContextVariable(CSTRING a1, RexxObjectPtr a2) { functions->SetContextVariable(this, a1, a2); }
    RexxObjectPtr GetContextVariable(CSTRING a1) { return functions->GetContextVariable(this, a1); }
    void DropContextVariable(CSTRING a1) { functions->DropContextVariable(this, a1); }
    RexxDirectoryObject GetAllContextVariables() { return functions->GetAllContextVariables(this); }
    RexxObjectPtr GetCallerContext() { return functions->GetCallerContext(this); }
    RexxVariableReferenceObject GetContextVariableReference(CSTRING a1) { return functions->GetContextVariableReference(this, a1); }
    void ThrowException0(size_t a1) { functions->ThrowException0(this, a1); }
    void ThrowException1(size_t a1, RexxObjectPtr a2) { functions->ThrowException1(this, a1, a2); }
    void ThrowException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { functions->ThrowException2(this, a1, a2, a3); }
    void ThrowException(size_t a1, RexxArrayObject a2) { functions->ThrowException(this, a1, a2); }
    void ThrowCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { functions->ThrowCondition(this, a1, a2, a3, a4); }
    RexxObjectPtr RequestGlobalReference(RexxObjectPtr a1) { return threadContext->RequestGlobalReference(a1); }
    void ReleaseGlobalReference(RexxObjectPtr a1) { threadContext->ReleaseGlobalReference(a1); }
    void ReleaseLocalReference(RexxObjectPtr a1) { threadContext->ReleaseLocalReference(a1); }
    RexxObjectPtr SendMessage(RexxObjectPtr a1, CSTRING a2, RexxArrayObject a3) { return threadContext->SendMessage(a1, a2, a3); }
    RexxObjectPtr SendMessage0(RexxObjectPtr a1, CSTRING a2) { return threadContext->SendMessage0(a1, a2); }
    RexxObjectPtr SendMessage1(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3) { return threadContext->SendMessage1(a1, a2, a3); }
    RexxObjectPtr SendMessage2(RexxObjectPtr a1, CSTRING a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->SendMessage2(a1, a2, a3, a4); }
    RexxDirectoryObject GetLocalEnvironment() { return threadContext->GetLocalEnvironment(); }
    RexxDirectoryObject GetGlobalEnvironment() { return threadContext->GetGlobalEnvironment(); }
    logical_t IsInstanceOf(RexxObjectPtr a1, RexxClassObject a2) { return threadContext->IsInstanceOf(a1, a2); }
    logical_t IsOfType(RexxObjectPtr a1, CSTRING a2) { return threadContext->IsOfType(a1, a2); }
    logical_t HasMethod(RexxObjectPtr a1, CSTRING a2) { return threadContext->HasMethod(a1, a2); }
    RexxPackageObject LoadPackage(CSTRING a1) { return threadContext->LoadPackage(a1); }
    RexxPackageObject LoadPackageFromData(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->LoadPackageFromData(a1, a2, a3); }
    logical_t LoadLibrary(CSTRING a1) { return threadContext->LoadLibrary(a1); }
    logical_t RegisterLibrary(CSTRING a1, RexxPackageEntry *a2) { return threadContext->RegisterLibrary(a1, a2); }
    RexxClassObject FindClass(CSTRING a1) { return threadContext->FindClass(a1); }
    RexxClassObject FindPackageClass(RexxPackageObject a1, CSTRING a2) { return threadContext->FindPackageClass(a1, a2); }
    RexxDirectoryObject GetPackageRoutines(RexxPackageObject a1) { return threadContext->GetPackageRoutines(a1); }
    RexxDirectoryObject GetPackagePublicRoutines(RexxPackageObject a1) { return threadContext->GetPackagePublicRoutines(a1); }
    RexxDirectoryObject GetPackageClasses(RexxPackageObject a1) { return threadContext->GetPackageClasses(a1); }
    RexxDirectoryObject GetPackagePublicClasses(RexxPackageObject a1) { return threadContext->GetPackagePublicClasses(a1); }
    RexxDirectoryObject GetPackageMethods(RexxPackageObject a1) { return threadContext->GetPackageMethods(a1); }
    RexxObjectPtr CallRoutine(RexxRoutineObject a1, RexxArrayObject a2) { return threadContext->CallRoutine(a1, a2); }
    RexxObjectPtr CallProgram(CSTRING a1, RexxArrayObject a2) { return threadContext->CallProgram(a1, a2); }
    RexxMethodObject NewMethod(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewMethod(a1, a2, a3); }
    RexxRoutineObject NewRoutine(CSTRING a1, CSTRING a2, size_t a3) { return threadContext->NewRoutine(a1, a2, a3); }
    logical_t IsRoutine(RexxObjectPtr a1) { return threadContext->IsRoutine(a1); }
    logical_t IsMethod(RexxObjectPtr a1) { return threadContext->IsMethod(a1); }
    RexxPackageObject GetRoutinePackage(RexxRoutineObject a1) { return threadContext->GetRoutinePackage(a1); }
    RexxPackageObject GetMethodPackage(RexxMethodObject a1) { return threadContext->GetMethodPackage(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1) { return threadContext->ObjectToCSelf(a1); }
    RexxObjectPtr WholeNumberToObject(wholenumber_t a1) { return threadContext->WholeNumberToObject(a1); }
    RexxObjectPtr UintptrToObject(uintptr_t a1) { return threadContext->UintptrToObject(a1); }
    RexxObjectPtr IntptrToObject(intptr_t a1) { return threadContext->IntptrToObject(a1); }
    RexxObjectPtr ValueToObject(ValueDescriptor *a1) { return threadContext->ValueToObject(a1); }
    RexxArrayObject ValuesToObject(ValueDescriptor *a1, size_t a2) { return threadContext->ValuesToObject(a1, a2); }
    logical_t ObjectToValue(RexxObjectPtr a1, ValueDescriptor *a2) { return threadContext->ObjectToValue(a1, a2); }
    RexxObjectPtr StringSizeToObject(size_t a1) { return threadContext->StringSizeToObject(a1); }
    logical_t ObjectToWholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->ObjectToWholeNumber(a1, a2); }
    logical_t ObjectToStringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->ObjectToStringSize(a1, a2); }
    RexxObjectPtr Int64ToObject(int64_t a1) { return threadContext->Int64ToObject(a1); }
    RexxObjectPtr UnsignedInt64ToObject(uint64_t a1) { return threadContext->UnsignedInt64ToObject(a1); }
    logical_t ObjectToInt64(RexxObjectPtr a1, int64_t *a2) { return threadContext->ObjectToInt64(a1, a2); }
    logical_t ObjectToUnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->ObjectToUnsignedInt64(a1, a2); }
    RexxObjectPtr Int32ToObject(int32_t a1) { return threadContext->Int32ToObject(a1); }
    RexxObjectPtr UnsignedInt32ToObject(uint32_t a1) { return threadContext->UnsignedInt32ToObject(a1); }
    logical_t ObjectToInt32(RexxObjectPtr a1, int32_t *a2) { return threadContext->ObjectToInt32(a1, a2); }
    logical_t ObjectToUnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->ObjectToUnsignedInt32(a1, a2); }
    logical_t ObjectToUintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->ObjectToUintptr(a1, a2); }
    logical_t ObjectToIntptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->ObjectToIntptr(a1, a2); }
    logical_t ObjectToLogical(RexxObjectPtr a1, logical_t *a2) { return threadContext->ObjectToLogical(a1, a2); }
    RexxObjectPtr LogicalToObject(logical_t a1) { return threadContext->LogicalToObject(a1); }
    RexxObjectPtr DoubleToObject(double a1) { return threadContext->DoubleToObject(a1); }
    RexxObjectPtr DoubleToObjectWithPrecision(double a1, size_t a2) { return threadContext->DoubleToObjectWithPrecision(a1, a2); }
    logical_t ObjectToDouble(RexxObjectPtr a1, double *a2) { return threadContext->ObjectToDouble(a1, a2); }
    RexxStringObject ObjectToString(RexxObjectPtr a1) { return threadContext->ObjectToString(a1); }
    CSTRING ObjectToStringValue(RexxObjectPtr a1) { return threadContext->ObjectToStringValue(a1); }
    size_t StringGet(RexxStringObject a1, size_t a2, POINTER a3, size_t a4) { return threadContext->StringGet(a1, a2, a3, a4); }
    size_t StringLength(RexxStringObject a1) { return threadContext->StringLength(a1); }
    CSTRING StringData(RexxStringObject a1) { return threadContext->StringData(a1); }
    RexxStringObject NewString(CSTRING a1, size_t a2) { return threadContext->NewString(a1, a2); }
    RexxStringObject NewStringFromAsciiz(CSTRING a1) { return threadContext->NewStringFromAsciiz(a1); }
    RexxStringObject StringUpper(RexxStringObject a1) { return threadContext->StringUpper(a1); }
    RexxStringObject StringLower(RexxStringObject a1) { return threadContext->StringLower(a1); }
    logical_t IsString(RexxObjectPtr a1) { return threadContext->IsString(a1); }
    RexxBufferStringObject NewBufferString(size_t a1) { return threadContext->NewBufferString(a1); }
    size_t BufferStringLength(RexxBufferStringObject a1) { return threadContext->BufferStringLength(a1); }
    POINTER BufferStringData(RexxBufferStringObject a1) { return threadContext->BufferStringData(a1); }
    RexxStringObject FinishBufferString(RexxBufferStringObject a1, size_t a2) { return threadContext->FinishBufferString(a1, a2); }
    void DirectoryPut(RexxDirectoryObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->DirectoryPut(a1, a2, a3); }
    RexxObjectPtr DirectoryAt(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryAt(a1, a2); }
    RexxObjectPtr DirectoryRemove(RexxDirectoryObject a1, CSTRING a2) { return threadContext->DirectoryRemove(a1, a2); }
    RexxDirectoryObject NewDirectory() { return threadContext->NewDirectory(); }
    logical_t IsDirectory(RexxObjectPtr a1) { return threadContext->IsDirectory(a1); }
    RexxObjectPtr ArrayAt(RexxArrayObject a1, size_t a2) { return threadContext->ArrayAt(a1, a2); }
    void ArrayPut(RexxArrayObject a1, RexxObjectPtr a2, size_t a3) { threadContext->ArrayPut(a1, a2, a3); }
    size_t ArrayAppend(RexxArrayObject a1, RexxObjectPtr a2) { return threadContext->ArrayAppend(a1, a2); }
    size_t ArrayAppendString(RexxArrayObject a1, CSTRING a2, size_t a3) { return threadContext->ArrayAppendString(a1, a2, a3); }
    size_t ArraySize(RexxArrayObject a1) { return threadContext->ArraySize(a1); }
    size_t ArrayItems(RexxArrayObject a1) { return threadContext->ArrayItems(a1); }
    size_t ArrayDimension(RexxArrayObject a1) { return threadContext->ArrayDimension(a1); }
    RexxArrayObject NewArray(size_t a1) { return threadContext->NewArray(a1); }
    RexxArrayObject ArrayOfOne(RexxObjectPtr a1) { return threadContext->ArrayOfOne(a1); }
    RexxArrayObject ArrayOfTwo(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ArrayOfTwo(a1, a2); }
    RexxArrayObject ArrayOfThree(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->ArrayOfThree(a1, a2, a3); }
    RexxArrayObject ArrayOfFour(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->ArrayOfFour(a1, a2, a3, a4); }
    logical_t IsArray(RexxObjectPtr a1) { return threadContext->IsArray(a1); }
    POINTER BufferData(RexxBufferObject a1) { return threadContext->BufferData(a1); }
    size_t BufferLength(RexxBufferObject a1) { return threadContext->BufferLength(a1); }
    RexxBufferObject NewBuffer(wholenumber_t a1) { return threadContext->NewBuffer(a1); }
    logical_t IsBuffer(RexxObjectPtr a1) { return threadContext->IsBuffer(a1); }
    POINTER PointerValue(RexxPointerObject a1) { return threadContext->PointerValue(a1); }
    RexxPointerObject NewPointer(POINTER a1) { return threadContext->NewPointer(a1); }
    logical_t IsPointer(RexxObjectPtr a1) { return threadContext->IsPointer(a1); }
    RexxObjectPtr SupplierItem(RexxSupplierObject a1) { return threadContext->SupplierItem(a1); }
    RexxObjectPtr SupplierIndex(RexxSupplierObject a1) { return threadContext->SupplierIndex(a1); }
    logical_t SupplierAvailable(RexxSupplierObject a1) { return threadContext->SupplierAvailable(a1); }
    void SupplierNext(RexxSupplierObject a1) { threadContext->SupplierNext(a1); }
    RexxSupplierObject NewSupplier(RexxArrayObject a1, RexxArrayObject a2) { return threadContext->NewSupplier(a1, a2); }
    RexxStemObject NewStem(CSTRING a1) { return threadContext->NewStem(a1); }
    void SetStemElement(RexxStemObject a1, CSTRING a2, RexxObjectPtr a3) { threadContext->SetStemElement(a1, a2, a3); }
    RexxObjectPtr GetStemElement(RexxStemObject a1, CSTRING a2) { return threadContext->GetStemElement(a1, a2); }
    void DropStemElement(RexxStemObject a1, CSTRING a2) { threadContext->DropStemElement(a1, a2); }
    void SetStemArrayElement(RexxStemObject a1, size_t a2, RexxObjectPtr a3) { threadContext->SetStemArrayElement(a1, a2, a3); }
    RexxObjectPtr GetStemArrayElement(RexxStemObject a1, size_t a2) { return threadContext->GetStemArrayElement(a1, a2); }
    void DropStemArrayElement(RexxStemObject a1, size_t a2) { threadContext->DropStemArrayElement(a1, a2); }
    RexxDirectoryObject GetAllStemElements(RexxStemObject a1) { return threadContext->GetAllStemElements(a1); }
    RexxObjectPtr GetStemValue(RexxStemObject a1) { return threadContext->GetStemValue(a1); }
    logical_t IsStem(RexxObjectPtr a1) { return threadContext->IsStem(a1); }
    void RaiseException0(size_t a1) { threadContext->RaiseException0(a1); }
    void RaiseException1(size_t a1, RexxObjectPtr a2) { threadContext->RaiseException1(a1, a2); }
    void RaiseException2(size_t a1, RexxObjectPtr a2, RexxObjectPtr a3) { threadContext->RaiseException2(a1, a2, a3); }
    void RaiseException(size_t a1, RexxArrayObject a2) { threadContext->RaiseException(a1, a2); }
    void RaiseCondition(CSTRING a1, RexxStringObject a2, RexxObjectPtr a3, RexxObjectPtr a4) { threadContext->RaiseCondition(a1, a2, a3, a4); }
    logical_t CheckCondition() { return threadContext->CheckCondition(); }
    RexxDirectoryObject GetConditionInfo() { return threadContext->GetConditionInfo(); }
    void DecodeConditionInfo(RexxDirectoryObject a1, RexxCondition *a2) { threadContext->DecodeConditionInfo(a1, a2); }
    void ClearCondition() { threadContext->ClearCondition(); }
    logical_t DisplayCondition() { return threadContext->DisplayCondition(); }
    POINTER MutableBufferData(RexxMutableBufferObject a1) { return threadContext->MutableBufferData(a1); }
    size_t MutableBufferLength(RexxMutableBufferObject a1) { return threadContext->MutableBufferLength(a1); }
    size_t SetMutableBufferLength(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferLength(a1, a2); }
    RexxMutableBufferObject NewMutableBuffer(wholenumber_t a1) { return threadContext->NewMutableBuffer(a1); }
    logical_t IsMutableBuffer(RexxObjectPtr a1) { return threadContext->IsMutableBuffer(a1); }
    size_t MutableBufferCapacity(RexxMutableBufferObject a1) { return threadContext->MutableBufferCapacity(a1); }
    POINTER SetMutableBufferCapacity(RexxMutableBufferObject a1, size_t a2) { return threadContext->SetMutableBufferCapacity(a1, a2); }
    RexxStringObject VariableReferenceName(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceName(a1); }
    RexxObjectPtr VariableReferenceValue(RexxVariableReferenceObject a1) { return threadContext->VariableReferenceValue(a1); }
    void SetVariableReferenceValue(RexxVariableReferenceObject a1, RexxObjectPtr a2) { threadContext->SetVariableReferenceValue(a1, a2); }
    logical_t IsVariableReference(RexxObjectPtr a1) { return threadContext->IsVariableReference(a1); }
    void StringTablePut(RexxStringTableObject a1, RexxObjectPtr a2, CSTRING a3) { threadContext->StringTablePut(a1, a2, a3); }
    RexxObjectPtr StringTableAt(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableAt(a1, a2); }
    RexxObjectPtr StringTableRemove(RexxStringTableObject a1, CSTRING a2) { return threadContext->StringTableRemove(a1, a2); }
    RexxStringTableObject NewStringTable() { return threadContext->NewStringTable(); }
    logical_t IsStringTable(RexxObjectPtr a1) { return threadContext->IsStringTable(a1); }
    RexxObjectPtr SendMessageScoped(RexxObjectPtr a1, CSTRING a2, RexxClassObject a3, RexxArrayObject a4) { return threadContext->SendMessageScoped(a1, a2, a3, a4); }
    RexxInstance *GetInterpreterInstance() { return threadContext->GetInterpreterInstance(); }
    void RaiseConditionObject(RexxDirectoryObject a1) { threadContext->RaiseConditionObject(a1); }
    POINTER ObjectToCSelf(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->ObjectToCSelf(a1, a2); }
    RexxObjectPtr WholeNumber(wholenumber_t a1) { return threadContext->WholeNumber(a1); }
    RexxObjectPtr Uintptr(uintptr_t a1) { return threadContext->Uintptr(a1); }
    RexxObjectPtr Intptr(intptr_t a1) { return threadContext->Intptr(a1); }
    RexxObjectPtr StringSize(size_t a1) { return threadContext->StringSize(a1); }
    logical_t WholeNumber(RexxObjectPtr a1, wholenumber_t *a2) { return threadContext->WholeNumber(a1, a2); }
    logical_t StringSize(RexxObjectPtr a1, size_t *a2) { return threadContext->StringSize(a1, a2); }
    RexxObjectPtr Int64(int64_t a1) { return threadContext->Int64(a1); }
    RexxObjectPtr UnsignedInt64(uint64_t a1) { return threadContext->UnsignedInt64(a1); }
    logical_t Int64(RexxObjectPtr a1, int64_t *a2) { return threadContext->Int64(a1, a2); }
    logical_t UnsignedInt64(RexxObjectPtr a1, uint64_t *a2) { return threadContext->UnsignedInt64(a1, a2); }
    RexxObjectPtr Int32(int32_t a1) { return threadContext->Int32(a1); }
    RexxObjectPtr UnsignedInt32(uint32_t a1) { return threadContext->UnsignedInt32(a1); }
    logical_t Int32(RexxObjectPtr a1, int32_t *a2) { return threadContext->Int32(a1, a2); }
    logical_t UnsignedInt32(RexxObjectPtr a1, uint32_t *a2) { return threadContext->UnsignedInt32(a1, a2); }
    logical_t Uintptr(RexxObjectPtr a1, uintptr_t *a2) { return threadContext->Uintptr(a1, a2); }
    logical_t Intptr(RexxObjectPtr a1, intptr_t *a2) { return threadContext->Intptr(a1, a2); }
    logical_t Logical(RexxObjectPtr a1, logical_t *a2) { return threadContext->Logical(a1, a2); }
    RexxObjectPtr Logical(logical_t a1) { return threadContext->Logical(a1); }
    RexxObjectPtr Double(double a1) { return threadContext->Double(a1); }
    logical_t Double(RexxObjectPtr a1, double *a2) { return threadContext->Double(a1, a2); }
    CSTRING CString(RexxObjectPtr a1) { return threadContext->CString(a1); }
    RexxStringObject String(CSTRING a1, size_t a2) { return threadContext->String(a1, a2); }
    RexxStringObject String(CSTRING a1) { return threadContext->String(a1); }
    RexxStringObject CString(CSTRING a1) { return threadContext->CString(a1); }
    RexxArrayObject Array(RexxObjectPtr a1) { return threadContext->Array(a1); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2) { return threadContext->Array(a1, a2); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3) { return threadContext->Array(a1, a2, a3); }
    RexxArrayObject Array(RexxObjectPtr a1, RexxObjectPtr a2, RexxObjectPtr a3, RexxObjectPtr a4) { return threadContext->Array(a1, a2, a3, a4); }
    RexxObjectPtr Nil() { return threadContext->Nil(); }
    RexxObjectPtr True() { return threadContext->True(); }
    RexxObjectPtr False() { return threadContext->False(); }
    RexxStringObject NullString() { return threadContext->NullString(); }
    size_t InterpreterVersion() { return threadContext->InterpreterVersion(); }
    size_t LanguageLevel() { return threadContext->LanguageLevel(); }
    void AddCommandEnvironment(const char *a1, REXXPFN a2, int a3) { threadContext->AddCommandEnvironment(a1, a2, a3); }
    logical_t StackCommandEnvironment(const char *a1, REXXPFN a2, int a3) { return threadContext->StackCommandEnvironment(a1, a2, a3); }
    POINTER GetApplicationData() { return threadContext->GetApplicationData(); }
#endif
};

struct RexxIORedirectorContext_ {
    IORedirectorInterface *functions;
#ifdef __cplusplus
    void ReadInput(CSTRING *a1, size_t *a2) { functions->ReadInput(this, a1, a2); }
    void ReadInputBuffer(CSTRING *a1, size_t *a2) { functions->ReadInputBuffer(this, a1, a2); }
    void WriteOutput(CSTRING a1, size_t a2) { functions->WriteOutput(this, a1, a2); }
    void WriteError(CSTRING a1, size_t a2) { functions->WriteError(this, a1, a2); }
    void WriteOutputBuffer(CSTRING a1, size_t a2) { functions->WriteOutputBuffer(this, a1, a2); }
    void WriteErrorBuffer(CSTRING a1, size_t a2) { functions->WriteErrorBuffer(this, a1, a2); }
    logical_t IsInputRedirected() { return functions->IsInputRedirected(this); }
    logical_t IsOutputRedirected() { return functions->IsOutputRedirected(this); }
    logical_t IsErrorRedirected() { return functions->IsErrorRedirected(this); }
    logical_t AreOutputAndErrorSameTarget() { return functions->AreOutputAndErrorSameTarget(this); }
    logical_t IsRedirectionRequested() { return functions->IsRedirectionRequested(this); }
#endif
};
/* END GENERATED context-structs */

#ifdef __cplusplus
}
#endif

#endif
