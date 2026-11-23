/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include "rexx.h"
#include <string.h>

static PFNREXXPULLQUEUE pull_queue = RexxPullQueue;

static int check_queue(const char *name)
{
    struct {
        unsigned before;
        SYSTEMTIME stamp;
        unsigned after;
    } time = {0x12345678u, {0}, 0x98765432u};
    char buffer[64];
    RXSTRING data = {sizeof buffer, buffer};
    static const char expected[] = "from the interpreter";

    if (pull_queue(name, &data, &time.stamp, RXQUEUE_NOWAIT) != RXQUEUE_OK)
        return 2;
    if (data.strlength != sizeof expected - 1 ||
        memcmp(data.strptr, expected, sizeof expected - 1) != 0)
        return 3;
    if (time.before != 0x12345678u || time.after != 0x98765432u ||
        time.stamp.wYear == 0 || time.stamp.wMonth >= 12 ||
        time.stamp.wDay < 1 || time.stamp.wDay > 31)
        return 4;
    return RexxDeleteQueue(name) != RXQUEUE_OK;
}

int main(int argc, char **argv)
{
    /* Only rexxapi.lib is linked into this program. It finds its sibling DLL. */
    void *block = RexxAllocateMemory(16);
    if (block == NULL || pull_queue == NULL || RexxFreeMemory(block) != 0)
        return 1;
    return argc == 2 ? check_queue(argv[1]) : 0;
}
