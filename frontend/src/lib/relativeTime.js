// Convierte ISO timestamp a "hace N días" / "hace N horas" / etc. en castellano.
// Source: Intl.RelativeTimeFormat con locale 'es' — produce "hace 3 días" nativamente.

const RTF = new Intl.RelativeTimeFormat('es', { numeric: 'auto' });

export function formatRelative(isoTimestamp) {
  const then = new Date(isoTimestamp);
  if (isNaN(then.getTime())) return isoTimestamp;
  const now = new Date();
  const diffSec = Math.round((then.getTime() - now.getTime()) / 1000);
  const absSec = Math.abs(diffSec);

  if (absSec < 60) return RTF.format(diffSec, 'second');
  if (absSec < 3600) return RTF.format(Math.round(diffSec / 60), 'minute');
  if (absSec < 86400) return RTF.format(Math.round(diffSec / 3600), 'hour');
  if (absSec < 2592000) return RTF.format(Math.round(diffSec / 86400), 'day');
  if (absSec < 31536000) return RTF.format(Math.round(diffSec / 2592000), 'month');
  return RTF.format(Math.round(diffSec / 31536000), 'year');
}
