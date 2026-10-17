export const DEFAULT_HOST = 'https://mymind.octaviantocan.com';
export const SESSION_COOKIE = 'mm_session';

/**
 * Reads the configured replica origin.
 * @returns {Promise<string>} The origin without a trailing slash, the default host when none is set.
 */
export async function getHost() {
  const { host } = await chrome.storage.local.get('host');
  return normalizeHost(host) ?? DEFAULT_HOST;
}

/**
 * Stores the replica origin used by every request.
 * @param {string} value - An http(s) origin such as `http://localhost:4331`; empty resets to the default.
 * @returns {Promise<string>} The origin that was stored.
 * @throws {Error} When the value is not an http(s) URL.
 */
export async function setHost(value) {
  const trimmed = String(value ?? '').trim();
  if (!trimmed) {
    await chrome.storage.local.remove('host');
    return DEFAULT_HOST;
  }
  const host = normalizeHost(trimmed);
  if (!host) throw new Error('Enter an http or https address.');
  await chrome.storage.local.set({ host });
  return host;
}

/**
 * Normalises a user-typed address to an origin.
 * @param {string | undefined} value - The raw address.
 * @returns {string | null} The origin, or null when it is not an http(s) URL.
 */
export function normalizeHost(value) {
  if (!value) return null;
  try {
    const url = new URL(value);
    return url.protocol === 'http:' || url.protocol === 'https:' ? url.origin : null;
  } catch {
    return null;
  }
}
