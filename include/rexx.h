/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The classic Rexx programming interface: the scalar and object handle types,
 * the string, exit and shared-variable structures, the queue time stamp and
 * the system exit parameter blocks.
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

struct _RXFNCCAL_PARM {
    RXFNC_FLAGS rxfnc_flags;
    const char *rxfnc_name;
    unsigned short rxfnc_namel;
    const char *rxfnc_que;
    unsigned short rxfnc_quel;
    unsigned short rxfnc_argc;
    const CONSTRXSTRING *rxfnc_argv;
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
/* END GENERATED structs */

#ifdef __cplusplus
}
#endif

#endif
