/*
 * Platform Abstraction Layer - Windows Implementation
 */

#ifdef _WIN32

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <winsock2.h>
#include <ws2tcpip.h>
#include <io.h>
#include <fcntl.h>
#include <process.h>
#include <conio.h>

#include "platform.h"

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#pragma comment(lib, "ws2_32.lib")

/* Define ETIMEDOUT if not available (for pthread compatibility) */
#ifndef ETIMEDOUT
#define ETIMEDOUT 110
#endif

/* ========== Directory Operations ========== */

struct mdh_dir {
    HANDLE handle;
    WIN32_FIND_DATAW find_data;
    int first_read;
    mdh_dirent_t entry;
};

mdh_dir_t *mdh_opendir(const char *path) {
    mdh_dir_t *dir = (mdh_dir_t *)malloc(sizeof(mdh_dir_t));
    if (!dir) return NULL;

    /* Convert UTF-8 path to wide string and append wildcard */
    wchar_t wpath[MAX_PATH];
    int wlen = MultiByteToWideChar(CP_UTF8, 0, path, -1, wpath, MAX_PATH - 3);
    if (wlen == 0) {
        free(dir);
        return NULL;
    }

    /* Append \* for directory listing */
    size_t pathlen = wcslen(wpath);
    if (pathlen > 0 && wpath[pathlen - 1] != L'\\' && wpath[pathlen - 1] != L'/') {
        wcscat(wpath, L"\\");
    }
    wcscat(wpath, L"*");

    dir->handle = FindFirstFileW(wpath, &dir->find_data);
    if (dir->handle == INVALID_HANDLE_VALUE) {
        free(dir);
        return NULL;
    }
    dir->first_read = 1;
    return dir;
}

mdh_dirent_t *mdh_readdir(mdh_dir_t *dir) {
    if (!dir || dir->handle == INVALID_HANDLE_VALUE) return NULL;

    if (dir->first_read) {
        dir->first_read = 0;
    } else {
        if (!FindNextFileW(dir->handle, &dir->find_data)) {
            return NULL;
        }
    }

    /* Convert wide string back to UTF-8 */
    int len = WideCharToMultiByte(CP_UTF8, 0, dir->find_data.cFileName, -1,
                                   dir->entry.name, sizeof(dir->entry.name), NULL, NULL);
    if (len == 0) {
        dir->entry.name[0] = '\0';
    }
    return &dir->entry;
}

int mdh_closedir(mdh_dir_t *dir) {
    if (!dir) return -1;
    BOOL ok = FindClose(dir->handle);
    free(dir);
    return ok ? 0 : -1;
}

/* ========== String Utilities ========== */

int mdh_strcasecmp(const char *s1, const char *s2) {
    return _stricmp(s1, s2);
}

int mdh_strncasecmp(const char *s1, const char *s2, size_t n) {
    return _strnicmp(s1, s2, n);
}

/* ========== Terminal I/O ========== */

struct mdh_terminal_state {
    DWORD input_mode;
    DWORD output_mode;
};

int mdh_terminal_get_size(int *width, int *height) {
    CONSOLE_SCREEN_BUFFER_INFO csbi;
    HANDLE hConsole = GetStdHandle(STD_OUTPUT_HANDLE);

    if (hConsole == INVALID_HANDLE_VALUE) return -1;

    if (!GetConsoleScreenBufferInfo(hConsole, &csbi)) {
        return -1;
    }

    if (width) *width = csbi.srWindow.Right - csbi.srWindow.Left + 1;
    if (height) *height = csbi.srWindow.Bottom - csbi.srWindow.Top + 1;
    return 0;
}

