// Wrappers fetch para los endpoints axum.
// Backend retorna body de error con shape: { "error": { "code": "<UPPER_SNAKE>", "message": "<castellano>" } }
// Estos wrappers lanzan ApiError con .code, .message, .status — los componentes los muestran tal cual.

export class ApiError extends Error {
  constructor({ status, code, message }) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.code = code;
  }
}

async function unwrap(resp) {
  const ct = resp.headers.get('content-type') || '';
  if (resp.ok) {
    if (resp.status === 204) return null;
    if (ct.includes('application/json')) return resp.json();
    return resp.text();
  }
  // Error path
  let code = 'HTTP_ERROR';
  let message = `Error ${resp.status}`;
  if (ct.includes('application/json')) {
    try {
      const body = await resp.json();
      code = body?.error?.code ?? code;
      message = body?.error?.message ?? message;
    } catch {}
  }
  throw new ApiError({ status: resp.status, code, message });
}

function networkError(_e) {
  // fetch lanza TypeError cuando hay error de red, abort, CORS, etc.
  const message = 'No se pudo conectar al servidor local. Comprueba que la app esté en ejecución.';
  return new ApiError({ status: 0, code: 'NETWORK_ERROR', message });
}

export async function postJson(url, body) {
  let resp;
  try {
    resp = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function postMultipart(url, formData) {
  let resp;
  try {
    resp = await fetch(url, { method: 'POST', body: formData });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function getJson(url) {
  let resp;
  try {
    resp = await fetch(url, { method: 'GET' });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}

export async function deleteJson(url) {
  let resp;
  try {
    resp = await fetch(url, { method: 'DELETE' });
  } catch (e) {
    throw networkError(e);
  }
  return unwrap(resp);
}
