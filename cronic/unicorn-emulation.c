/*
 * Ironic Emulator Unicorn PPC LLE emulation code, C port
 * Copyright (C) 2025 Techflash
 */

#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <stdint.h>
#include <arpa/inet.h>
#include <unicorn/unicorn.h>
#include "cronic.h"

/*
 * several useful debug flags are available here to
 * enable as you wish:
 * DEBUG_MEM: Memory and I/O R/W debugging
 * DEBUG_CPU: Instruction execution logging
 * DEBUG_CPU_VERBOSE: Register dumps after every instruction
 */
#define DEBUG_MEM
#define DEBUG_CPU
/*#define DEBUG_CPU_VERBOSE*/

typedef struct {
	uint32_t start;
	uint32_t size;
	uint32_t end;
	int32_t  offset;
} memRegion_t;

static void MEM_SetupRegion(memRegion_t *r) {
	r->end = r->start + r->size;
	/*r->handle = IPC_AllocRaw(r->start, r->size);*/
}

static void EMU_DumpState(uc_engine *uc) {
	uint32_t reg[34];
	int i;
	puts("Code:");
	puts("TODO");
	puts("Registers:");
	for (i = 0; i < 32; i++) {
		uc_reg_read(uc, UC_PPC_REG_0 + i, &reg[i]);
		reg[i] = ntohl(reg[i]);
	}


	uc_reg_read(uc, UC_PPC_REG_PC, &reg[32]);
	reg[32] = ntohl(reg[32]);
	uc_reg_read(uc, UC_PPC_REG_LR, &reg[33]);
	reg[33] = ntohl(reg[33]);
	printf("r0  - r3  : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[0],  reg[1],  reg[2],  reg[3]);
	printf("r4  - r7  : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[4],  reg[5],  reg[6],  reg[7]);
	printf("r8  - r11 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[8],  reg[9],  reg[10], reg[11]);
	printf("r12 - r15 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[12], reg[13], reg[14], reg[15]);
	printf("r16 - r19 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[16], reg[17], reg[18], reg[19]);
	printf("r20 - r23 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[20], reg[21], reg[22], reg[23]);
	printf("r24 - r27 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[24], reg[25], reg[26], reg[27]);
	printf("r28 - r31 : 0x%08x 0x%08x 0x%08x 0x%08x\n", reg[28], reg[29], reg[30], reg[31]);
	printf("pc, lr    : 0x%08x 0x%08x\n", reg[32], reg[33]);
	fflush(stdout);
}


/* continue emulation */
static bool EMU_KeepGoing = true;

/* reset vector, mapped to EXI boot stub */
static uint32_t reset_vec = 0xffff0100;

/* MEM1 */
#define MEM1_START 0x0
#define MEM1_SIZE  0x01800000
static memRegion_t mem1 = { MEM1_START, MEM1_SIZE, 0, 0 };

/* MEM2 */
#define MEM2_START 0x10000000
#define MEM2_SIZE  0x04000000
static memRegion_t mem2 = { MEM2_START, MEM2_SIZE, 0, 0 };

/* Legacy regs */
#define LEGC_START 0x0c000000
#define LEGC_SIZE  0x00800000
static memRegion_t legc = { LEGC_START, LEGC_SIZE, 0, 0 };

/* Hollywood regs */
#define HLWD_START 0x0d800000
#define HLWD_SIZE  0x00800000
static memRegion_t hlwd = { HLWD_START, HLWD_SIZE, 0, 0 };

/* Hollywood regs (mirror) */
#define MIRR_START 0x0d000000
#define MIRR_SIZE  0x00800000
static memRegion_t mirr = { MIRR_START, MIRR_SIZE, 0, 0 };

/* Broadway reset vector */
#define RVEC_REAL_START 0x0d806840
#define RVEC_START      0xffff0000
#define RVEC_SIZE       0x1000
static memRegion_t rvec = { RVEC_REAL_START, RVEC_SIZE, 0, -256 };


static uint64_t MEM_Read(uc_engine *uc, uint64_t offset, uint32_t size, void *_info) {
	uint64_t val;
	memRegion_t *info = (memRegion_t *)_info;
	/*printf("MEM_Read @ 0x%08X, %d bytes\n", offset + info.start, size);
	offset = offset & 0x0FFFFFFF;
	printf("phys offset @ 0x%08X\n", offset + info.start); */
#ifdef DEBUG_MEM
	printf("MEM_Read @ 0x%08X, %d bytes\n", offset + info->start, size);
#endif
	switch (size) {
	case 4:
		val = htonl(IPC_Read32(offset + info->start + info->offset));
		break;
	case 2:	
		val = htons(IPC_Read16(offset + info->start + info->offset));
		break;
	case 1:
		val = IPC_Read8 (offset + info->start + info->offset);
		break;
	default:
		printf("FATAL: Unknown read size: %d\n", size);
		EMU_KeepGoing = false;
		return 0;
	}
#ifdef DEBUG_MEM
	printf("got val 0x%08X\n", val);
#endif
	return val;
}

static void MEM_Write(uc_engine *uc, uint64_t offset, uint32_t size, uint64_t value, void *_info) {
	memRegion_t *info = (memRegion_t *)_info;
	/*offset = offset & 0x0FFFFFFF;
	printf("phys offset @ 0x%08X\n", offset + info.start);*/
	switch (size) {
	case 4:
#ifdef DEBUG_MEM
		printf("MEM_Write @ 0x%08X, 4 bytes, value 0x%08X\n", offset + info->start, value);
#endif
		IPC_Write32(offset + info->start + info->offset, ntohl(value));
		return;
	case 2:
#ifdef DEBUG_MEM
		printf("MEM_Write @ 0x%08X, 2 bytes, value 0x%04X\n", offset + info->start, value);
#endif
		IPC_Write16(offset + info->start + info->offset, ntohs(value));
		return;
	case 1:
#ifdef DEBUG_MEM
		printf("MEM_Write @ 0x%08X, 1 byte, value 0x%02X\n", offset + info->start, value);
#endif
		IPC_Write8(value, offset + info->start + info->offset);
		return;
	default:
		printf("FATAL: Unknown write size: %d\n", size);
		EMU_KeepGoing = false;
		return;
	}
	return;
}

static bool EMU_InvalidInstHandler(uc_engine *uc, void *data) {
	uint32_t pc, inst;
	uc_reg_read(uc, UC_PPC_REG_PC, &pc);
	uc_mem_read(uc, pc, &inst, 4);
	printf("Handling invalid instruction 0x%08X\n", inst);

	if ((inst >> 26) == 0x38) { /* psq_l */
		printf("[STUB] Skipping psq_l at 0x%08X\n", pc);
		pc += 4;
		uc_reg_write(uc, UC_PPC_REG_PC, &pc);
		return true;
	}

	return false;
}

static void EMU_IntHandler(uc_engine *uc, uint32_t intno, void *data) {
    printf("Interrupt %d fired!\n", intno);
    EMU_DumpState(uc);
    sleep(1);
    return;
}

int main(int argc, char *argv[]) {
	uc_engine *uc;
	uc_err err;
	uc_hook hook;
	int ret, ipcErr;
	uint32_t pc, pcTemp, insnData;

	ret = 0;
	puts("Setting up Ironic <--> Cronic IPC interface...");
	ipcErr = IPC_Init();
	if (ipcErr) {
		puts("ERROR: problem setting up IPC interface, check logs above!");
		return 1;
	}
	puts("Setting up Unicorn emulation...");

	/* Initialize emulator in PPC 750CL mode */
	err = uc_open(UC_ARCH_PPC, UC_MODE_PPC32 | UC_MODE_BIG_ENDIAN, &uc);
	if (err)
		goto errSetup;
	err = uc_ctl(uc, UC_CTL_IO_WRITE | UC_CTL_CPU_MODEL, UC_CPU_PPC32_750CL_V2_0);
	if (err)
		goto errSetup;

	/* map MEM1 */
	puts("Setting up MEM1...");
	MEM_SetupRegion(&mem1);
	err = uc_mmio_map(uc, MEM1_START, MEM1_SIZE, MEM_Read, &mem1, MEM_Write, &mem1);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, MEM1_START, MEM1_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	/* map MEM2 */
	puts("Setting up MEM2...");
	MEM_SetupRegion(&mem2);
	err = uc_mmio_map(uc, MEM2_START, MEM2_SIZE, MEM_Read, &mem2, MEM_Write, &mem2);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, MEM2_START, MEM2_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	/* map Hollywood register range */
	puts("Setting up Hollywood registers...");
	MEM_SetupRegion(&hlwd);
	err = uc_mmio_map(uc, HLWD_START, HLWD_SIZE, MEM_Read, &hlwd, MEM_Write, &hlwd);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, HLWD_START, HLWD_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	/* map Hollywood (mirror) register range */
	puts("Setting up Hollywood (mirror) registers...");
	MEM_SetupRegion(&mirr);
	err = uc_mmio_map(uc, MIRR_START, MIRR_SIZE, MEM_Read, &mirr, MEM_Write, &mirr);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, MIRR_START, MIRR_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	/* map legacy register range */
	puts("Setting up legacy (Flipper) registers...");
	MEM_SetupRegion(&legc);
	err = uc_mmio_map(uc, LEGC_START, LEGC_SIZE, MEM_Read, &legc, MEM_Write, &legc);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, LEGC_START, LEGC_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	/* map Broadway reset vector to EXI boot stub */
	puts("Setting up reset vector...");
	MEM_SetupRegion(&rvec);
	err = uc_mmio_map(uc, RVEC_START, RVEC_SIZE, MEM_Read, &rvec, MEM_Write, &rvec);
	if (err)
		goto errSetup;
	err = uc_mem_protect(uc, RVEC_START, RVEC_SIZE, UC_PROT_ALL);
	if (err)
		goto errSetup;

	err = uc_hook_add(uc, &hook, UC_HOOK_INSN_INVALID, EMU_InvalidInstHandler, NULL, 0, 0xffffffff);
	if (err)
		goto errSetup;
	err = uc_hook_add(uc, &hook, UC_HOOK_INTR, EMU_IntHandler, NULL, 0, 0xffffffff);
	if (err)
		goto errSetup;

	puts("Starting Broadway emulation...");
	pc = reset_vec;

	while (EMU_KeepGoing) {
		/* uc_emu_start(uc, pc, 0xffffffff, 0, 0); */
		err = uc_emu_start(uc, pc, 0xffffffff, 0, 1);
		if (err)
			goto excpt;
		err = uc_reg_read(uc, UC_PPC_REG_PC, &pc);
		if (err)
			goto excpt;

		pcTemp = pc & 0x0FFFFFFF;
#ifdef DEBUG_CPU
		printf("Emulating @ 0x%08X\n", pcTemp);
#endif
		/*uc_mem_read(uc, pcTemp, &insnData, 4);*/

		if (IPC_Err)
			goto ipcErr;
	}
ipcErr:
	ret = 1;
	puts("ERROR: Ironic <--> Cronic IPC Error detected, see above for details!");
	goto out;

errSetup:
	ret = 1;
	printf("ERROR: during setup: %s\n", uc_strerror(err));
	goto out;
excpt:
	ret = 1;
	puts("excpt");
	printf("ERROR: %s, occurred @ 0x%08X\n", uc_strerror(err), pc);
	pc = pc & 0x0FFFFFFF;
	EMU_DumpState(uc);
out:
	puts("Exiting...");
	return ret;
}