int mdh_terminal_raw_mode(mdh_terminal_state_t **state) {
    if (!state) return -1;

    HANDLE hInput = GetStdHandle(STD_INPUT_HANDLE);
    HANDLE hOutput = GetStdHandle(STD_OUTPUT_HANDLE);

    if (hInput == INVALID_HANDLE_VALUE) return -1;

    mdh_terminal_state_t *s = (mdh_terminal_state_t *)malloc(sizeof(mdh_terminal_state_t));
    if (!s) return -1;

    /* Save original modes */
    if (!GetConsoleMode(hInput, &s->input_mode)) {
        free(s);
        return -1;
    }

    if (hOutput != INVALID_HANDLE_VALUE) {
        GetConsoleMode(hOutput, &s->output_mode);
    }

    /* Set raw mode: disable line input and echo */
    DWORD new_mode = s->input_mode;
    new_mode &= ~(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
    new_mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

    if (!SetConsoleMode(hInput, new_mode)) {
        free(s);
        return -1;
    }

    /* Enable VT processing on output for ANSI escape codes */
    if (hOutput != INVALID_HANDLE_VALUE) {
        DWORD out_mode = s->output_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        SetConsoleMode(hOutput, out_mode);
    }

    *state = s;
    return 0;
}

int mdh_terminal_restore(mdh_terminal_state_t *state) {
    if (!state) return -1;

    HANDLE hInput = GetStdHandle(STD_INPUT_HANDLE);
    HANDLE hOutput = GetStdHandle(STD_OUTPUT_HANDLE);

    int rc = 0;
    if (!SetConsoleMode(hInput, state->input_mode)) {
        rc = -1;
    }

    if (hOutput != INVALID_HANDLE_VALUE) {
        SetConsoleMode(hOutput, state->output_mode);
    }

    free(state);
    return rc;
}

int mdh_terminal_read_char(void) {
    HANDLE hInput = GetStdHandle(STD_INPUT_HANDLE);
    if (hInput == INVALID_HANDLE_VALUE) return -1;

    INPUT_RECORD record;
    DWORD read;

    while (1) {
        if (!ReadConsoleInputW(hInput, &record, 1, &read) || read == 0) {
            return -1;
        }

        if (record.EventType == KEY_EVENT && record.Event.KeyEvent.bKeyDown) {
            wchar_t wc = record.Event.KeyEvent.uChar.UnicodeChar;
            if (wc != 0) {
                /* Convert to UTF-8 (simple case: ASCII) */
                if (wc < 128) {
                    return (int)wc;
                }
                /* For non-ASCII, would need proper UTF-8 encoding */
                return (int)wc;
            }

            /* Handle special keys via virtual key code */
            WORD vk = record.Event.KeyEvent.wVirtualKeyCode;
            switch (vk) {
                case VK_UP:    return 0x1B5B41; /* ESC [ A - encoded as multi-byte */
                case VK_DOWN:  return 0x1B5B42;
                case VK_RIGHT: return 0x1B5B43;
                case VK_LEFT:  return 0x1B5B44;
                default: break;
            }
        }
    }
}

int mdh_terminal_input_available(void) {
    HANDLE hInput = GetStdHandle(STD_INPUT_HANDLE);
    if (hInput == INVALID_HANDLE_VALUE) return 0;

    DWORD events;
    if (!GetNumberOfConsoleInputEvents(hInput, &events)) {
        return 0;
    }
    return events > 0 ? 1 : 0;
}

/* ========== Networking ========== */

static int wsa_initialized = 0;

int mdh_net_init(void) {
    if (wsa_initialized) return 0;

    WSADATA wsaData;
    int result = WSAStartup(MAKEWORD(2, 2), &wsaData);
    if (result != 0) {
        return -1;
    }
    wsa_initialized = 1;
    return 0;
}

void mdh_net_cleanup(void) {
    if (wsa_initialized) {
        WSACleanup();
        wsa_initialized = 0;
    }
}

int mdh_socket_create(int domain, int type, int protocol) {
    return (int)socket(domain, type, protocol);
}

int mdh_socket_close(int sockfd) {
    return closesocket((SOCKET)sockfd);
}

int mdh_socket_errno(void) {
    return WSAGetLastError();
}

int mdh_poll(mdh_pollfd_t *fds, size_t nfds, int timeout_ms) {
    /* WSAPoll has same structure as poll on Unix */
    WSAPOLLFD *wsa_fds = (WSAPOLLFD *)fds;
    return WSAPoll(wsa_fds, (ULONG)nfds, timeout_ms);
}

/* ========== Threading ========== */

struct mdh_thread {
    HANDLE handle;
    void *(*func)(void *);
    void *arg;
    void *result;
};

struct mdh_mutex {
    SRWLOCK lock;
};

struct mdh_cond {
    CONDITION_VARIABLE cond;
};

static DWORD WINAPI thread_wrapper(LPVOID arg) {
    mdh_thread_t *t = (mdh_thread_t *)arg;
    t->result = t->func(t->arg);
    return 0;
}

mdh_thread_t *mdh_thread_create(void *(*func)(void *), void *arg) {
    mdh_thread_t *t = (mdh_thread_t *)malloc(sizeof(mdh_thread_t));
    if (!t) return NULL;

    t->func = func;
    t->arg = arg;
    t->result = NULL;

    t->handle = CreateThread(NULL, 0, thread_wrapper, t, 0, NULL);
    if (t->handle == NULL) {
        free(t);
        return NULL;
    }
    return t;
}

int mdh_thread_join(mdh_thread_t *thread, void **retval) {
    if (!thread) return -1;

    DWORD wait_result = WaitForSingleObject(thread->handle, INFINITE);
    if (wait_result != WAIT_OBJECT_0) {
        return -1;
    }

    if (retval) {
        *retval = thread->result;
    }

    CloseHandle(thread->handle);
    free(thread);
    return 0;
}

int mdh_thread_detach(mdh_thread_t *thread) {
    if (!thread) return -1;
    CloseHandle(thread->handle);
    free(thread);
    return 0;
}

mdh_mutex_t *mdh_mutex_create(void) {
    mdh_mutex_t *m = (mdh_mutex_t *)malloc(sizeof(mdh_mutex_t));
    if (!m) return NULL;

    InitializeSRWLock(&m->lock);
    return m;
}

void mdh_mutex_destroy(mdh_mutex_t *mutex) {
    if (mutex) {
        /* SRWLOCK doesn't need explicit destruction */
        free(mutex);
    }
}

int mdh_mutex_lock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    AcquireSRWLockExclusive(&mutex->lock);
    return 0;
}

