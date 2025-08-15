/*
 * Ironic Emulator "Cronic" interface
 * Copyright (C) 2025 Techflash
 *
 * Based on files in the pyronic/ directory:
 * [no copyright header]
 */

#ifndef _CRONIC_H
#define _CRONIC_H
#include <stdint.h>

extern int      IPC_Init   (void);
extern uint8_t  IPC_Read8  (uint32_t addr);
extern uint16_t IPC_Read16 (uint32_t addr);
extern uint32_t IPC_Read32 (uint32_t addr);
extern void     IPC_Write8 (uint32_t addr, uint8_t  data);
extern void     IPC_Write16(uint32_t addr, uint16_t data);
extern void     IPC_Write32(uint32_t addr, uint32_t data);


extern int IPC_Err;
#endif
