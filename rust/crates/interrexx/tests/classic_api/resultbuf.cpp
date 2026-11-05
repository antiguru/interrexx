// RexxStart of "return 'abc'" with a caller's result buffer of 3 (exactly the result), 4 and 2
// bytes, each inside a larger block filled with 'X': whether the result went into the buffer,
// its length, and the block's bytes after the call.
#include <stdio.h>
#include <string.h>
#include <rexx.h>
static void run(size_t size) {
    char block[8];
    memset(block, 'X', sizeof block);
    RXSTRING instore[2];
    const char *source = "return 'abc'";
    instore[0].strptr = (char *)source;
    instore[0].strlength = strlen(source);
    instore[1].strptr = NULL;
    instore[1].strlength = 0;
    RXSTRING result;
    result.strptr = block;
    result.strlength = size;
    short rc;
    RexxStart(0, NULL, "resultbuf", instore, "CMD", RXCOMMAND, NULL, &rc, &result);
    printf("size %zu: %s length %zu value %.*s block ", size,
           result.strptr == block ? "in buffer" : "new block", result.strlength,
           (int)result.strlength, result.strptr);
    for (size_t at = 0; at < sizeof block; at++) {
        putchar(block[at] == 0 ? '0' : block[at]);
    }
    putchar('\n');
    if (result.strptr != block) {
        RexxFreeMemory(result.strptr);
    }
}
int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    run(3);
    run(4);
    run(2);
    return 0;
}
