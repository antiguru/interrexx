/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The classic Rexx programming interface: the scalar and object handle types,
 * the string, exit and shared-variable structures, the queue time stamp, the
 * system exit parameter blocks, the handler types, and the functions the
 * interpreter and the API libraries export.
 *
 * Each section between a BEGIN GENERATED and an END GENERATED line is written
 * from interrexx's Rust definitions by rust/crates/interrexx-api/src/header.rs:
 * change those, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1. The rest of this file is written by hand.
 */

#ifndef INTERREXX_REXX_H
#define INTERREXX_REXX_H

#include "rexxapitypes.h"
#include "rexxplatformdefs.h"
#include "rexxapidefs.h"

#ifdef __cplusplus
extern "C" {
#endif

/* The calling convention of a function the interpreter calls. */
#define RexxEntry REXXENTRY

/* BEGIN GENERATED typedefs */
typedef ssize_t wholenumber_t;
typedef size_t stringsize_t;
typedef size_t logical_t;
typedef const char *CSTRING;
typedef void *POINTER;
typedef int RexxReturnCode;
/* END GENERATED typedefs */

/* The parameter block an exit handler receives; its layout is the one the
 * exit's function code names. */
typedef char *PEXIT;

/* BEGIN GENERATED handles */
#ifdef __cplusplus
}
struct _RexxObjectPtr {};
struct _RexxStringObject : public _RexxObjectPtr {};
struct _RexxBufferStringObject : public _RexxStringObject {};
struct _RexxArrayObject : public _RexxObjectPtr {};
struct _RexxBufferObject : public _RexxObjectPtr {};
struct _RexxPointerObject : public _RexxObjectPtr {};
struct _RexxMethodObject : public _RexxObjectPtr {};
struct _RexxRoutineObject : public _RexxObjectPtr {};
struct _RexxPackageObject : public _RexxObjectPtr {};
struct _RexxClassObject : public _RexxObjectPtr {};
struct _RexxDirectoryObject : public _RexxObjectPtr {};
struct _RexxStringTableObject : public _RexxObjectPtr {};
struct _RexxSupplierObject : public _RexxObjectPtr {};
struct _RexxStemObject : public _RexxObjectPtr {};
struct _RexxMutableBufferObject : public _RexxObjectPtr {};
struct _RexxVariableReferenceObject : public _RexxObjectPtr {};
typedef _RexxObjectPtr *RexxObjectPtr;
typedef _RexxStringObject *RexxStringObject;
typedef _RexxBufferStringObject *RexxBufferStringObject;
typedef _RexxArrayObject *RexxArrayObject;
typedef _RexxBufferObject *RexxBufferObject;
typedef _RexxPointerObject *RexxPointerObject;
typedef _RexxMethodObject *RexxMethodObject;
typedef _RexxRoutineObject *RexxRoutineObject;
typedef _RexxPackageObject *RexxPackageObject;
typedef _RexxClassObject *RexxClassObject;
typedef _RexxDirectoryObject *RexxDirectoryObject;
typedef _RexxStringTableObject *RexxStringTableObject;
typedef _RexxSupplierObject *RexxSupplierObject;
typedef _RexxStemObject *RexxStemObject;
typedef _RexxMutableBufferObject *RexxMutableBufferObject;
typedef _RexxVariableReferenceObject *RexxVariableReferenceObject;
extern "C" {
#else
typedef struct _RexxObjectPtr *RexxObjectPtr;
typedef struct _RexxStringObject *RexxStringObject;
typedef struct _RexxBufferStringObject *RexxBufferStringObject;
typedef struct _RexxArrayObject *RexxArrayObject;
typedef struct _RexxBufferObject *RexxBufferObject;
typedef struct _RexxPointerObject *RexxPointerObject;
typedef struct _RexxMethodObject *RexxMethodObject;
typedef struct _RexxRoutineObject *RexxRoutineObject;
typedef struct _RexxPackageObject *RexxPackageObject;
typedef struct _RexxClassObject *RexxClassObject;
typedef struct _RexxDirectoryObject *RexxDirectoryObject;
typedef struct _RexxStringTableObject *RexxStringTableObject;
typedef struct _RexxSupplierObject *RexxSupplierObject;
typedef struct _RexxStemObject *RexxStemObject;
typedef struct _RexxMutableBufferObject *RexxMutableBufferObject;
typedef struct _RexxVariableReferenceObject *RexxVariableReferenceObject;
#endif
/* END GENERATED handles */

/* BEGIN GENERATED structs */
typedef struct _RXSTRING RXSTRING;
typedef struct _CONSTRXSTRING CONSTRXSTRING;
typedef struct _RXSYSEXIT RXSYSEXIT;
typedef struct _SHVBLOCK SHVBLOCK;
typedef struct _RexxQueueTime RexxQueueTime;
typedef struct _RexxConditionData RexxConditionData;

struct _RXSTRING {
    size_t strlength;
    char *strptr;
};

struct _CONSTRXSTRING {
    size_t strlength;
    const char *strptr;
};

struct _RXSYSEXIT {
    const char *sysexit_name;
    int sysexit_code;
};

struct _SHVBLOCK {
    SHVBLOCK *shvnext;
    CONSTRXSTRING shvname;
    RXSTRING shvvalue;
    size_t shvnamelen;
    size_t shvvaluelen;
    unsigned char shvcode;
    unsigned char shvret;
};

struct _RexxQueueTime {
    uint16_t hours;
    uint16_t minutes;
    uint16_t seconds;
    uint16_t hundredths;
    uint16_t day;
    uint16_t month;
    uint16_t year;
    uint16_t weekday;
    uint32_t microseconds;
    uint32_t yearday;
};

struct _RexxConditionData {
    wholenumber_t code;
    wholenumber_t rc;
    RXSTRING message;
    RXSTRING errortext;
    size_t position;
    RXSTRING program;
};

typedef struct _RXFNC_FLAGS {
    unsigned rxfferr : 1;
    unsigned rxffnfnd : 1;
    unsigned rxffsub : 1;
} RXFNC_FLAGS;
typedef struct _RXOFNC_FLAGS {
    unsigned rxfferr : 1;
    unsigned rxffnfnd : 1;
    unsigned rxffsub : 1;
} RXOFNC_FLAGS;
typedef struct _RXEXF_FLAGS {
    unsigned rxfferr : 1;
    unsigned rxffnfnd : 1;
    unsigned rxffsub : 1;
} RXEXF_FLAGS;
typedef struct _RXCMD_FLAGS {
    unsigned rxfcfail : 1;
    unsigned rxfcerr : 1;
} RXCMD_FLAGS;
typedef struct _RXMSQ_FLAGS {
    unsigned rxfmlifo : 1;
} RXMSQ_FLAGS;
typedef struct _RXHLT_FLAGS {
    unsigned rxfhhalt : 1;
} RXHLT_FLAGS;
typedef struct _RXTRC_FLAGS {
    unsigned rxftrace : 1;
} RXTRC_FLAGS;

typedef struct _RXFNCCAL_PARM RXFNCCAL_PARM;
typedef struct _RXOFNCCAL_PARM RXOFNCCAL_PARM;
typedef struct _RXEXFCAL_PARM RXEXFCAL_PARM;
typedef struct _RXCMDHST_PARM RXCMDHST_PARM;
typedef struct _RXMSQPSH_PARM RXMSQPSH_PARM;
typedef struct _RXSIOSAY_PARM RXSIOSAY_PARM;
typedef struct _RXSIOTRC_PARM RXSIOTRC_PARM;
typedef struct _RXVARNOVALUE_PARM RXVARNOVALUE_PARM;
typedef struct _RXVALCALL_PARM RXVALCALL_PARM;
typedef struct _RXMSQPLL_PARM RXMSQPLL_PARM;
typedef struct _RXMSQSIZ_PARM RXMSQSIZ_PARM;
typedef struct _RXMSQNAM_PARM RXMSQNAM_PARM;
typedef struct _RXSIOTRD_PARM RXSIOTRD_PARM;
typedef struct _RXSIODTR_PARM RXSIODTR_PARM;
typedef struct _RXHLTTST_PARM RXHLTTST_PARM;
typedef struct _RXTRCTST_PARM RXTRCTST_PARM;

struct _RXFNCCAL_PARM {
    RXFNC_FLAGS rxfnc_flags;
    const char *rxfnc_name;
    unsigned short rxfnc_namel;
    const char *rxfnc_que;
    unsigned short rxfnc_quel;
    unsigned short rxfnc_argc;
    CONSTRXSTRING *rxfnc_argv;
    RXSTRING rxfnc_retc;
};

struct _RXOFNCCAL_PARM {
    RXOFNC_FLAGS rxfnc_flags;
    CONSTRXSTRING rxfnc_name;
    size_t rxfnc_argc;
    RexxObjectPtr *rxfnc_argv;
    RexxObjectPtr rxfnc_retc;
};

struct _RXEXFCAL_PARM {
    RXEXF_FLAGS rxfnc_flags;
    CONSTRXSTRING rxfnc_name;
    size_t rxfnc_argc;
    RexxObjectPtr *rxfnc_argv;
    RexxObjectPtr rxfnc_retc;
};

struct _RXCMDHST_PARM {
    RXCMD_FLAGS rxcmd_flags;
    const char *rxcmd_address;
    unsigned short rxcmd_addressl;
    const char *rxcmd_dll;
    unsigned short rxcmd_dll_len;
    CONSTRXSTRING rxcmd_command;
    RXSTRING rxcmd_retc;
};

struct _RXMSQPSH_PARM {
    RXMSQ_FLAGS rxmsq_flags;
    CONSTRXSTRING rxmsq_value;
};

struct _RXSIOSAY_PARM {
    CONSTRXSTRING rxsio_string;
};

struct _RXSIOTRC_PARM {
    CONSTRXSTRING rxsio_string;
};

struct _RXVARNOVALUE_PARM {
    RexxStringObject variable_name;
    RexxObjectPtr value;
};

struct _RXVALCALL_PARM {
    RexxStringObject selector;
    RexxStringObject variable_name;
    RexxObjectPtr value;
};

struct _RXMSQPLL_PARM {
    RXSTRING rxmsq_retc;
};

struct _RXMSQSIZ_PARM {
    size_t rxmsq_size;
};

struct _RXMSQNAM_PARM {
    RXSTRING rxmsq_name;
};

struct _RXSIOTRD_PARM {
    RXSTRING rxsiotrd_retc;
};

struct _RXSIODTR_PARM {
    RXSTRING rxsiodtr_retc;
};

struct _RXHLTTST_PARM {
    RXHLT_FLAGS rxhlt_flags;
};

struct _RXTRCTST_PARM {
    RXTRC_FLAGS rxtrc_flags;
};
/* END GENERATED structs */

/* BEGIN GENERATED pointers */
typedef RXSTRING *PRXSTRING;
typedef CONSTRXSTRING *PCONSTRXSTRING;
typedef RXSYSEXIT *PRXSYSEXIT;
typedef SHVBLOCK *PSHVBLOCK;
/* END GENERATED pointers */

/* In C++, BEGIN_EXTERN_C() opens a block of declarations with C linkage and
 * END_EXTERN_C() closes it; in C both expand to nothing. */
#ifdef __cplusplus
#define BEGIN_EXTERN_C() extern "C" {
#define END_EXTERN_C() }
#else
#define BEGIN_EXTERN_C()
#define END_EXTERN_C()
#endif

/* The null object reference, the null string pointer, and two older names of
 * the constant string types. */
#define NULLOBJECT NULL
#define NO_CSTRING NULL
#define CONSTANT_STRING CSTRING
#define CONSTANT_RXSTRING CONSTRXSTRING

/* MAKERXSTRING(r, p, l) points the RXSTRING r at the l characters at p.
 * RXSTRPTR(r) is its pointer and RXSTRLEN(r) its length, 0 when it is
 * empty. An RXSTRING is empty (RXNULLSTRING) when its pointer is null, the
 * null string (RXZEROLENSTRING) when its pointer is not null and its length
 * is 0, and valid (RXVALIDSTRING) when it holds at least one character. */
#define MAKERXSTRING(r, p, l) ((r).strptr = (p), (r).strlength = (l))
#define RXSTRPTR(r) ((r).strptr)
#define RXNULLSTRING(r) ((r).strptr == NULL)
#define RXZEROLENSTRING(r) ((r).strptr != NULL && (r).strlength == 0)
#define RXVALIDSTRING(r) ((r).strptr != NULL && (r).strlength != 0)
#define RXSTRLEN(r) (RXNULLSTRING(r) ? (size_t)0 : (size_t)(r).strlength)

/* BEGIN GENERATED handlers */
typedef RexxReturnCode REXXENTRY RexxSubcomHandler(CONSTRXSTRING *, unsigned short *, RXSTRING *);
typedef size_t REXXENTRY RexxRoutineHandler(CSTRING, size_t, CONSTRXSTRING *, CSTRING, RXSTRING *);
typedef size_t REXXENTRY RexxFunctionHandler(CSTRING, size_t, CONSTRXSTRING *, CSTRING, RXSTRING *);
typedef int REXXENTRY RexxExitHandler(int, int, PEXIT);
/* END GENERATED handlers */

/* BEGIN GENERATED constants */
#define MAX_QUEUE_NAME_LENGTH 250
#define RX_CB_OK 0
#define RX_CB_BADP 1
#define RX_CB_NOTSTARTED 2
#define RX_CB_TOOMANYP 3
#define RX_CB_BADN 8
/* END GENERATED constants */

/* BEGIN GENERATED exports */
int REXXENTRY RexxStart(size_t, CONSTRXSTRING *, CSTRING, RXSTRING *, CSTRING, int, RXSYSEXIT *, short *, RXSTRING *);
void REXXENTRY RexxWaitForTermination(void);
RexxReturnCode REXXENTRY RexxDidRexxTerminate(void);
RexxReturnCode REXXENTRY RexxShutDownAPI(void);
RexxReturnCode REXXENTRY RexxTranslateProgram(CSTRING, CSTRING, RXSYSEXIT *);
RexxReturnCode REXXENTRY RexxTranslateInstoreProgram(CSTRING, CONSTRXSTRING *, RXSTRING *);
char *REXXENTRY RexxGetVersionInformation(void);
RexxReturnCode REXXENTRY RexxRegisterSubcomDll(CSTRING, CSTRING, CSTRING, CSTRING, size_t);
RexxReturnCode REXXENTRY RexxRegisterSubcomExe(CSTRING, REXXPFN, CSTRING);
RexxReturnCode REXXENTRY RexxQuerySubcom(CSTRING, CSTRING, unsigned short *, char *);
RexxReturnCode REXXENTRY RexxDeregisterSubcom(CSTRING, CSTRING);
RexxReturnCode REXXENTRY RexxVariablePool(SHVBLOCK *);
RexxReturnCode REXXENTRY RexxCallBack(CSTRING, size_t, CONSTRXSTRING *, short *, RXSTRING *);
RexxReturnCode REXXENTRY RexxRegisterFunctionDll(CSTRING, CSTRING, CSTRING);
RexxReturnCode REXXENTRY RexxRegisterFunctionExe(CSTRING, REXXPFN);
RexxReturnCode REXXENTRY RexxDeregisterFunction(CSTRING);
RexxReturnCode REXXENTRY RexxQueryFunction(CSTRING);
RexxReturnCode REXXENTRY RexxRegisterExitDll(CSTRING, CSTRING, CSTRING, CSTRING, size_t);
RexxReturnCode REXXENTRY RexxRegisterExitExe(CSTRING, REXXPFN, CSTRING);
RexxReturnCode REXXENTRY RexxDeregisterExit(CSTRING, CSTRING);
RexxReturnCode REXXENTRY RexxQueryExit(CSTRING, CSTRING, unsigned short *, char *);
RexxReturnCode REXXENTRY RexxSetHalt(process_id_t, thread_id_t);
RexxReturnCode REXXENTRY RexxSetTrace(process_id_t, thread_id_t);
RexxReturnCode REXXENTRY RexxResetTrace(process_id_t, thread_id_t);
RexxReturnCode REXXENTRY RexxAddMacro(CSTRING, CSTRING, size_t);
RexxReturnCode REXXENTRY RexxDropMacro(CSTRING);
RexxReturnCode REXXENTRY RexxSaveMacroSpace(size_t, CSTRING *, CSTRING);
RexxReturnCode REXXENTRY RexxLoadMacroSpace(size_t, CSTRING *, CSTRING);
RexxReturnCode REXXENTRY RexxQueryMacro(CSTRING, unsigned short *);
RexxReturnCode REXXENTRY RexxReorderMacro(CSTRING, size_t);
RexxReturnCode REXXENTRY RexxClearMacroSpace(void);
RexxReturnCode REXXENTRY RexxCreateQueue(char *, size_t, CSTRING, size_t *);
RexxReturnCode REXXENTRY RexxOpenQueue(CSTRING, size_t *);
RexxReturnCode REXXENTRY RexxQueueExists(CSTRING);
RexxReturnCode REXXENTRY RexxDeleteQueue(CSTRING);
RexxReturnCode REXXENTRY RexxQueryQueue(CSTRING, size_t *);
RexxReturnCode REXXENTRY RexxAddQueue(CSTRING, CONSTRXSTRING *, size_t);
RexxReturnCode REXXENTRY RexxPullFromQueue(CSTRING, RXSTRING *, RexxQueueTime *, size_t);
RexxReturnCode REXXENTRY RexxClearQueue(CSTRING);
void *REXXENTRY RexxAllocateMemory(size_t);
RexxReturnCode REXXENTRY RexxFreeMemory(void *);

typedef int (REXXENTRY *PFNREXXSTART)(size_t, CONSTRXSTRING *, CSTRING, RXSTRING *, CSTRING, int, RXSYSEXIT *, short *, RXSTRING *);
typedef void (REXXENTRY *PFNREXXWAITFORTERMINATION)(void);
typedef RexxReturnCode (REXXENTRY *PFNREXXDIDREXXTERMINATE)(void);
typedef RexxReturnCode (REXXENTRY *PFNREXXSHUTDOWNAPI)(void);
typedef RexxReturnCode (REXXENTRY *PFNREXXTRANSLATEPROGRAM)(CSTRING, CSTRING, RXSYSEXIT *);
typedef RexxReturnCode (REXXENTRY *PFNREXXTRANSLATEINSTOREPROGRAM)(CSTRING, CONSTRXSTRING *, RXSTRING *);
typedef char *(REXXENTRY *PFNGETVERSIONINFORMATION)(void);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTERSUBCOMDLL)(CSTRING, CSTRING, CSTRING, char *, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTERSUBCOMEXE)(CSTRING, REXXPFN, char *);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUERYSUBCOM)(CSTRING, CSTRING, unsigned short *, char *);
typedef RexxReturnCode (REXXENTRY *PFNREXXDEREGISTERSUBCOM)(CSTRING, CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXVARIABLEPOOL)(SHVBLOCK *);
typedef RexxReturnCode (REXXENTRY *PFNREXXCALLBACK)(CSTRING, size_t, CONSTRXSTRING *, short *, RXSTRING *);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTERFUNCTIONDLL)(CSTRING, CSTRING, CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTERFUNCTIONEXE)(CSTRING, REXXPFN);
typedef RexxReturnCode (REXXENTRY *PFNREXXDEREGISTERFUNCTION)(CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUERYFUNCTION)(CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTEREXITDLL)(CSTRING, CSTRING, CSTRING, char *, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXREGISTEREXITEXE)(CSTRING, REXXPFN, char *);
typedef RexxReturnCode (REXXENTRY *PFNREXXDEREGISTEREXIT)(CSTRING, CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUERYEXIT)(CSTRING, CSTRING, unsigned short *, char *);
typedef RexxReturnCode (REXXENTRY *PFNREXXSETHALT)(process_id_t, thread_id_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXSETTRACE)(process_id_t, thread_id_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXRESETTRACE)(process_id_t, thread_id_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXADDMACRO)(CSTRING, CSTRING, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXDROPMACRO)(CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXSAVEMACROSPACE)(size_t, CSTRING *, CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXLOADMACROSPACE)(size_t, CSTRING *, CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUERYMACRO)(CSTRING, unsigned short *);
typedef RexxReturnCode (REXXENTRY *PFNREXXREORDERMACRO)(CSTRING, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXCLEARMACROSPACE)(void);
typedef RexxReturnCode (REXXENTRY *PFNREXXCREATEQUEUE)(char *, size_t, CSTRING, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXOPENQUEUE)(CSTRING, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUEUEEXISTS)(CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXDELETEQUEUE)(CSTRING);
typedef RexxReturnCode (REXXENTRY *PFNREXXQUERYQUEUE)(CSTRING, size_t *);
typedef RexxReturnCode (REXXENTRY *PFNREXXADDQUEUE)(CSTRING, CONSTRXSTRING *, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXPULLFROMQUEUE)(CSTRING, RXSTRING *, RexxQueueTime *, size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXCLEARQUEUE)(CSTRING);
typedef void *(REXXENTRY *PFNREXXALLOCATEMEMORY)(size_t);
typedef RexxReturnCode (REXXENTRY *PFNREXXFREEMEMORY)(void *);

