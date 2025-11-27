/* Memory layout for STM32F072RBT6 */
/* See datasheet for details */

MEMORY
{
  /* Flash memory: 128KB */
  FLASH : ORIGIN = 0x08000000, LENGTH = 128K

  /* RAM: 16KB */
  RAM : ORIGIN = 0x20000000, LENGTH = 16K
}

/* The location of the stack can be overridden using the
   `_stack_start` symbol.  Place the stack at end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* The location of the .text section can be overridden using the
   `_stext` symbol.  By default it will place after .vector_table */
/* _stext = ORIGIN(FLASH) + 0x40c; */
