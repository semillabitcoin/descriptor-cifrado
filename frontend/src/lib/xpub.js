// Validación cliente del xpub (formato superficial; el backend valida criptográficamente).
// Regex deriva de la spec BIP-32 + BIP-49 + BIP-84 + tpub mainnet/testnet.

export const XPUB_REGEX = /^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$/;

export function validateXpub(text) {
  if (typeof text !== 'string') return false;
  const trimmed = text.trim();
  return XPUB_REGEX.test(trimmed);
}
