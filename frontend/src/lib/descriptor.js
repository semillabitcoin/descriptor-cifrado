// Detección y conversión de descriptores single-chain a multipath BIP-389.
//
// Contexto: Nunchuk Desktop exporta descriptores receive-only (/0/*) o change-only (/1/*).
// Estos no incluyen derivación multipath <a;b>/*, lo que provoca MISSING_MULTIPATH_WILDCARD
// al intentar cifrar con el backend BED.
//
// Regla par-impar sibling (alineada con Sparrow Wallet / drongo OutputDescriptor.java y BIP44/BIP389):
//   pairFor(n) = [floor(n/2)*2, floor(n/2)*2 + 1]
//   /0/* y /1/* → <0;1>/*   (receive=0, change=1, par BIP44)
//   /2/* y /3/* → <2;3>/*   (par Liana recovery)
//   Casos N≥4 escalan con la misma fórmula.

/**
 * Detecta si un descriptor usa derivación single-chain (/N/*) sin ser ya multipath.
 *
 * @param {string} s - El descriptor a analizar.
 * @returns {boolean} true si contiene al menos un `/N/*` (N entero no negativo)
 *   que no está dentro de `<a;b>`. Devuelve false si no es string, está vacío,
 *   o si el descriptor ya usa multipath (<a;b>/*) o derivación hardened (/Nh/*, /N'/*).
 */
export function detectSingleChain(s) {
  if (typeof s !== 'string' || s.length === 0) return false;
  // Regex: /\d+/* literal — no colisiona con <a;b>/* (no contiene /N/* literal)
  // y excluye /Nh/* y /N'/* porque 'h' o "'" interrumpen el match de /(\d+)\/*
  return /\/(\d+)\/\*/.test(s);
}

/**
 * Convierte todos los cosigners single-chain (/N/*) de un descriptor a multipath
 * usando la regla par-impar sibling compatible con BIP44/BIP389 y Sparrow Wallet.
 *
 * - /0/* y /1/* → /<0;1>/*
 * - /2/* y /3/* → /<2;3>/*
 * - Cosigners que ya usan <a;b>/* quedan intactos (no matchean la regex).
 * - Elimina el checksum BIP-380 trailing (#xxxxxxxx) del resultado final,
 *   ya que el checksum original ya no sería válido tras la conversión.
 *
 * @param {string} s - El descriptor a convertir.
 * @returns {string} El descriptor convertido, sin checksum.
 */
export function convertSingleChainToMultipath(s) {
  if (typeof s !== 'string') return s;

  function pairFor(n) {
    const base = Math.floor(n / 2) * 2;
    return [base, base + 1];
  }

  let result = s.replace(/\/(\d+)\/\*/g, (_, n) => {
    const [a, b] = pairFor(parseInt(n, 10));
    return `/<${a};${b}>/*`;
  });

  // Eliminar checksum BIP-380 trailing: #[a-z0-9]{8} (case-insensitive)
  result = result.replace(/\s*#[a-z0-9]{8}$/i, '');

  return result;
}
