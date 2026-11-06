/* The app's side of the share moment, part one: what the app tells the moment about the find, loaded before the
   engine's parts so their card faces can draw it. The prototype drew a camera shop's product; the app draws the find. */

/** The find as the app describes it: its title, the kind chip (site or kind) and its icon, and its number. */
const APPF = { title: 'Saved', kind: 'Link', glyph: 'link', no: '' };

/** A blank picture until the app sends the find's own (or the kind's placeholder is drawn). */
const APP_PHOTO0 = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';

/** Line icons for the kinds, on a 24 by 24 grid (Lucide, ISC licence). */
const APP_GLYPHS = {
  link: 'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71',
  note: 'M16 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8zM15 3v4a2 2 0 0 0 2 2h4M8 13h8M8 17h5',
  image: 'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM9 11a2 2 0 1 0 0-4 2 2 0 0 0 0 4zM21 15l-3.1-3.1a2 2 0 0 0-2.8 0L6 21',
  video: 'M5 3h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2zM10 8.5v7l6-3.5z',
  audio: 'M9 18V5l12-2v13M9 18a3 3 0 1 1-6 0 3 3 0 0 1 6 0zM21 16a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
  pdf: 'M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7zM14 2v4a2 2 0 0 0 2 2h4M8 13h2M8 17h8M14 13h2',
  file: 'M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7zM14 2v4a2 2 0 0 0 2 2h4',
};
const APP_PATHS = {};
const appPath = k => APP_PATHS[k] || (APP_PATHS[k] = new Path2D(APP_GLYPHS[k] || APP_GLYPHS.file));

/**
 * Splits the find's title into the lines the card has room for.
 * @param {CanvasRenderingContext2D} g a context already set to the title's font
 * @param {number} w the widest a line may be, in card pixels
 * @returns {string[]} one or two lines; the second ends in an ellipsis when the title does not fit
 */
function appLines(g, w) {
  const words = String(APPF.title || '').replace(/\s+/g, ' ').trim().split(' ').filter(Boolean);
  if (!words.length) return ['Saved'];
  const fit = s => g.measureText(s).width <= w;
  const cut = s => { if (fit(s)) return s; while (s.length > 1 && !fit(s + '…')) s = s.slice(0, -1).trimEnd(); return s + '…'; };
  let i = 0, first = '';
  while (i < words.length && fit(first ? first + ' ' + words[i] : words[i])) { first = first ? first + ' ' + words[i] : words[i]; i++; }
  let rest = words.slice(i).join(' ');
  // A first word too long for the line (a file name, say) breaks inside the word.
  if (!first) { let k = rest.length; while (k > 1 && !fit(rest.slice(0, k))) k--; first = rest.slice(0, k); rest = rest.slice(k); }
  return rest ? [first, cut(rest)] : [first];
}

/**
 * The kind chip's icon, drawn as the prototype drew its camera icon.
 * @param {string} col the chip's ink
 * @returns {(g: CanvasRenderingContext2D, cx: number, cy: number) => void}
 */
function appIcon(col) {
  return (g, cx, cy) => { g.save(); g.translate(cx - 5.5, cy - 5.5); g.scale(11 / 24, 11 / 24); g.strokeStyle = col; g.lineWidth = 2.2; g.lineCap = 'round'; g.lineJoin = 'round'; g.stroke(appPath(APPF.glyph)); g.restore(); };
}

/**
 * Drops empty chips and shortens the kind chip so the row fits on the card, as the prototype's row did.
 * @param {Array} items the chips, [text, options], the kind first
 * @param {number} room how wide the row may be, in card pixels
 */
function appFitChips(items, room = 186) {
  for (let i = items.length - 1; i >= 0; i--) if (!items[i][0]) items.splice(i, 1);
  const m = document.createElement('canvas').getContext('2d'); m.font = `500 11px ${FONT}`;
  const width = () => items.reduce((w, [t, o]) => w + m.measureText(t).width + 12 + (o.icon ? 14 : 0) + 4, -4);
  if (!items.length || width() <= room) return;
  let t = items[0][0];
  while (t.length > 3 && width() > room) { t = t.slice(0, -1).trimEnd(); items[0][0] = t + '…'; }
}

/**
 * The picture for a find that has none yet: night blue, a few stars and the kind's icon in gold, 600 by 800 like the
 * prototype's photo so the card's window crops it the same way.
 * @returns {string} a data URL
 */
function appPlaceholder() {
  const cv = document.createElement('canvas'); cv.width = 600; cv.height = 800; const g = cv.getContext('2d');
  const bg = g.createRadialGradient(300, 380, 20, 300, 400, 520); bg.addColorStop(0, '#1F4378'); bg.addColorStop(.6, '#0F2348'); bg.addColorStop(1, '#081430');
  g.fillStyle = bg; g.fillRect(0, 0, 600, 800);
  let s = 7; const R = () => { s = (s * 16807) % 2147483647; return s / 2147483647; };
  for (let i = 0; i < 90; i++) { g.globalAlpha = .15 + R() * .5; g.fillStyle = '#E8F0FF'; g.beginPath(); g.arc(R() * 600, R() * 800, R() * 2 + .6, 0, 7); g.fill(); }
  g.globalAlpha = 1; g.save(); g.translate(300 - 96, 400 - 96); g.scale(8, 8);
  g.shadowColor = 'rgba(243,226,176,.55)'; g.shadowBlur = 3; g.strokeStyle = '#F3E2B0'; g.lineWidth = 1.5; g.lineCap = 'round'; g.lineJoin = 'round'; g.stroke(appPath(APPF.glyph)); g.restore();
  return cv.toDataURL('image/png');
}

/* A page that cannot start (no WebGL, say) tells the app at once, so it plays its own version instead. */
window.addEventListener('error', e => { try { window.InvApp && window.InvApp.post('fail', JSON.stringify({ m: String(e.message || e) })); } catch (x) {} });
