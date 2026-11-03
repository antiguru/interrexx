// Registers a function in the process through RexxRegisterFunctionExe and calls it from a
// program RexxStart runs. Built linked -lrexx -lrexxapi, -lrexxapi -lrexx, -lrexx alone, and
// -lrexxapi alone with -DFIND_REXXSTART, where RexxStart is looked up in the librexx.so.4 the
// loader mapped as librexxapi.so.4's dependency. Each must find the registration from RexxStart.
#include <stdio.h>
#include <string.h>
#include <rexx.h>
#ifdef FIND_REXXSTART
#include <dlfcn.h>
#endif
static size_t RexxEntry orderf(const char *, size_t, CONSTRXSTRING *, const char *, PRXSTRING r) {
    strcpy(r->strptr, "fromC");
    r->strlength = 5;
    return 0;
}
int main() {
    setvbuf(stdout, NULL, _IONBF, 0);
    int reg = RexxRegisterFunctionExe("ORDERF", (REXXPFN)orderf);
    int query = RexxQueryFunction("ORDERF");
    printf("register %d query %d\n", reg, query);
    const char *source = "return orderf()";
    RXSTRING instore[2];
    instore[0].strptr = (char *)source;
    instore[0].strlength = strlen(source);
    instore[1].strptr = NULL;
    instore[1].strlength = 0;
    char buffer[256];
    RXSTRING result;
    result.strptr = buffer;
    result.strlength = sizeof buffer;
    short rc = -99;
#ifdef FIND_REXXSTART
    void *librexx = dlopen("librexx.so.4", RTLD_NOW | RTLD_NOLOAD);
    if (librexx == NULL) {
        printf("librexx.so.4 not loaded\n");
        return 1;
    }
    typedef int (RexxEntry *Start)(size_t, PCONSTRXSTRING, const char *, PRXSTRING, const char *,
                                   int, PRXSYSEXIT, short *, PRXSTRING);
    Start RexxStart = (Start)dlsym(librexx, "RexxStart");
#endif
    int code = RexxStart(0, NULL, "order", instore, "CMD", RXCOMMAND, NULL, &rc, &result);
    printf("code %d result %.*s\n", code, (int)result.strlength, result.strptr);
    printf("deregister %d\n", (int)RexxDeregisterFunction("ORDERF"));
    return 0;
}
