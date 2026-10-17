const ID_FORMAT = /^[0-9A-Za-z_-]{8,64}$/;

/**
 * Validates a server-issued id and makes it safe to put into a URL path.
 * @param {unknown} id - A card or annotation id received from the server or from a message.
 * @returns {string} The percent-encoded id.
 * @throws {Error} When the value is not a string in the replica's id format.
 */
export function pathId(id) {
  if (typeof id !== 'string' || !ID_FORMAT.test(id)) throw new Error('invalid id');
  return encodeURIComponent(id);
}
