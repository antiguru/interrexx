// The classic RexxStart family beside RexxStart's own run: the error messages and the version,
// the translations, RexxStart's in-store images, its `//T` and its result and short, and
// RexxSetHalt and RexxSetTrace naming the thread a program runs on. Its standard output is the
// oracle's, line for line.
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <pthread.h>
#include <rexx.h>

extern "C" const char *RexxGetErrorMessage(int);
extern "C" const char *RexxGetErrorMessageByNumber(int);
extern "C" RexxReturnCode RexxCompileProgram(const char *, const char *, PRXSYSEXIT, bool);

static void show(const char *what, int rc, short rexxrc, RXSTRING *r)
{
    printf("%s: rc=%d short=%d result=%s len=%zu\n", what, rc, rexxrc,
        r->strptr ? r->strptr : "(null)", r->strlength);
}

static int start(const char *what, const char *name, PRXSTRING instore, const char *arg)
{
    CONSTRXSTRING a;
    short s = -99;
    RXSTRING r;
    char buf[16] = "preset";
    r.strptr = buf;
    r.strlength = sizeof buf;
    if (arg) { a.strptr = arg; a.strlength = strlen(arg); }
    int rc = RexxStart(arg ? 1 : 0, arg ? &a : NULL, name, instore, NULL, RXCOMMAND, NULL, &s, &r);
    show(what, rc, s, &r);
    return rc;
}

static const char *runner_program;
static void *runner(void *)
{
    short s = 0;
    RXSTRING r = {0, NULL};
    int rc = RexxStart(0, NULL, runner_program, NULL, NULL, RXCOMMAND, NULL, &s, &r);
    printf("runner %s: rc=%d short=%d result=%s\n", runner_program, rc, s, r.strptr ? r.strptr : "(null)");
    RexxFreeMemory(r.strptr);
    return NULL;
}

static size_t SelfHalt(const char *, size_t, PCONSTRXSTRING, const char *, PRXSTRING ret)
{
    int rc = RexxSetHalt(getpid(), (thread_id_t)pthread_self());
    sprintf(ret->strptr, "%d", rc);
    ret->strlength = strlen(ret->strptr);
    return 0;
}

static size_t SelfTrace(const char *, size_t argc, PCONSTRXSTRING, const char *, PRXSTRING ret)
{
    int rc = argc ? RexxResetTrace(getpid(), (thread_id_t)pthread_self())
                  : RexxSetTrace(getpid(), (thread_id_t)pthread_self());
    sprintf(ret->strptr, "%d", rc);
    ret->strlength = strlen(ret->strptr);
    return 0;
}

int main()
{
    setvbuf(stdout, NULL, _IONBF, 0);
    char *version = RexxGetVersionInformation();
    printf("version %.33s\n", version);
    RexxFreeMemory(version);
    int codes[] = {3001, 35929, 40001, 0, 3, 99999, 999013};
    for (int code : codes)
    {
        const char *m = RexxGetErrorMessage(code);
        printf("code %d: %s\n", code, m ? m : "(null)");
    }
    int numbers[] = {1, 200, 534, 0, 99999};
    for (int number : numbers)
    {
        const char *m = RexxGetErrorMessageByNumber(number);
        printf("number %d: %s\n", number, m ? m : "(null)");
    }
    printf("terminated %d\n", (int)RexxDidRexxTerminate());
    RexxWaitForTermination();

    start("good", "good.rex", NULL, "x");
    start("leading blank", "good.rex", NULL, " lead");
    start("no result", "nores.rex", NULL, NULL);
    start("missing", "nothere.rex", NULL, NULL);
    start("//T good", "good.rex", NULL, "//T");
    start("//t bad", "bad.rex", NULL, "//t");
    start("bad", "bad.rex", NULL, NULL);

    RXSTRING in[2];
    const char *src = "say 'instore ran' arg(1); return 'is' arg(1)";
    in[0].strptr = (char *)src;
    in[0].strlength = strlen(src);
    in[1].strptr = NULL;
    in[1].strlength = 0;
    start("instore source", "inst", in, "a");
    printf("image filled: %s\n", in[1].strptr && in[1].strlength > 0 ? "yes" : "no");
    in[0].strptr = NULL;
    in[0].strlength = 0;
    start("instore image", "inst", in, "b");
    RexxFreeMemory(in[1].strptr);
    in[0].strptr = NULL;
    in[1].strptr = NULL;
    in[0].strlength = in[1].strlength = 0;
    start("macrospace missing", "nores.rex", in, NULL);
    const char *bsrc = "if then";
    in[0].strptr = (char *)bsrc;
    in[0].strlength = strlen(bsrc);
    in[1].strptr = NULL;
    in[1].strlength = 0;
    start("instore bad", "ibad", in, NULL);

    CONSTRXSTRING s1;
    RXSTRING img = {0, NULL};
    s1.strptr = src;
    s1.strlength = strlen(src);
    int trc = RexxTranslateInstoreProgram("tis", &s1, &img);
    printf("translate instore rc=%d image=%s\n", trc, img.strptr && img.strlength ? "yes" : "no");
    in[0].strptr = NULL;
    in[0].strlength = 0;
    in[1] = img;
    start("translated image", "tis", in, "c");
    RexxFreeMemory(img.strptr);
    CONSTRXSTRING s2;
    RXSTRING img2 = {0, NULL};
    s2.strptr = bsrc;
    s2.strlength = strlen(bsrc);
    trc = RexxTranslateInstoreProgram("tbad", &s2, &img2);
    printf("translate instore bad rc=%d image=%s\n", trc, img2.strptr ? "yes" : "no");
    trc = RexxTranslateProgram("good.rex", "good.img", NULL);
    printf("translate rc=%d\n", trc);
    start("run translated", "good.img", NULL, "t");
    trc = RexxCompileProgram("good.rex", "good.enc", NULL, true);
    printf("compile encoded rc=%d\n", trc);
    start("run encoded", "good.enc", NULL, "e");
    trc = RexxTranslateProgram("bad.rex", "bad.img", NULL);
    printf("translate bad rc=%d exists=%d\n", trc, access("bad.img", F_OK) == 0);
    trc = RexxTranslateProgram("nothere.rex", "x.img", NULL);
    printf("translate missing rc=%d\n", trc);
    trc = RexxTranslateProgram("good.rex", NULL, NULL);
    printf("translate no output rc=%d\n", trc);

    printf("idle halt=%d trace=%d reset=%d\n", RexxSetHalt(getpid(), (thread_id_t)pthread_self()),
        RexxSetTrace(getpid(), (thread_id_t)pthread_self()),
        RexxResetTrace(getpid(), (thread_id_t)pthread_self()));
    pthread_t t;
    runner_program = "loop.rex";
    pthread_create(&t, NULL, runner, NULL);
    usleep(500000);
    printf("halt from another thread=%d\n", RexxSetHalt(getpid(), (thread_id_t)t));
    pthread_join(t, NULL);
    printf("after the run=%d\n", RexxSetHalt(getpid(), (thread_id_t)t));
    RexxRegisterFunctionExe("SELFHALT", (REXXPFN)SelfHalt);
    RexxRegisterFunctionExe("SELFTRACE", (REXXPFN)SelfTrace);
    runner_program = "self.rex";
    runner(NULL);
    runner_program = "trace.rex";
    runner(NULL);
    RexxDeregisterFunction("SELFHALT");
    RexxDeregisterFunction("SELFTRACE");
    return 0;
}