#define REXXSTART RexxStart
#define REXXWAITFORTERMINATION RexxWaitForTermination
#define REXXDIDREXXTERMINATE RexxDidRexxTerminate
#define REXXSHUTDOWNAPI RexxShutDownAPI
#define REXXTRANSLATEPROGRAM RexxTranslateProgram
#define REXXTRANSLATEINSTOREPROGRAM RexxTranslateInstoreProgram
#define REXXGETVERSIONINFORMATON RexxGetVersionInformation
#define REXXREGISTERSUBCOMDLL RexxRegisterSubcomDll
#define REXXREGISTERSUBCOMEXE RexxRegisterSubcomExe
#define REXXQUERYSUBCOM RexxQuerySubcom
#define REXXDEREGISTERSUBCOM RexxDeregisterSubcom
#define REXXVARIABLEPOOL RexxVariablePool
#define REXXCALLBACK RexxCallBack
#define REXXREGISTERFUNCTIONDLL RexxRegisterFunctionDll
#define REXXREGISTERFUNCTIONEXE RexxRegisterFunctionExe
#define REXXDEREGISTERFUNCTION RexxDeregisterFunction
#define REXXQUERYFUNCTION RexxQueryFunction
#define REXXREGISTEREXITDLL RexxRegisterExitDll
#define REXXREGISTEREXITEXE RexxRegisterExitExe
#define REXXDEREGISTEREXIT RexxDeregisterExit
#define REXXQUERYEXIT RexxQueryExit
#define REXXSETHALT RexxSetHalt
#define REXXSETTRACE RexxSetTrace
#define REXXRESETTRACE RexxResetTrace
#define REXXADDMACRO RexxAddMacro
#define REXXDROPMACRO RexxDropMacro
#define REXXSAVEMACROSPACE RexxSaveMacroSpace
#define REXXLOADMACROSPACE RexxLoadMacroSpace
#define REXXQUERYMACRO RexxQueryMacro
#define REXXREORDERMACRO RexxReorderMacro
#define REXXCLEARMACROSPACE RexxClearMacroSpace
/* END GENERATED exports */

#ifdef __cplusplus
}
#endif

/* The exports whose signatures name a type of this platform's. */
#include "rexxplatformapis.h"

#endif
