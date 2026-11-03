// RexxStart of the program the first argument names while another thread sends the process a
// SIGINT half a second in: what RexxStart answers, and whether the handler stays installed.
#include <stdio.h>
#include <string.h>
#include <signal.h>
#include <unistd.h>
#include <pthread.h>
#include <rexx.h>
static void *interrupter(void *) {
    usleep(500000);
    kill(getpid(), SIGINT);
    return NULL;
}
int main(int, char **argv) {
    setvbuf(stdout, NULL, _IONBF, 0);
    printf("before\n");
    pthread_t t;
    pthread_create(&t, NULL, interrupter, NULL);
    short s = -99; RXSTRING r = {0, NULL};
    int rc = RexxStart(0, NULL, argv[1], NULL, NULL, RXCOMMAND, NULL, &s, &r);
    printf("after rc=%d short=%d result=%s\n", rc, s, r.strptr ? r.strptr : "(null)");
    pthread_join(t, NULL);
    struct sigaction sa; sigaction(SIGINT, NULL, &sa);
    printf("handler afterwards: %s\n", sa.sa_handler == SIG_DFL ? "default" : sa.sa_handler == SIG_IGN ? "ignored" : "installed");
    return 0;
}
