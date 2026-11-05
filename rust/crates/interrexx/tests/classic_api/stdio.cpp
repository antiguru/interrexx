// C stdio output and the output of programs RexxStart runs, with standard output and error
// redirected to files: the lines land in the order the program wrote them.
#include <stdio.h>
#include <string.h>
#include <rexx.h>
static void start(const char *source) {
    RXSTRING instore[2];
    instore[0].strptr = (char *)source;
    instore[0].strlength = strlen(source);
    instore[1].strptr = NULL;
    instore[1].strlength = 0;
    short rc;
    RexxStart(0, NULL, "stdio", instore, "CMD", RXCOMMAND, NULL, &rc, NULL);
}
int main() {
    printf("c one\n");
    fprintf(stderr, "c err one\n");
    start("say 'rexx one'; call lineout 'STDERR', 'rexx err one'");
    printf("c two\n");
    start("say 'rexx two'; call charout , 'rexx three'; say");
    printf("c three\n");
    start("call lineout 'STDERR', 'rexx err two'");
    printf("c four\n");
    return 0;
}
