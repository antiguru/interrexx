/* SPDX-License-Identifier: MIT OR Apache-2.0 */
#include "oorexxapi.h"

int main(void)
{
    RexxInstance *instance = NULL;
    RexxThreadContext *thread = NULL;
    if (RexxCreateInterpreter(&instance, &thread, NULL) != 1 ||
        instance == NULL || thread == NULL)
        return 1;
    instance->Terminate();
    return 0;
}
