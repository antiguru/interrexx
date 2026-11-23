/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include "rexx.h"

static PFNREXXPULLQUEUE pull_queue = RexxPullQueue;

int main(void)
{
    /* Only rexxapi.lib is linked into this program. It finds its sibling DLL. */
    void *block = RexxAllocateMemory(16);
    if (block == NULL || pull_queue == NULL)
        return 1;
    return RexxFreeMemory(block) != 0;
}
