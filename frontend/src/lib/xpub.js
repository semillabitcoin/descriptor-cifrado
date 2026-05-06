// Validación cliente del xpub (formato superficial; el backend valida criptográficamente).
// Regex deriva de la spec BIP-32 + BIP-49 + BIP-84 + tpub mainnet/testnet.
//
// Interop Liana (descriptor-style): Liana exporta xpubs con el prefijo de fingerprint y ruta
// de derivación en formato BIP-380 descriptor, p. ej.:
//   [68a9ec24/48h/0h/0h/2h]xpub6Euvf9G...TTVMZ/<0;1>/*
// normalizeXpub strippea ese prefix y el sufijo de derivación para obtener la xpub bare
// que espera el backend /api/decrypt.
//
// Decisión D-XPUB-NORM (locked): el strip de derivación usa /.*$ (primer '/' después de
// la xpub hasta el final). La xpub base58 nunca contiene '/', así que este approach
// permisivo cubre todas las variantes BIP-380 y multipath sin enumerar regexes frágiles.

export const XPUB_REGEX = /^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$/;

/**
 * Normaliza un string xpub, aceptando tanto formato bare clásico como descriptor-style
 * de Liana/BIP-380 (con prefix [fingerprint/path] y sufijo /<0;1>/* o /*).
 *
 * - Si `text` no es string, devuelve ''.
 * - Trimea whitespace inicial/final.
 * - Strippea prefix descriptor: `[cualquier-contenido]` al inicio.
 * - Strippea sufijo de derivación: todo desde el primer '/' hasta el final.
 * - Devuelve la xpub bare, o el string limpio si no matchea formato esperado.
 */
export function normalizeXpub(text) {
  if (typeof text !== 'string') return '';
  let s = text.trim();
  // Strip prefix descriptor: [fp/path] al inicio (balanceado por ] más cercano)
  s = s.replace(/^\[[^\]]*\]/, '');
  // Strip sufijo de derivación: desde el primer '/' hasta el final
  // (la xpub base58 nunca contiene '/', así que esto es seguro)
  s = s.replace(/\/[^\s]*$/, '');
  return s;
}

/**
 * Valida si el texto es una xpub válida (bare o descriptor-style).
 * Normaliza antes de testar para aceptar ambos formatos.
 */
export function validateXpub(text) {
  if (typeof text !== 'string') return false;
  const normalized = normalizeXpub(text);
  return XPUB_REGEX.test(normalized);
}
