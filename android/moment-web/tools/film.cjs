// Films the app's share moment in headless Chromium at a phone's size, in moment time (?film), the way the app runs
// it: a stand-in for the app's bridge answers the tag step after tagMs and logs every message.
// Usage: node tools/film.cjs OUTDIR [step=200] [total=9000] [rarity=5] [width=412] [height=915] [tagMs=1200] [tapAt=]
// Frames land in OUTDIR/fNNN-ms.png; OUTDIR/log.json has the messages the page sent.
const { chromium } = require('playwright');
const http = require('http'), fs = require('fs'), path = require('path');
const [out, step = '200', total = '9000', rar = '5', w = '412', h = '915', tagMs = '1200', tapAt = ''] = process.argv.slice(2);
const MAIN = path.join(__dirname, '..', '..', 'app', 'src', 'main');
const server = http.createServer((req, res) => {
  const u = decodeURIComponent(req.url.split('?')[0]);
  const f = u.startsWith('/assets/') ? path.join(MAIN, u) : u.startsWith('/res/') ? path.join(MAIN, u) : null;
  if (!f || !fs.existsSync(f)) { res.writeHead(404); return res.end(); }
  res.writeHead(200, { 'content-type': f.endsWith('.js') ? 'text/javascript' : f.endsWith('.html') ? 'text/html' : f.endsWith('.ttf') ? 'font/ttf' : 'application/octet-stream' });
  fs.createReadStream(f).pipe(res);
});
(async () => {
  fs.mkdirSync(out, { recursive: true }); for (const f of fs.readdirSync(out)) if (f.endsWith('.png')) fs.unlinkSync(path.join(out, f));
  await new Promise(r => server.listen(0, '127.0.0.1', r)); const port = server.address().port;
  const b = await chromium.launch({ args: ['--enable-unsafe-swiftshader', '--use-angle=swiftshader', '--ignore-gpu-blocklist'] });
  const p = await b.newPage({ viewport: { width: +w, height: +h }, deviceScaleFactor: 1, hasTouch: true, isMobile: true });
  const errs = []; p.on('pageerror', e => errs.push(e.message)); p.on('console', m => { if (m.type() === 'error') errs.push(m.text().slice(0, 300)); });
  await p.addInitScript(tag => {
    window.__log = [];
    window.InvApp = { post(n, j) { const d = j ? JSON.parse(j) : null; window.__log.push([n, d]);
      if (n === 'tags') window.__tagsAt = { d, after: tag }; } };
  }, +tagMs);
  await p.goto(`http://127.0.0.1:${port}/assets/moment/index.html?film`);
  for (let i = 0; i < 100 && !(await p.evaluate(() => window.__log.some(e => e[0] === 'ready' || e[0] === 'fail'))); i++) {
    await p.evaluate(() => window.__film && window.__film(50, 1, false)); await p.waitForTimeout(100);
  }
  await p.evaluate(r => InvMoment.start({ find: { title: 'Leica M6, the rangefinder camera of the eighties', kind: 'Leica Camera', glyph: 'link', no: 'No. 1,487', count: 1486, saved: 1, space: { label: 'Cameras', col: '#4F8A5B' }, tags: ['film camera', 'rangefinder', '1980s'] }, rarity: r }), +rar);
  const DT = 50, per = Math.max(1, Math.round(+step / DT)); let t = 0, n = 0, tagT = null, tapped = false;
  while (t < +total) {
    await p.evaluate(([dt, k]) => window.__film(dt, k, true), [DT, per]); t += DT * per;
    await p.waitForTimeout(15);
    await p.screenshot({ path: path.join(out, `f${String(n++).padStart(3, '0')}-${t}.png`) });
    const ta = await p.evaluate(() => window.__tagsAt);
    if (ta && tagT == null) tagT = t;
    if (tagT != null && t - tagT >= ta.after && !(await p.evaluate(() => window.__tagsSent))) await p.evaluate(() => { window.__tagsSent = true; InvMoment.tagsDone({ space: { label: 'Cameras', col: '#4F8A5B' }, tags: ['film camera', 'rangefinder'] }); });
    if (tapAt && !tapped && t >= +tapAt) { tapped = true; await p.mouse.click(+w / 2, +h * .7); }
    if (await p.evaluate(() => window.__log.some(e => e[0] === 'done'))) break;
  }
  const log = await p.evaluate(() => window.__log.filter(e => e[0] !== 'sfx').concat([['sfx count', window.__log.filter(e => e[0] === 'sfx').length]]));
  fs.writeFileSync(path.join(out, 'log.json'), JSON.stringify({ log, errs }, null, 1));
  console.log('frames', n, 'ms', t, 'errors', JSON.stringify(errs.slice(0, 8)), '\nlog', JSON.stringify(log).slice(0, 900));
  await b.close(); server.close();
})();
