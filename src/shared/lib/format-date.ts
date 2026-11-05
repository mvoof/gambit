const MS_PER_SECOND = 1000;

/** `25 Jan 2025, 21:48`, in the language of the interface. */
export const formatPlayedAt = (unixSeconds: number, language: string) =>
  new Intl.DateTimeFormat(language, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(unixSeconds * MS_PER_SECOND);
