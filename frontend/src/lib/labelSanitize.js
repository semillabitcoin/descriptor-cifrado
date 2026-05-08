const ALLOWED = /^[a-zA-Z0-9 _-]$/;

export function sanitizeLabelForFilename(s) {
  if (typeof s !== 'string') return '';
  let out = '';
  for (const ch of s) {
    if (ALLOWED.test(ch)) {
      out += ch;
    } else {
      out += '-';
    }
    if (out.length >= 80) break;
  }
  return out.trim();
}
