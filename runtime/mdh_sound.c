/* Borrowed signed16 little-endian PCM is copied before returning. The device
 * callback touches only this bridge's ring and OS buffers, never MdhValue/RC. */
#include "mdh_runtime.h"
#include <stdlib.h>
#include <string.h>
#if defined(__APPLE__)
#include <AudioToolbox/AudioToolbox.h>
#include <pthread.h>
#elif defined(_WIN32)
#include <windows.h>
#else
#include <pthread.h>
#endif

#define PCM_CAPACITY (1024u * 1024u)
extern void __mdh_builtin_error(const char *message);
static bool opened, null_device;
static unsigned channels;
static unsigned char ring[PCM_CAPACITY];
static size_t read_pos, write_pos, ring_bytes, pending_bytes;
#if defined(__APPLE__)
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static AudioQueueRef device;
static AudioQueueBufferRef buffers[3];
static size_t in_flight[3];
#define LOCK() pthread_mutex_lock(&lock)
#define UNLOCK() pthread_mutex_unlock(&lock)
static void fill_buffer(void *unused, AudioQueueRef queue, AudioQueueBufferRef buffer) {
    (void)unused;
    unsigned index = 0;
    while (index < 3 && buffers[index] != buffer) ++index;
    if (index == 3) return;
    LOCK();
    pending_bytes -= in_flight[index];
    size_t take = ring_bytes < buffer->mAudioDataBytesCapacity ? ring_bytes : buffer->mAudioDataBytesCapacity;
    size_t first = take < PCM_CAPACITY - read_pos ? take : PCM_CAPACITY - read_pos;
    memcpy(buffer->mAudioData, ring + read_pos, first);
    memcpy((unsigned char *)buffer->mAudioData + first, ring, take - first);
    read_pos = (read_pos + take) % PCM_CAPACITY;
    ring_bytes -= take;
    in_flight[index] = take;
    memset((unsigned char *)buffer->mAudioData + take, 0, buffer->mAudioDataBytesCapacity - take);
    buffer->mAudioDataByteSize = buffer->mAudioDataBytesCapacity;
    UNLOCK();
    AudioQueueEnqueueBuffer(queue, buffer, 0, NULL);
}
#else
#define LOCK() ((void)0)
#define UNLOCK() ((void)0)
#if defined(_WIN32)
static DWORD main_id;
static void capture_main(void) { main_id = GetCurrentThreadId(); }
#if defined(_MSC_VER)
#pragma section(".CRT$XCU", read)
__declspec(allocate(".CRT$XCU")) static void (*main_init)(void) = capture_main;
#else
__attribute__((constructor)) static void main_init(void) { capture_main(); }
#endif
#else
static pthread_t main_id;
__attribute__((constructor)) static void capture_main(void) { main_id = pthread_self(); }
#endif
#endif
static void main_thread(void) {
#if defined(__APPLE__)
    bool is_main = pthread_main_np() != 0;
#elif defined(_WIN32)
    bool is_main = GetCurrentThreadId() == main_id;
#else
    bool is_main = pthread_equal(pthread_self(), main_id) != 0;
#endif
    if (!is_main) __mdh_builtin_error("Nae sound aff the main thread!");
}
static bool disabled(const char *name) {
    const char *mode = getenv(name);
    return mode && strcmp(mode, "none") == 0;
}
MdhValue __mdh_sound_open(MdhValue rate, MdhValue count) {
    main_thread();
    if (rate.tag != MDH_TAG_INT || count.tag != MDH_TAG_INT ||
        (rate.data != 22050 && rate.data != 44100) || (count.data != 1 && count.data != 2))
        __mdh_builtin_error("sound_open wants rate 22050 or 44100 and channels 1 or 2");
    if (opened) __mdh_builtin_error("sound_open: sound is already open");
    channels = (unsigned)count.data;
    read_pos = write_pos = ring_bytes = pending_bytes = 0;
    null_device = disabled("MDH_SCREEN") || disabled("MDH_SOUND");
#if defined(__APPLE__)
    if (!null_device) {
        AudioStreamBasicDescription format = {0};
        format.mSampleRate = (Float64)rate.data;
        format.mFormatID = kAudioFormatLinearPCM;
        format.mFormatFlags = kLinearPCMFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked;
        format.mBytesPerPacket = format.mBytesPerFrame = channels * 2;
        format.mFramesPerPacket = 1;
        format.mChannelsPerFrame = channels;
        format.mBitsPerChannel = 16;
        OSStatus status = AudioQueueNewOutput(&format, fill_buffer, NULL, NULL, NULL, 0, &device);
        for (unsigned i = 0; !status && i < 3; ++i) {
            in_flight[i] = 0;
            status = AudioQueueAllocateBuffer(device, ((unsigned)rate.data / 200) * channels * 2, &buffers[i]);
            if (!status) {
                memset(buffers[i]->mAudioData, 0, buffers[i]->mAudioDataBytesCapacity);
                buffers[i]->mAudioDataByteSize = buffers[i]->mAudioDataBytesCapacity;
                status = AudioQueueEnqueueBuffer(device, buffers[i], 0, NULL);
            }
        }
        if (!status) status = AudioQueueStart(device, NULL);
        if (status) {
            if (device) AudioQueueDispose(device, true);
            device = NULL;
            memset(buffers, 0, sizeof(buffers));
            memset(in_flight, 0, sizeof(in_flight));
            read_pos = write_pos = ring_bytes = pending_bytes = 0;
            __mdh_builtin_error("sound_open: couldnae open the audio device");
        }
    }
#else
    null_device = true;
#endif
    opened = true;
    return __mdh_make_nil();
}
MdhValue __mdh_sound_queue(MdhValue pcm) {
    main_thread();
    if (!opened) __mdh_builtin_error("sound_queue: nae sound is open");
    if (pcm.tag != MDH_TAG_BYTES) __mdh_builtin_error("sound_queue wants bytes PCM");
    MdhBytes *bytes = (MdhBytes *)(intptr_t)pcm.data;
    if (bytes->length < 0 || bytes->length % (channels * 2) != 0)
        __mdh_builtin_error("sound_queue wants complete signed16 PCM frames");
    LOCK();
    if ((uint64_t)bytes->length > PCM_CAPACITY - pending_bytes) {
        UNLOCK();
        __mdh_builtin_error("sound_queue: PCM ring is full");
    }
    size_t length = (size_t)bytes->length;
    size_t first = length < PCM_CAPACITY - write_pos ? length : PCM_CAPACITY - write_pos;
    if (length) {
        memcpy(ring + write_pos, bytes->data, first);
        memcpy(ring, bytes->data + first, length - first);
    }
    write_pos = (write_pos + length) % PCM_CAPACITY;
    ring_bytes += length;
    pending_bytes += length;
    UNLOCK();
    return __mdh_make_nil();
}
MdhValue __mdh_sound_queued(void) {
    main_thread();
    LOCK();
    size_t samples = pending_bytes / 2;
    UNLOCK();
    return __mdh_make_int((int64_t)samples);
}
MdhValue __mdh_sound_close(void) {
    main_thread();
#if defined(__APPLE__)
    /* Synchronous disposal joins callbacks before their state is reset. */
    if (opened && !null_device) AudioQueueDispose(device, true);
    device = NULL;
    memset(buffers, 0, sizeof(buffers));
    memset(in_flight, 0, sizeof(in_flight));
#endif
    opened = false;
    read_pos = write_pos = ring_bytes = pending_bytes = 0;
    return __mdh_make_nil();
}
