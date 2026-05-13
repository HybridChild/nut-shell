/* Memory layout for STM32H753ZIT6 (LQFP144, 2 MB Flash, 1 MB RAM) */
/* See DS12117 and RM0433 for details */

MEMORY
{
  /* 2 MB internal Flash */
  FLASH  (rx)  : ORIGIN = 0x08000000, LENGTH = 2M

  /* 128 KB Data TCM — fast access, CPU-only (no DMA) */
  DTCM   (xrw) : ORIGIN = 0x20000000, LENGTH = 128K

  /* 512 KB AXI SRAM — default data/bss region; accessible by DMA and USB */
  RAM    (xrw) : ORIGIN = 0x24000000, LENGTH = 512K

  /* 64 KB SRAM4 — D3 domain, accessible by low-power peripherals */
  SRAM4  (xrw) : ORIGIN = 0x38000000, LENGTH = 64K
}

/* Stack defaults to top of RAM (0x24080000); DTCM is available for explicit placement */
