/**
 * Format an ISO date string (YYYY-MM-DD) to a locale-friendly display.
 * Returns the input unchanged if it cannot be parsed.
 */
export function formatDate(isoDate: string): string {
  const d = new Date(isoDate);
  if (isNaN(d.getTime())) return isoDate;
  return d.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });
}

/**
 * Truncate a 40-char fingerprint to the common "short" display form:
 * first 8 + "..." + last 8.
 */
export function truncateFp(fp: string): string {
  if (fp.length <= 20) return fp;
  return `${fp.slice(0, 8)}...${fp.slice(-8)}`;
}

/**
 * Format a fingerprint with a space every 4 characters for readability.
 * E.g. "AABB CCDD EEFF 0011 ..."
 */
export function formatFp(fp: string): string {
  return fp
    .toUpperCase()
    .replace(/(.{4})/g, '$1 ')
    .trim();
}
