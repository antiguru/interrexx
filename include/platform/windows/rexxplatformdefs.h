/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The calling conventions of the Rexx programming interface on this platform.
 *
 * The section between the BEGIN GENERATED and END GENERATED lines is written
 * from the per-platform table of rust/crates/interrexx-api/src/header.rs:
 * change that, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1.
 */

#ifndef INTERREXX_REXXPLATFORMDEFS_H
#define INTERREXX_REXXPLATFORMDEFS_H

/* BEGIN GENERATED platform-defs */
#define REXXENTRY APIENTRY
#define REXXEXPORT __declspec(dllexport)
/* END GENERATED platform-defs */

#ifndef __INT64_C
#define __INT64_C(value) value##LL
#endif

#endif
