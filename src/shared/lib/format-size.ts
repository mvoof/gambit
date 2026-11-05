const BYTES_IN_A_KILOBYTE = 1024;
const BYTES_IN_A_MEGABYTE = BYTES_IN_A_KILOBYTE * BYTES_IN_A_KILOBYTE;
const SIZE_DECIMALS = 1;

/** `12.4 MB` or `84 kB`, with the unit in the language of the interface. */
export const formatSize = (bytes: number, language: string) => {
  const isLarge = bytes >= BYTES_IN_A_MEGABYTE;
  const format = new Intl.NumberFormat(language, {
    style: 'unit',
    unit: isLarge ? 'megabyte' : 'kilobyte',
    maximumFractionDigits: isLarge ? SIZE_DECIMALS : 0,
  });

  return format.format(
    bytes / (isLarge ? BYTES_IN_A_MEGABYTE : BYTES_IN_A_KILOBYTE)
  );
};
