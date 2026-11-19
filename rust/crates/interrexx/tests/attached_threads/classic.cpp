// A classic library with no package entry, built as `attachclassic`: one function handler,
// `AtClassicOne`, which answers `1` and starts no thread.
#include "oorexxapi.h"

extern "C" size_t RexxEntry AtClassicOne(const char *, size_t, CONSTRXSTRING *, const char *,
                                         RXSTRING *result)
{
    result->strptr[0] = '1';
    result->strlength = 1;
    return 0;
}
