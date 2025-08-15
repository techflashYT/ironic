/*
 * Ironic Emulator "Cronic" interface
 * Copyright (C) 2025 Techflash
 *
 * Based on files in the pyronic/ directory:
 * [no copyright header]
 *
 * Safe byteswapping code, and endianness detection code,
 * based on code from Everythingnet:
 * Copyright (C) 2025 Techflash
 */

#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdbool.h>
#include <unistd.h>
#include <signal.h>
#include <sys/un.h>
#include <sys/socket.h>
#include <arpa/inet.h>

#include "cronic.h"

#define PPC_SOCK "/tmp/ironic-ppc.sock"

static bool initialized = false;
static int sock = -1;
int IPC_Err = 0;

typedef union {
	uint32_t u32[4];
	uint32_t u16[8];
	uint8_t  u8 [16];
} msg_t;
static msg_t msg;


enum {
	IRONIC_READ		= 1,
	IRONIC_WRITE		= 2,
	IRONIC_MSG		= 3,
	IRONIC_ACK		= 4,
	IRONIC_MSGNORET		= 5,
	IRONIC_PPC_READ8	= 6,
	IRONIC_PPC_READ16	= 7,
	IRONIC_PPC_READ32	= 8,
	IRONIC_PPC_WRITE8	= 9,
	IRONIC_PPC_WRITE16 	= 10,
	IRONIC_PPC_WRITE32 	= 11
};

/* In order of accuracy:
 * - We have __BYTE_ORDER__ and __ORDER_LITTLER_ENDIAN__, and they are equal?
 * - __LITTLE_ENDIAN__ or __LITTLE_ENDIAN are defined?
 *
 * And the reverse of the above for BE:
 * - We have __BYTE_ORDER__ and __ORDER_BIG_ENDIAN__, and they are equal?
 * - __BIG_ENDIAN__ or __BIG_ENDIAN are defined?
 */
#if defined(__BYTE_ORDER__) && defined(__ORDER_LITTLE_ENDIAN__)
#  if __BYTE_ORDER__ == __ORDER_LITTLE_ENDIAN__
#    define CRONIC_CPU_IS_LE 1
#  elif defined(__ORDER_BIG_ENDIAN__)
#    if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
#      define CRONIC_CPU_IS_BE 1
#    endif
#  endif
#elif defined(__BIG_ENDIAN__) || defined(_BIG_ENDIAN)
#  define CRONIC_CPU_IS_BE 1
#elif defined(__LITTLE_ENDIAN__) || defined(_LITTLE_ENDIAN)
#  define CRONIC_CPU_IS_LE 1
#else
#  error "Unable to determine endianness for this platform, or endianness is neither big nor little"
#endif

#ifdef CRONIC_CPU_IS_BE

/* we need to have a function to swap bytes, however,
 * we must beware that some platforms already provide this capability.
 */
#  ifndef __swap32
static inline uint32_t __cronic_swap32(uint32_t in) {
	return (uint32_t)((in << 24) | ((in << 8) & 0x00FF0000) |
			((in >> 8) & 0x0000FF00) | (in >> 24));
}
#    define __swap32 __cronic_swap32
/* clean up after ourselves */
#    undef CRONIC_NEED_SWAP32
#  endif
#endif



int IPC_Init(void) {
	struct sockaddr_un srv;
	int ret, set = 1;
	if (initialized) {
		puts("IPC: trying to initialize 2nd client?");
		return 1;
	}
	initialized = true;

	/* ignore sigpipe if we get it, else we crash if ironic blows up */
	signal(SIGPIPE, SIG_IGN);

	sock = socket(AF_UNIX, SOCK_STREAM, 0);
	if (sock < 0) {
		perror("socket");
		return 1;
	}
	
	/* try to gracefully downgrade it to just setting errno to EPIPE */
#ifdef SO_NOSIGPIPE
	ret = setsockopt(sock, SOL_SOCKET, SO_NOSIGPIPE, (void *)&set, sizeof(int));
	if (ret < 0) {
		perror("setsockopt (not fatal)");
		puts("Note that the PPC LLE may crash if Ironic throws an error!");
	}
#else
	puts("System does not support SO_NOSIGPIPE - note that the PPC LLE may crash if Ironic throws an error!");
#endif

	srv.sun_family = AF_UNIX;
	strcpy(srv.sun_path, PPC_SOCK);

	ret = connect(sock, (struct sockaddr *)&srv, sizeof(srv));
	if (ret < 0) {
		perror("socket");
		close(sock);
		return 1;
	}

	return 0;
}

uint8_t IPC_Read8(uint32_t addr) {
	uint8_t ret;
	msg.u32[0] = IRONIC_PPC_READ8;
	msg.u32[1] = addr;
	msg.u32[2] = 0;
	if (write(sock, &msg, 12) != 12) {
		IPC_Err = 1;
		return 0;
	}

	if (read(sock, &ret, 1) != 1) {
		IPC_Err = 1;
		return 0;
	}

	return ret;
}

uint16_t IPC_Read16(uint32_t addr) {
	uint16_t ret;
	msg.u32[0] = IRONIC_PPC_READ16;
	msg.u32[1] = addr;
	msg.u32[2] = 0;
	if (write(sock, &msg, 12) != 12) {
		IPC_Err = 1;
		return 0;
	}

	if (read(sock, &ret, 2) != 2) {
		IPC_Err = 1;
		return 0;
	}

	return ret;
}

uint32_t IPC_Read32(uint32_t addr) {
	uint32_t ret;
	msg.u32[0] = IRONIC_PPC_READ32;
	msg.u32[1] = addr;
	msg.u32[2] = 0;
	if (write(sock, &msg, 12) != 12) {
		IPC_Err = 1;
		return 0;
	}

	if (read(sock, &ret, 4) != 4) {
		IPC_Err = 1;
		return 0;
	}

	return ret;
}

void IPC_Write8(uint32_t addr, uint8_t data) {
	char resp[2];
	msg.u32[0]  = IRONIC_PPC_WRITE8;
	msg.u32[1]  = addr;
	msg.u32[2]  = 0;
	msg.u8 [12] = data;
	if (write(sock, &msg, 13) != 13) {
		IPC_Err = 1;
		return;
	}

	if (read(sock, resp, 2) != 2) {
		IPC_Err = 1;
		return;
	}

	if (strncmp(resp, "OK", 2) != 0) {
		IPC_Err = 1;
		return;
	}

	return;
}

void IPC_Write16(uint32_t addr, uint16_t data) {
	char resp[2];
	msg.u32[0] = IRONIC_PPC_WRITE16;
	msg.u32[1] = addr;
	msg.u32[2] = 0;
	msg.u16[6] = data;
	if (write(sock, &msg, 14) != 14) {
		IPC_Err = 1;
		return;
	}

	if (read(sock, resp, 2) != 2) {
		IPC_Err = 1;
		return;
	}

	if (strncmp(resp, "OK", 2) != 0) {
		IPC_Err = 1;
		return;
	}

	return;
}

void IPC_Write32(uint32_t addr, uint32_t data) {
	char resp[2];
	msg.u32[0] = IRONIC_PPC_WRITE32;
	msg.u32[1] = addr;
	msg.u32[2] = 0;
	msg.u32[3] = data;
	if (write(sock, &msg, 16) != 16) {
		IPC_Err = 1;
		return;
	}

	if (read(sock, resp, 2) != 2) {
		IPC_Err = 1;
		return;
	}

	if (strncmp(resp, "OK", 2) != 0) {
		IPC_Err = 1;
		return;
	}

	return;
}
