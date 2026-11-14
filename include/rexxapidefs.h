/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The codes of the classic Rexx programming interface: how a program is
 * called, the exit function and subfunction codes, the shared-variable
 * request and answer codes, and the flags and return codes of registration,
 * the macrospace and the queues.
 *
 * The section between the BEGIN GENERATED and END GENERATED lines is written
 * from interrexx's Rust definitions by rust/crates/interrexx-api/src/header.rs:
 * change those, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1.
 */

#ifndef INTERREXX_REXXAPIDEFS_H
#define INTERREXX_REXXAPIDEFS_H

/* BEGIN GENERATED constants */
#define RXCOMMAND 0
#define RXSUBROUTINE 1
#define RXFUNCTION 2
#define RXMETHOD 3
#define RXSCRIPT 4
#define RXENDLST 0
#define RXFNC 2
#define RXCMD 3
#define RXMSQ 4
#define RXSIO 5
#define RXHLT 7
#define RXTRC 8
#define RXINI 9
#define RXTER 10
#define RXEXF 12
#define RXNOVAL 13
#define RXVALUE 14
#define RXOFNC 15
#define RXNOOFEXITS 16
#define RXFNCCAL 1
#define RXCMDHST 1
#define RXMSQPLL 1
#define RXMSQPSH 2
#define RXMSQSIZ 3
#define RXMSQNAM 20
#define RXSIOSAY 1
#define RXSIOTRC 2
#define RXSIOTRD 3
#define RXSIODTR 4
#define RXSIOTLL 5
#define RXHLTCLR 1
#define RXHLTTST 2
#define RXTRCTST 1
#define RXINIEXT 1
#define RXTEREXT 1
#define RXEXFCAL 1
#define RXNOVALCALL 1
#define RXVALUECALL 1
#define RXOFNCCAL 1
#define RXEXIT_HANDLED 0
#define RXEXIT_NOT_HANDLED 1
#define RXEXIT_RAISE_ERROR (-1)
#define RXSHV_OK 0x00
#define RXSHV_SET 0x00
#define RXSHV_FETCH 0x01
#define RXSHV_NEWV 0x01
#define RXSHV_DROPV 0x02
#define RXSHV_LVAR 0x02
#define RXSHV_SYSET 0x03
#define RXSHV_SYFET 0x04
#define RXSHV_TRUNC 0x04
#define RXSHV_SYDRO 0x05
#define RXSHV_NEXTV 0x06
#define RXSHV_PRIV 0x07
#define RXSHV_BADN 0x08
#define RXSHV_MEMFL 0x10
#define RXSHV_BADF 0x80
#define RXSHV_NOAVL 144
#define RXSUBCOM_DROPPABLE 0x00
#define RXSUBCOM_OK 0
#define RXSUBCOM_ERROR 0x01
#define RXSUBCOM_ISREG 0x01
#define RXSUBCOM_NONDROP 0x01
#define RXSUBCOM_FAILURE 0x02
#define RXSUBCOM_DUP 10
#define RXSUBCOM_MAXREG 20
#define RXSUBCOM_NOTREG 30
#define RXSUBCOM_NOCANDROP 40
#define RXSUBCOM_LOADERR 50
#define RXSUBCOM_NOPROC 127
#define RXSUBCOM_BADENTRY 1001
#define RXSUBCOM_NOEMEM 1002
#define RXSUBCOM_BADTYPE 1003
#define RXSUBCOM_NOTINIT 1004
#define RXEXIT_DROPPABLE 0x00
#define RXEXIT_OK 0
#define RXEXIT_ERROR 0x01
#define RXEXIT_ISREG 0x01
#define RXEXIT_NONDROP 0x01
#define RXEXIT_FAILURE 0x02
#define RXEXIT_DUP 10
#define RXEXIT_MAXREG 20
#define RXEXIT_NOTREG 30
#define RXEXIT_NOCANDROP 40
#define RXEXIT_LOADERR 50
#define RXEXIT_NOPROC 127
#define RXEXIT_BADENTRY 1001
#define RXEXIT_NOEMEM 1002
#define RXEXIT_BADTYPE 1003
#define RXEXIT_NOTINIT 1004
#define RXFUNC_OK 0
#define RXFUNC_DYNALINK 1
#define RXFUNC_CALLENTRY 2
#define RXFUNC_DEFINED 10
#define RXFUNC_NOMEM 20
#define RXFUNC_NOTREG 30
#define RXFUNC_MODNOTFND 40
#define RXFUNC_ENTNOTFND 50
#define RXFUNC_BADTYPE 70
#define RXFUNC_NOEMEM 1002
#define RXFUNC_NOTINIT 1004
#define RXMACRO_OK 0
#define RXMACRO_NO_STORAGE 1
#define RXMACRO_SEARCH_BEFORE 1
#define RXMACRO_NOT_FOUND 2
#define RXMACRO_SEARCH_AFTER 2
#define RXMACRO_EXTENSION_REQUIRED 3
#define RXMACRO_ALREADY_EXISTS 4
#define RXMACRO_FILE_ERROR 5
#define RXMACRO_SIGNATURE_ERROR 6
#define RXMACRO_SOURCE_NOT_FOUND 7
#define RXMACRO_INVALID_POSITION 8
#define RXMACRO_NOT_INIT 1004
#define RXQUEUE_FIFO 0
#define RXQUEUE_NOWAIT 0
#define RXQUEUE_OK 0
#define RXQUEUE_LIFO 1
#define RXQUEUE_STORAGE 1
#define RXQUEUE_WAIT 1
#define RXQUEUE_SIZE 2
#define RXQUEUE_DUP 3
#define RXQUEUE_BADQNAME 5
#define RXQUEUE_PRIORITY 6
#define RXQUEUE_BADWAITFLAG 7
#define RXQUEUE_EMPTY 8
#define RXQUEUE_NOTREG 9
#define RXQUEUE_ACCESS 10
#define RXQUEUE_MAXREG 11
#define RXQUEUE_MEMFAIL 1002
#define RXQUEUE_NOEMEM 1002
#define RXQUEUE_NOTINIT 1004
#define RXAPI_OK 0
#define RXAPI_MEMFAIL 1002
#define RXAPI_NORXAPI 1004
#define RXARI_OK 0
#define RXARI_NOT_FOUND 1
#define RXARI_PROCESSING_ERROR 2
#define RXAUTOBUFLEN 256
/* END GENERATED constants */

#endif
