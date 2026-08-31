/** Locale-aware number formatting, used by the demo counter. */
export const formatNumber = (value: number, locale: string) =>
  new Intl.NumberFormat(locale).format(value);

/** Trims a string and collapses inner whitespace runs to a single space. */
export const normalizeSpaces = (value: string) =>
  value.trim().replace(/\s+/g, ' ');