int mdh_mutex_unlock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    ReleaseSRWLockExclusive(&mutex->lock);
    return 0;
}

int mdh_mutex_trylock(mdh_mutex_t *mutex) {
    if (!mutex) return -1;
    return TryAcquireSRWLockExclusive(&mutex->lock) ? 0 : -1;
}

mdh_cond_t *mdh_cond_create(void) {
    mdh_cond_t *c = (mdh_cond_t *)malloc(sizeof(mdh_cond_t));
    if (!c) return NULL;

    InitializeConditionVariable(&c->cond);
    return c;
}

void mdh_cond_destroy(mdh_cond_t *cond) {
    if (cond) {
        /* CONDITION_VARIABLE doesn't need explicit destruction */
        free(cond);
    }
}

int mdh_cond_wait(mdh_cond_t *cond, mdh_mutex_t *mutex) {
    if (!cond || !mutex) return -1;
    return SleepConditionVariableSRW(&cond->cond, &mutex->lock, INFINITE, 0) ? 0 : -1;
}

int mdh_cond_timedwait(mdh_cond_t *cond, mdh_mutex_t *mutex, uint64_t timeout_ns) {
    if (!cond || !mutex) return -1;

    DWORD timeout_ms = (DWORD)(timeout_ns / 1000000);
    if (timeout_ms == 0 && timeout_ns > 0) {
        timeout_ms = 1;
    }

    if (!SleepConditionVariableSRW(&cond->cond, &mutex->lock, timeout_ms, 0)) {
        if (GetLastError() == ERROR_TIMEOUT) {
            return ETIMEDOUT; /* Use ETIMEDOUT for compatibility */
        }
        return -1;
    }
    return 0;
}

int mdh_cond_signal(mdh_cond_t *cond) {
    if (!cond) return -1;
    WakeConditionVariable(&cond->cond);
    return 0;
}

int mdh_cond_broadcast(mdh_cond_t *cond) {
    if (!cond) return -1;
    WakeAllConditionVariable(&cond->cond);
    return 0;
}

/* ========== Time Functions ========== */

uint64_t mdh_time_monotonic_ns(void) {
    static LARGE_INTEGER frequency = {0};
    if (frequency.QuadPart == 0) {
        QueryPerformanceFrequency(&frequency);
    }

    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);

    /* Convert to nanoseconds */
    return (uint64_t)((counter.QuadPart * 1000000000ULL) / frequency.QuadPart);
}

uint64_t mdh_time_realtime_ns(void) {
    FILETIME ft;
    GetSystemTimeAsFileTime(&ft);

    /* Convert FILETIME (100-ns intervals since 1601-01-01) to Unix epoch */
    ULARGE_INTEGER uli;
    uli.LowPart = ft.dwLowDateTime;
    uli.HighPart = ft.dwHighDateTime;

    /* Subtract Windows epoch to Unix epoch difference (in 100-ns intervals) */
    /* 11644473600 seconds * 10,000,000 (100-ns per second) */
    const uint64_t EPOCH_DIFF = 116444736000000000ULL;
    uint64_t unix_100ns = uli.QuadPart - EPOCH_DIFF;

    return unix_100ns * 100; /* Convert 100-ns to ns */
}

void mdh_sleep_ms(uint32_t milliseconds) {
    Sleep(milliseconds);
}

/* ========== Process Control ========== */

mdh_shell_result_t *mdh_shell_exec(const char *command, bool capture_stderr) {
    mdh_shell_result_t *result = (mdh_shell_result_t *)malloc(sizeof(mdh_shell_result_t));
    if (!result) return NULL;

    result->stdout_data = NULL;
    result->stderr_data = NULL;
    result->exit_code = -1;

    /* Build command with cmd.exe */
    const char *redirect = capture_stderr ? " 2>&1" : "";
    size_t cmd_len = strlen("cmd.exe /c ") + strlen(command) + strlen(redirect) + 1;
    char *full_cmd = (char *)malloc(cmd_len);
    if (!full_cmd) {
        free(result);
        return NULL;
    }
    snprintf(full_cmd, cmd_len, "cmd.exe /c %s%s", command, redirect);

    FILE *fp = _popen(full_cmd, "r");
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
        _pclose(fp);
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
                _pclose(fp);
                free(result);
                return NULL;
            }
            buf = newbuf;
        }
        memcpy(buf + len, temp, nread);
        len += nread;
    }
    buf[len] = '\0';

    int status = _pclose(fp);
    result->stdout_data = buf;
    result->exit_code = status;

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
    return system(command);
}

int mdh_mkstemp(char *tmplate) {
    /* Windows doesn't have mkstemp, use _mktemp_s + _open */
    if (_mktemp_s(tmplate, strlen(tmplate) + 1) != 0) {
        return -1;
    }
    return _open(tmplate, _O_CREAT | _O_EXCL | _O_RDWR, _S_IREAD | _S_IWRITE);
}

int mdh_unlink(const char *path) {
    return _unlink(path);
}

const char *mdh_getenv(const char *name) {
    return getenv(name);
}

#endif /* _WIN32 */
