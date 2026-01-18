/*
 * Platform Abstraction Layer - Unix/POSIX Implementation
 */

#ifndef _WIN32

#define _GNU_SOURCE
#define _XOPEN_SOURCE 700

#include "platform.h"

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <termios.h>
#include <time.h>
#include <unistd.h>

/* ========== Directory Operations ========== */

struct mdh_dir {
    DIR *dir;
    mdh_dirent_t entry;
};

mdh_dir_t *mdh_opendir(const char *path) {
    DIR *d = opendir(path);
    if (!d) return NULL;

    mdh_dir_t *dir = (mdh_dir_t *)malloc(sizeof(mdh_dir_t));
    if (!dir) {
        closedir(d);
        return NULL;
    }
    dir->dir = d;
    return dir;
}

mdh_dirent_t *mdh_readdir(mdh_dir_t *dir) {
    if (!dir || !dir->dir) return NULL;

    struct dirent *ent = readdir(dir->dir);
    if (!ent) return NULL;

    strncpy(dir->entry.name, ent->d_name, sizeof(dir->entry.name) - 1);
    dir->entry.name[sizeof(dir->entry.name) - 1] = '\0';
    return &dir->entry;
}

int mdh_closedir(mdh_dir_t *dir) {
    if (!dir) return -1;
    int rc = closedir(dir->dir);
    free(dir);
    return rc;
}

/* ========== String Utilities ========== */

int mdh_strcasecmp(const char *s1, const char *s2) {
    return strcasecmp(s1, s2);
}

int mdh_strncasecmp(const char *s1, const char *s2, size_t n) {
    return strncasecmp(s1, s2, n);
}

/* ========== Terminal I/O ========== */

struct mdh_terminal_state {
    struct termios original;
};

int mdh_terminal_get_size(int *width, int *height) {
    struct winsize w;
    if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &w) != 0) {
        return -1;
    }
    if (width) *width = w.ws_col;
    if (height) *height = w.ws_row;
    return 0;
}

int mdh_terminal_raw_mode(mdh_terminal_state_t **state) {
    if (!state) return -1;

    mdh_terminal_state_t *s = (mdh_terminal_state_t *)malloc(sizeof(mdh_terminal_state_t));
    if (!s) return -1;

    if (tcgetattr(STDIN_FILENO, &s->original) != 0) {
        free(s);
        return -1;
    }

    struct termios raw = s->original;
    raw.c_lflag &= ~(ECHO | ICANON);
    raw.c_cc[VMIN] = 1;
    raw.c_cc[VTIME] = 0;

    if (tcsetattr(STDIN_FILENO, TCSANOW, &raw) != 0) {
        free(s);
        return -1;
    }

    *state = s;
    return 0;
}

int mdh_terminal_restore(mdh_terminal_state_t *state) {
    if (!state) return -1;
    int rc = tcsetattr(STDIN_FILENO, TCSANOW, &state->original);
    free(state);
    return rc;
}

int mdh_terminal_read_char(void) {
    unsigned char c;
    if (read(STDIN_FILENO, &c, 1) == 1) {
        return c;
    }
    return -1;
}

int mdh_terminal_input_available(void) {
    struct pollfd pfd = { .fd = STDIN_FILENO, .events = POLLIN };
    return poll(&pfd, 1, 0) > 0 && (pfd.revents & POLLIN);
}

/* ========== Networking ========== */

int mdh_net_init(void) {
    /* No initialization needed on Unix */
    return 0;
}

void mdh_net_cleanup(void) {
    /* No cleanup needed on Unix */
}

int mdh_socket_create(int domain, int type, int protocol) {
    return socket(domain, type, protocol);
}

int mdh_socket_close(int sockfd) {
    return close(sockfd);
}

int mdh_socket_errno(void) {
    return errno;
}

int mdh_poll(mdh_pollfd_t *fds, size_t nfds, int timeout_ms) {
    /* mdh_pollfd_t matches struct pollfd layout on Unix */
    return poll((struct pollfd *)fds, (nfds_t)nfds, timeout_ms);
}

/* ========== Threading ========== */

struct mdh_thread {
    pthread_t thread;
};

struct mdh_mutex {
    pthread_mutex_t mutex;
};

struct mdh_cond {
    pthread_cond_t cond;
};

mdh_thread_t *mdh_thread_create(void *(*func)(void *), void *arg) {
    mdh_thread_t *t = (mdh_thread_t *)malloc(sizeof(mdh_thread_t));
    if (!t) return NULL;

    if (pthread_create(&t->thread, NULL, func, arg) != 0) {
        free(t);
        return NULL;
    }
    return t;
}

int mdh_thread_join(mdh_thread_t *thread, void **retval) {
    if (!thread) return -1;
    int rc = pthread_join(thread->thread, retval);
    free(thread);
    return rc;
}

int mdh_thread_detach(mdh_thread_t *thread) {
    if (!thread) return -1;
    int rc = pthread_detach(thread->thread);
    free(thread);
    return rc;
}

