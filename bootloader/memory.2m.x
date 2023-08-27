MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  BOOT2                             : ORIGIN = 0x10000000, LENGTH = 0x100
  FLASH                             : ORIGIN = 0x10000100, LENGTH = 24K - LENGTH(BOOT2)
  APPLICATION                       : ORIGIN = 0x10007000, LENGTH = 4096K - LENGTH(BOOT2)
  RAM                               : ORIGIN = 0x20000000, LENGTH = 256K
}

__bootloader_application_start = ORIGIN(APPLICATION) - ORIGIN(BOOT2);
__bootloader_application_end = ORIGIN(APPLICATION) + LENGTH(APPLICATION) - ORIGIN(BOOT2);
