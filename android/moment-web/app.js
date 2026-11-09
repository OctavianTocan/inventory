/* The app's side of the share moment, part two: the moment plays full screen in Inventory's Android WebView, over the
   app that shared. Loaded after the engine's parts. The app talks to it through window.InvMoment (below) and hears it
   through window.InvApp.post(name, json): ready, fail, sky, sfx, tags, skip, done. The app plays the sounds and
   vibrations itself (share-moment-sounds), shows its own tag step on the card's back (share-tag-step-icon-grid), and
   blurs the page behind as the sky comes in. */
const APP = (() => {
  const bridge = window.InvApp;
  const post = (name, data) => {
    try { if (bridge) bridge.post(name, data == null ? '' : JSON.stringify(data)); else (window.__appLog = window.__appLog || []).push([name, data]); } catch (e) {}
  };
  let kept = false, skipped = false, aborted = false, tagsRes = null, inTags = false, started = false;
  const bands = document.getElementById('bands');

  /** Scales the 360 by 760 stage to the screen and centres it; the bands round it carry the sky's colours. */
  const fit = () => {
    const k = Math.min(innerWidth / W, innerHeight / H);
    scr.style.transform = `translate(${((innerWidth - W * k) / 2).toFixed(2)}px, ${((innerHeight - H * k) / 2).toFixed(2)}px) scale(${k})`;
  };
  fit(); addEventListener('resize', fit);

  return {
    post,
    get skipped() { return skipped; },
    get inTags() { return inTags; },
    set inTags(v) { inTags = v; },
    get aborted() { return aborted; },
    set aborted(v) { aborted = v; },
    set skipped(v) { skipped = v; },
    started: () => started,
    begin() { started = true; },
    /** Holds the tag step's promise until the app sends what was chosen. */
    waitTags: () => new Promise(res => { tagsRes = res; }),
    tagsDone(v) { const r = tagsRes; tagsRes = null; if (r) r(v || {}); },
    /** The sky comes in over the page: the bands outside the stage take its top and bottom colours, the app blurs. */
    skyIn(top, bottom, ms = 420) {
      bands.style.background = `linear-gradient(180deg, ${top} 0 50%, ${bottom} 50% 100%)`;
      A(bands, [{ opacity: 0 }, { opacity: 1 }], ms); post('sky', { a: 1, ms });
    },
    skyOut(ms) { if (getComputedStyle(bands).opacity !== '0') A(bands, [{ opacity: getComputedStyle(bands).opacity }, { opacity: 0 }], ms); post('sky', { a: 0, ms }); },
    /** From the star's break-off the rarity stays as it is. */
    keepRarity() { kept = true; post('rarity', { r: rarity }); },
    setRarity(r) { if (!kept && RC[r]) rarity = r; },
    /** The card's face as it is on screen, in CSS pixels: its centre and size, as if it faced the camera. */
    cardRect(c) {
      c.root.updateMatrixWorld(true);
      const p = new THREE.Vector3(); c.root.getWorldPosition(p);
      const sc = c.root.scale.x, r = glc.getBoundingClientRect();
      const at = (dx, dy) => { const v = new THREE.Vector3(p.x + dx * sc, p.y + dy * sc, p.z).project(CAM3); return [r.left + (v.x + 1) / 2 * r.width, r.top + (1 - v.y) / 2 * r.height]; };
      const [x0, y0] = at(-CW / 2, CH / 2), [x1, y1] = at(CW / 2, -CH / 2);
      return { x: (x0 + x1) / 2, y: (y0 + y1) / 2, w: x1 - x0, h: y1 - y0 };
    },
  };
})();

/* Every sound goes to the app, which plays it (only while the ringer is on) and turns it into a vibration. */
sound = false;
SFX.play = (name, arg) => { if (!skipping) APP.post('sfx', { n: name, a: arg ?? null, r: rarity }); };

/* The tag step is the app's own (its icon grid on the card's back). The card turns edge on, the app takes it from
   there, and when the app is done the card comes round face up from the other edge and flies into the backpack. */
TAGWAYS.push({ key: 'APP', name: 'The app’s tag step', d: 'The card turns over and the app shows its icon grid on the back.',
  async run(id, c) {
    if (APP.skipped) return;
    const s = c.s, fl0 = c.flipT, from = { rx: s.rx, ry: s.ry, rz: s.rz, fl: s.fl, y: c.y, sc: s.sc };
    c.interactive = false; c.idle = false; c.ptr = null; cardFree(c, true);
    const rect = APP.cardRect(c);
    SFX.play('whoosh');
    await drive(230, p => { s.fl = lerp(from.fl, fl0 + Math.PI / 2, p); s.rx = lerp(from.rx, 0, p); s.ry = lerp(from.ry, 0, p); s.rz = lerp(from.rz, 0, p); }, E.in);
    guard(id);
    c.vis = 0; APP.inTags = true;
    APP.post('tags', { rect, turnMs: 230 });
    const got = await APP.waitTags();
    APP.inTags = false; guard(id);
    if (got.space !== undefined || got.tags) { c.ts.space = got.space ? { label: got.space.label, col: got.space.col } : null; c.ts.tags = (got.tags || []).slice(0, 12); applyTags(c); }
    s.fl = fl0 + Math.PI * 1.5; c.vis = 1;
    await drive(230, p => { s.fl = lerp(fl0 + Math.PI * 1.5, fl0 + Math.PI * 2, p); }, E.out);
    guard(id);
    cardFree(c, false); c.flipT = s.fl; c.k = 70; c.d = 9; c.idle = true; c.interactive = true;
  } });
tagWay = TAGWAYS.length - 1;

/* The stage is scaled to the screen's width, so a shake would show its edges: the sky reaches past them, and a
   white-out covers the whole screen, not just the stage. */