mdh_mutex_t *mdh_mutex_create(void) {
    mdh_mutex_t *m = (mdh_mutex_t *)malloc(sizeof(mdh_mutex_t));
    if (!m) return NULL;

    if (pthread_mutex_init(&m->mutex, NULL) != 0) {
        free(m);
        return NULL;
    }
    return m;
}

void mdh_mutex_destroy(mdh_mutex_t *mutex) {
    if (mutex) {
        pthread_mutex_destroy(&mutex->mutex);
        free(mutex);
    }
}

int mdh_mutex_lock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    return pthread_mutex_lock(&mutex->mutex);
}

int mdh_mutex_unlock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    return pthread_mutex_unlock(&mutex->mutex);
}

int mdh_mutex_trylock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    return pthread_mutex_trylock(&mutex->mutex);
}

mdh_cond_t *mdh_cond_create(void) {
    mdh_cond_t *c = (mdh_cond_t *)malloc(sizeof(mdh_cond_t));
    if (!c) return NULL;

    if (pthread_cond_init(&c->cond, NULL) != 0) {
        free(c);
        return NULL;
    }
    return c;
}

void mdh_cond_destroy(mdh_cond_t *cond) {
    if (cond) {
        pthread_cond_destroy(&cond->cond);
        free(cond);
    }
}

int mdh_cond_wait(mdh_cond_t *cond, mdh_mutex_t *mutex) {
    if (!cond || !mutex) return -1;
    return pthread_cond_wait(&cond->cond, &mutex->mutex);
}

int mdh_cond_timedwait(mdh_cond_t *cond, mdh_mutex_t *mutex, uint64_t timeout_ns) {
    if (!cond || !mutex) return -1;

    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_sec += timeout_ns / 1000000000ULL;
    ts.tv_nsec += timeout_ns % 1000000000ULL;
    if (ts.tv_nsec >= 1000000000) {
        ts.tv_sec++;
        ts.tv_nsec -= 1000000000;
    }

    return pthread_cond_timedwait(&cond->cond, &mutex->mutex, &ts);
}

int mdh_cond_signal(mdh_cond_t *cond) {
    if (!cond) return -1;
    return pthread_cond_signal(&cond->cond);
}

int mdh_cond_broadcast(mdh_cond_t *cond) {
    if (!cond) return -1;
    return pthread_cond_broadcast(&cond->cond);
}

/* ========== Time Functions ========== */

uint64_t mdh_time_monotonic_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
        return 0;
    }
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

uint64_t mdh_time_realtime_ns(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_REALTIME, &ts) != 0) {
        return 0;
    }
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
}

void mdh_sleep_ms(uint32_t milliseconds) {
    struct timespec ts;
    ts.tv_sec = milliseconds / 1000;
    ts.tv_nsec = (milliseconds % 1000) * 1000000;
    nanosleep(&ts, NULL);
}

/* ========== Process Control ========== */

mdh_shell_result_t *mdh_shell_exec(const char *command, bool capture_stderr) {
    mdh_shell_result_t *result = (mdh_shell_result_t *)malloc(sizeof(mdh_shell_result_t));
    if (!result) return NULL;

    result->stdout_data = NULL;
    result->stderr_data = NULL;
    result->exit_code = -1;

    /* Build command with optional stderr redirect */
    const char *redirect = capture_stderr ? " 2>&1" : "";
    size_t cmd_len = strlen(command) + strlen(redirect) + 1;
    char *full_cmd = (char *)malloc(cmd_len);
    if (!full_cmd) {
        free(result);
        return NULL;
    }
    snprintf(full_cmd, cmd_len, "%s%s", command, redirect);

    FILE *fp = popen(full_cmd, "r");
    free(full_cmd);

    if (!fp) {
        free(result);
        return NULL;
    }

    /* Read output */
    size_t cap = 1024;
    size_t len = 0;
    char *buf = (char *)malloc(cap);
    if (!buf) {
        pclose(fp);
        free(result);
        return NULL;
    }

    size_t nread;
    char temp[1024];
    while ((nread = fread(temp, 1, sizeof(temp), fp)) > 0) {
        if (len + nread >= cap) {
            cap *= 2;
            char *newbuf = (char *)realloc(buf, cap);
            if (!newbuf) {
                free(buf);
                pclose(fp);
                free(result);
                return NULL;
            }
            buf = newbuf;
        }
        memcpy(buf + len, temp, nread);
        len += nread;
    }
    buf[len] = '\0';

    int status = pclose(fp);
    result->stdout_data = buf;
    result->exit_code = WIFEXITED(status) ? WEXITSTATUS(status) : -1;

    return result;
}

void mdh_shell_result_free(mdh_shell_result_t *result) {
    if (result) {
        free(result->stdout_data);
        free(result->stderr_data);
        free(result);
    }
}

int mdh_shell_status(const char *command) {
    int status = system(command);
    return WIFEXITED(status) ? WEXITSTATUS(status) : -1;
}

int mdh_mkstemp(char *template) {
    return mkstemp(template);
}

int mdh_unlink(const char *path) {
    return unlink(path);
}

const char *mdh_getenv(const char *name) {
    return getenv(name);
}

#endif /* !_WIN32 */
