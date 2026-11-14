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
};

struct RexxThreadContext_ {
    RexxInstance *instance;
    RexxThreadInterface *functions;
};

struct RexxMethodContext_ {
    RexxThreadContext *threadContext;
    MethodContextInterface *functions;
    ValueDescriptor *arguments;
};

struct RexxCallContext_ {
    RexxThreadContext *threadContext;
    CallContextInterface *functions;
    ValueDescriptor *arguments;
};

struct RexxExitContext_ {
    RexxThreadContext *threadContext;
    ExitContextInterface *functions;
    ValueDescriptor *arguments;
};

struct RexxIORedirectorContext_ {
    IORedirectorInterface *functions;
};
/* END GENERATED context-structs */

#ifdef __cplusplus
}
#endif

#endif
