#include <AudioToolbox/AudioToolbox.h>
static OSStatus fake_enqueue(AudioQueueRef queue, AudioQueueBufferRef buffer, UInt32 n, const AudioStreamPacketDescription *desc) {
    (void)queue; (void)buffer; (void)n; (void)desc; return 0;
}
#define AudioQueueEnqueueBuffer fake_enqueue
#include "../../runtime/mdh_sound.c"
#include <assert.h>
#include <stdio.h>
void __mdh_builtin_error(const char *message) { fprintf(stderr, "%s\n", message); abort(); }
MdhValue __mdh_make_nil(void) { MdhValue v = {MDH_TAG_NIL, 0}; return v; }
MdhValue __mdh_make_int(int64_t n) { MdhValue v = {MDH_TAG_INT, n}; return v; }
int main(void) {
    setenv("MDH_SOUND", "none", 1);
    __mdh_sound_open(__mdh_make_int(22050), __mdh_make_int(2));
    unsigned char source[] = {0, 128, 255, 127, 10, 0, 246, 255};
    MdhBytes bytes = {source, 8, 8};
    MdhValue pcm = {MDH_TAG_BYTES, (int64_t)(intptr_t)&bytes};
    /* Place the next copy across the ring boundary. */
    read_pos = write_pos = PCM_CAPACITY - 4;
    __mdh_sound_queue(pcm);
    memset(source, 7, 8); /* The caller may mutate or release immediately. */
    assert(__mdh_sound_queued().data == 4);
    unsigned char output[16];
    /* Apple's capacity field is const, so initialize via an aggregate. */
    AudioQueueBuffer actual = {16, output, 0, NULL, 0, NULL, 0};
    buffers[0] = &actual;
    fill_buffer(NULL, NULL, &actual);
    unsigned char expected[] = {0, 128, 255, 127, 10, 0, 246, 255, 0, 0, 0, 0, 0, 0, 0, 0};
    assert(memcmp(output, expected, 16) == 0);
    assert(ring_bytes == 0 && pending_bytes == 8);
    assert(__mdh_sound_queued().data == 4); /* In-flight data still counts. */
    fill_buffer(NULL, NULL, &actual);
    assert(__mdh_sound_queued().data == 0);
    for (unsigned i = 0; i < 16; ++i) assert(output[i] == 0);
    __mdh_sound_close();
    puts("sound ring copy/wrap/in-flight/underrun OK");
}
