/*
 * Platform Abstraction Layer for mdhavers Runtime
 *
 * Provides cross-platform APIs for:
 * - Directory operations
 * - String utilities
 * - Terminal I/O
 * - Networking
 * - Threading
 * - Time functions
 * - Process control
 */

#ifndef MDH_PLATFORM_H
#define MDH_PLATFORM_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========== Directory Operations ========== */

typedef struct mdh_dir mdh_dir_t;

typedef struct {
    char name[260];  /* MAX_PATH on Windows, sufficient for most Unix paths */
} mdh_dirent_t;

mdh_dir_t *mdh_opendir(const char *path);
mdh_dirent_t *mdh_readdir(mdh_dir_t *dir);
int mdh_closedir(mdh_dir_t *dir);

/* ========== String Utilities ========== */

int mdh_strcasecmp(const char *s1, const char *s2);
int mdh_strncasecmp(const char *s1, const char *s2, size_t n);

/* ========== Terminal I/O ========== */

typedef struct mdh_terminal_state mdh_terminal_state_t;

/* Get terminal size (returns 0 on success, -1 on error) */
int mdh_terminal_get_size(int *width, int *height);

/* Enable raw mode for character-by-character input */
int mdh_terminal_raw_mode(mdh_terminal_state_t **state);

/* Restore terminal to original state */
int mdh_terminal_restore(mdh_terminal_state_t *state);

/* Read a single character (blocking) */
int mdh_terminal_read_char(void);

/* Check if input is available (non-blocking) */
int mdh_terminal_input_available(void);

/* ========== Networking ========== */

/* Initialize networking subsystem (call once at startup) */
int mdh_net_init(void);

/* Cleanup networking subsystem (call at shutdown) */
void mdh_net_cleanup(void);

/* Create a socket */
int mdh_socket_create(int domain, int type, int protocol);

/* Close a socket */
int mdh_socket_close(int sockfd);

/* Get last socket error */
int mdh_socket_errno(void);

/* Socket operations - these are mostly portable but need errno handling */
/* bind, connect, listen, accept, send, recv are used directly */

/* Poll wrapper */
typedef struct {
    int fd;
    short events;
    short revents;
} mdh_pollfd_t;

#define MDH_POLLIN   0x0001
#define MDH_POLLOUT  0x0002
#define MDH_POLLERR  0x0008
#define MDH_POLLHUP  0x0010
#define MDH_POLLNVAL 0x0020

int mdh_poll(mdh_pollfd_t *fds, size_t nfds, int timeout_ms);

/* ========== Threading ========== */

typedef struct mdh_thread mdh_thread_t;
typedef struct mdh_mutex mdh_mutex_t;
typedef struct mdh_cond mdh_cond_t;

/* Thread creation and management */
mdh_thread_t *mdh_thread_create(void *(*func)(void *), void *arg);
int mdh_thread_join(mdh_thread_t *thread, void **retval);
int mdh_thread_detach(mdh_thread_t *thread);

/* Mutex operations */
mdh_mutex_t *mdh_mutex_create(void);
void mdh_mutex_destroy(mdh_mutex_t *mutex);
int mdh_mutex_lock(mdh_mutex_t *mutex);
int mdh_mutex_unlock(mdh_mutex_t *mutex);
int mdh_mutex_trylock(mdh_mutex_t *mutex);

/* Condition variable operations */
mdh_cond_t *mdh_cond_create(void);
void mdh_cond_destroy(mdh_cond_t *cond);
int mdh_cond_wait(mdh_cond_t *cond, mdh_mutex_t *mutex);
int mdh_cond_timedwait(mdh_cond_t *cond, mdh_mutex_t *mutex, uint64_t timeout_ns);
int mdh_cond_signal(mdh_cond_t *cond);
int mdh_cond_broadcast(mdh_cond_t *cond);

/* ========== Time Functions ========== */

/* Get monotonic time in nanoseconds */
uint64_t mdh_time_monotonic_ns(void);

/* Get wall clock time in nanoseconds since Unix epoch */
uint64_t mdh_time_realtime_ns(void);

/* Sleep for specified milliseconds */
void mdh_sleep_ms(uint32_t milliseconds);

/* ========== Process Control ========== */

/* Run a shell command and capture output */
typedef struct {
    char *stdout_data;
    char *stderr_data;
    int exit_code;
} mdh_shell_result_t;

mdh_shell_result_t *mdh_shell_exec(const char *command, bool capture_stderr);
void mdh_shell_result_free(mdh_shell_result_t *result);

/* Get shell command exit status only */
int mdh_shell_status(const char *command);

/* Temporary file operations */
int mdh_mkstemp(char *template);
int mdh_unlink(const char *path);

/* Environment variables */
const char *mdh_getenv(const char *name);

#ifdef __cplusplus
}
#endif

#endif /* MDH_PLATFORM_H */
