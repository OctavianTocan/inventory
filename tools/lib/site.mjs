// Pure recording URL helpers used by developer tooling.
/** mymind's own origin; recorded keys and bodies use it, the replica serves it from `/`. */
export const APP_ORIGIN = 'https://access.mymind.com/';
/** Local path prefixes the replica serves mymind's other origins under. */
export const LOCAL_PREFIXES = [
  ['/_static/', 'https://static.accelerator.net/'],
  ['/_mediahost/', 'https://media.mymind.host/'],
  ['/_carbon/', 'https://carbon-media.accelerator.net/'],
];

/**
 * Rewrite mymind's origins in text to the replica's local paths.
 * @param {string} text Input text.
 * @returns {string} The text with every mymind origin replaced.
 */
export const localize = (text) =>
  LOCAL_PREFIXES.reduce((out, [local, origin]) => out.replaceAll(origin, local), text).replaceAll(APP_ORIGIN, '/');