backdrop = css => { const b = add(`<div style="position:absolute;inset:-64px;background:${css};opacity:0"></div>`); A(b, [{ opacity: 0 }, { opacity: 1 }], 420); return b; };
const APPFLASH = [], flashEl = document.getElementById('flash');
flash = (alpha = .9, dur = .25, color = '#FFFFFF') => { APPFLASH.push({ alpha, dur, color, t: 0 }); };
onFrame(r => {
  const dt = r / 1000 / slow * spd(); let a = 0, col = '#FFFFFF';
  for (let i = APPFLASH.length - 1; i >= 0; i--) { const f = APPFLASH[i]; f.t += dt; const p = f.t / f.dur; if (p >= 1) { APPFLASH.splice(i, 1); continue; } const v = f.alpha * (1 - p) * (1 - p); if (v > a) { a = v; col = f.color; } }
  flashEl.style.opacity = a.toFixed(3); if (a) flashEl.style.background = col;
});

/* The page's own reset, without the prototype's share sheet. */
function reset() {
  run++; skipping = false; playing = false; rate = 1; ts = 1; panelClose = null; trauma = 0;
  ANIMS = []; glClear(); layer.innerHTML = ''; over.innerHTML = ''; hud.innerHTML = ''; P = []; FX = []; RINGS = []; FLASH = []; EMIT = []; TW.length = 0; APPFLASH.length = 0;
  [cam, shop, glc].forEach(e => e.getAnimations().forEach(a => a.cancel()));
  cam.style.transform = ''; cam.style.translate = ''; cam.style.rotate = '';
}

/* The page comes back: the bands go with the sky and the app stops blurring. */
const appBackToPage = backToPage;
backToPage = (ms = 160) => { APP.skyOut(ms); appBackToPage(ms); };

/* A tap skips everything, the tag step too (Octavian, 5 Nov). */
const appSkipNow = skipNow;
skipNow = async () => {
  if (APP.skipped || APP.aborted || APP.inTags) return;
  APP.skipped = true; APP.post('skip');
  await appSkipNow();
  if (!APP.aborted) APP.post('done', { skipped: true });
};
document.addEventListener('click', () => { if (playing && !APP.inTags) skipNow(); });

/** Plays the moment once, then tells the app it is over. */
async function appPlay() {
  reset(); const id = run; playing = true;
  try { await wait(60, id); skipHint(); await MOMENTS[0].play(id); }
  catch (e) { if (!(e instanceof Stop)) { console.error(e); APP.post('fail', { m: String(e && e.message || e) }); } }
  if (id === run) { reset(); if (!APP.skipped && !APP.aborted) APP.post('done', { skipped: false }); }
}

/** What the app sends: the find, its library numbers and the spaces and tags ThyMind picked. */
function appFind(f) {
  if (!f) return;
  if (f.title != null) APPF.title = f.title;
  if (f.kind != null) APPF.kind = f.kind;
  if (f.glyph != null) APPF.glyph = f.glyph;
  if (f.no != null) APPF.no = f.no;
  if (f.space !== undefined) { SPACES.length = 0; if (f.space) SPACES.push({ label: f.space.label, col: f.space.col, n: 0 }); }
  if (f.tags) { AUTO.length = 0; AUTO.push(...f.tags.slice(0, 12)); }
  if (f.count != null) APPC.before = f.count;
  if (f.saved != null) APPC.saved = f.saved;
}
const APPC = { before: null, saved: 1 };
const appCount = after => APPC.before == null ? '' : (APPC.before + (after ? APPC.saved : 0)).toLocaleString('en-GB');
let appPhotoSet = false;

window.InvMoment = {
  /** Starts the moment with what is known so far; the rest may follow. */
  start(o = {}) {
    if (APP.started()) return; APP.begin();
    appFind(o.find); rarity = 1; if (o.rarity) APP.setRarity(o.rarity);
    if (!appPhotoSet) { const u = appPlaceholder(); PHOTO.src = u; new THREE.TextureLoader().load(u, t => { if (!appPhotoSet) { PHOTO_T.image = t.image; PHOTO_T.needsUpdate = true; } }); }
    appPlay();
  },
  /** The library's count of the find, once the app has it. */
  rarity(r) { APP.setRarity(r); },
  /** More about the find as it arrives (its title, picture or number, the spaces and tags). */
  find(f) { appFind(f); },
  /** The find's picture, cropped to 3 by 4. */
  photo(url) {
    const img = new Image();
    img.onload = () => { appPhotoSet = true; PHOTO.src = url; PHOTO_T.image = img; PHOTO_T.needsUpdate = true; CARDS3.forEach(c => c.redraw && c.redraw()); };
    img.src = url;
  },
  /** The tag step is done: the space and tags chosen, for the card and the backpack's label. */
  tagsDone(v) { APP.tagsDone(v); },
  /** Back while it plays: the same as a tap. */
  skip() { if (playing && !APP.inTags) skipNow(); },
  /** Nothing was saved: everything fades without the backpack, and the app says why. */
  abort() {
    if (APP.aborted) return; APP.aborted = true; const id = ++run; playing = false;
    APP.tagsDone(null); backToPage(300);
    later(() => { if (id === run) reset(); APP.post('done', { aborted: true }); }, 340);
  },
};

/* Ready once the font is in and the card's shaders are compiled, so the first frames do not stall. */
Promise.all([document.fonts.load('600 16px "Google Sans Flex"'), document.fonts.load('500 11px "Google Sans Flex"'), window.__warm || Promise.resolve()])
  .then(() => APP.post('ready', { w: innerWidth, h: innerHeight, dpr: devicePixelRatio }), () => APP.post('ready', {}));
