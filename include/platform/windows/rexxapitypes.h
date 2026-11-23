/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The types of the Rexx programming interface whose definition is this
 * platform's.
 *
 * The section between the BEGIN GENERATED and END GENERATED lines is written
 * from the per-platform table of rust/crates/interrexx-api/src/header.rs:
 * change that, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1. The rest of this file is written by hand.
 */

#ifndef INTERREXX_REXXAPITYPES_H
#define INTERREXX_REXXAPITYPES_H

/* BEGIN GENERATED platform-types */
#include <stddef.h>
#include <stdint.h>
#include <limits.h>
#include <windows.h>

typedef SSIZE_T ssize_t;
typedef DWORD process_id_t;
typedef DWORD thread_id_t;
/* END GENERATED platform-types */

/* Windows APIs use pointer-size counts and the native calling convention. */
#if defined(_WIN64)
#define __REXX64__
#endif
#ifndef SSIZE_MAX
#define SSIZE_MAX ((ssize_t)(SIZE_MAX >> 1))
#endif
#ifndef SSIZE_MIN
#define SSIZE_MIN (-SSIZE_MAX - 1)
#endif

typedef size_t (APIENTRY *REXXPFN)();

#endif
