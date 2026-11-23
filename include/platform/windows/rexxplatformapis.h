/* SPDX-License-Identifier: MIT OR Apache-2.0 */

/*
 * The exported functions of the Rexx programming interface whose signatures
 * name a type of this platform's. rexx.h includes this file once it has
 * declared the types the signatures name.
 *
 * The section between the BEGIN GENERATED and END GENERATED lines is written
 * from interrexx's Rust definitions by rust/crates/interrexx-api/src/header.rs:
 * change those, then run the headers_are_current test of interrexx-api with
 * REXX_HEADERS_REFRESH=1.
 */

#ifndef INTERREXX_REXXPLATFORMAPIS_H
#define INTERREXX_REXXPLATFORMAPIS_H

#ifdef __cplusplus
extern "C" {
#endif

/* BEGIN GENERATED platform-exports */
RexxReturnCode REXXENTRY RexxPullQueue(CSTRING, RXSTRING *, SYSTEMTIME *, size_t);

typedef RexxReturnCode (REXXENTRY *PFNREXXPULLQUEUE)(CSTRING, RXSTRING *, SYSTEMTIME *, size_t);
/* END GENERATED platform-exports */

#ifdef __cplusplus
}
#endif

#endif
