// Helpers de descarga client-side. Los datos viven solo en memoria del navegador
// hasta que el usuario gatille la descarga (D-16: nada se persiste cliente-side).

function base64ToBytes(b64) {
  const binary = atob(b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export function triggerDownloadBytes(bytes, filename, mime = 'application/octet-stream') {
  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  // Revocar tras un tick para que el navegador termine la descarga.
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export function triggerDownloadBase64(b64, filename, mime = 'application/octet-stream') {
  triggerDownloadBytes(base64ToBytes(b64), filename, mime);
}

export function triggerDownloadText(text, filename, mime = 'text/plain') {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
