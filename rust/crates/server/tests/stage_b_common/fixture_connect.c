#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <unistd.h>

static int (*system_connect)(int, const struct sockaddr *, socklen_t);

/* Resolve the libc function before concurrent HTTP connections start. */
__attribute__((constructor)) static void initialize(void) {
    system_connect = dlsym(RTLD_NEXT, "connect");
    if (!system_connect) _exit(86);
}

/* Reject every network destination except the assigned fixture before the operating system can connect. */
int connect(int fd, const struct sockaddr *address, socklen_t length) {
    const char *port = getenv("MYMIND_FIXTURE_CONNECT_PORT");
    if (!port) _exit(86);
    if (address->sa_family == AF_INET) {
        const struct sockaddr_in *ipv4 = (const struct sockaddr_in *)address;
        if (length < sizeof(*ipv4) || ntohl(ipv4->sin_addr.s_addr) != INADDR_LOOPBACK ||
            ntohs(ipv4->sin_port) != strtoul(port, NULL, 10)) _exit(86);
    } else if (address->sa_family == AF_INET6) {
        _exit(86);
    }
    return system_connect(fd, address, length);
}
