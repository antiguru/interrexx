/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The types of the Rexx programming interface whose definition is this
 * platform's.
 *
 * The section between the BEGIN GENERATED and END GENERATED lines is written
 * from the per-platform table of rust/crates/interrexx-api/src/header.rs:
 * change that, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1.
 */

#ifndef INTERREXX_REXXAPITYPES_H
#define INTERREXX_REXXAPITYPES_H

/* BEGIN GENERATED platform-types */
#include <stdlib.h>
#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <unistd.h>
#include <sys/types.h>
#include <pthread.h>

typedef void *REXXPFN;
typedef pid_t process_id_t;
typedef pthread_t thread_id_t;

typedef struct _REXXDATETIME REXXDATETIME;

struct _REXXDATETIME {
    uint16_t hours;
    uint16_t minutes;
    uint16_t seconds;
    uint16_t hundredths;
    uint16_t day;
    uint16_t month;
    uint16_t year;
    uint16_t weekday;
    size_t microseconds;
    size_t yearday;
    uint16_t valid;
};
/* END GENERATED platform-types */

#endif
