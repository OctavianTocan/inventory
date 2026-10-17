/**
 * Names the device class mymind's server renders differently for.
 * @param {string} userAgent Request user agent.
 * @returns {string} `iphone`, `ipad`, `android-phone`, `android-tablet`, or empty for desktop.
 */
export function deviceVariant(userAgent = '') {
  if (/iPhone|iPod/.test(userAgent)) return 'iphone';
  if (/iPad/.test(userAgent)) return 'ipad';
  if (/Android.*Mobile/.test(userAgent)) return 'android-phone';
  if (/Android/.test(userAgent)) return 'android-tablet';
  return '';
}
