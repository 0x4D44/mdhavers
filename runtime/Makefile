# mdhavers Runtime Library Makefile
#
# Builds the C runtime library for compiled mdhavers programs.
# Requires: gcc (or clang), Boehm GC (libgc)

CC = gcc
CFLAGS = -Wall -Wextra -O2 -fPIC
LDFLAGS = -lgc

# Library name
LIB_NAME = libmdh_runtime
LIB_STATIC = $(LIB_NAME).a
LIB_SHARED = $(LIB_NAME).so

# Source files
SOURCES = mdh_runtime.c
OBJECTS = $(SOURCES:.c=.o)

# Default target
all: $(LIB_STATIC) $(LIB_SHARED)

# Static library
$(LIB_STATIC): $(OBJECTS)
	ar rcs $@ $^

# Shared library
$(LIB_SHARED): $(OBJECTS)
	$(CC) -shared -o $@ $^ $(LDFLAGS)

# Object files
%.o: %.c mdh_runtime.h
	$(CC) $(CFLAGS) -c $< -o $@

# Install (requires root or prefix)
PREFIX ?= /usr/local
install: $(LIB_STATIC) $(LIB_SHARED)
	install -d $(PREFIX)/lib
	install -d $(PREFIX)/include
	install -m 644 $(LIB_STATIC) $(PREFIX)/lib/
	install -m 755 $(LIB_SHARED) $(PREFIX)/lib/
	install -m 644 mdh_runtime.h $(PREFIX)/include/
	ldconfig || true

# Uninstall
uninstall:
	rm -f $(PREFIX)/lib/$(LIB_STATIC)
	rm -f $(PREFIX)/lib/$(LIB_SHARED)
	rm -f $(PREFIX)/include/mdh_runtime.h
	ldconfig || true

# Clean
clean:
	rm -f $(OBJECTS) $(LIB_STATIC) $(LIB_SHARED)

# Debug build
debug: CFLAGS += -g -DDEBUG
debug: all

# Test the library
test: $(LIB_STATIC)
	$(CC) -o test_runtime test_runtime.c -L. -lmdh_runtime -lgc -lm
	./test_runtime

.PHONY: all clean install uninstall debug test
