/* Generic memory layout for thumbv6m-none-eabi target (Cortex-M0/M0+) */
/* Used for binary size analysis - not tied to specific hardware */

MEMORY
{
  /* Generic Flash: 64KB (typical for low-end Cortex-M0) */
  FLASH : ORIGIN = 0x00000000, LENGTH = 64K

  /* Generic RAM: 8KB (typical for low-end Cortex-M0) */
  RAM : ORIGIN = 0x20000000, LENGTH = 8K
}

/* Place stack at end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
