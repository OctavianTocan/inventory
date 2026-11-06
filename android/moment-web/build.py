"""Builds the share moment the Android app plays (share-moment-shooting-star) into app/src/main/assets/moment/.

The moment is the prototype's own engine (parts/, copied from the share-moment prototype) and its moment 03, with the
app's two files round it: app-pre.js (what the find is) before the parts, app.js (the bridge to the app) after them.
A few lines of the parts that drew the prototype's camera shop are pointed at the find instead; each replacement must
match exactly, so a change to the parts that moves one fails the build instead of shipping the shop.

Usage, from android/moment-web: npm ci && python3 build.py
"""
import json
import os
import re
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, '..', 'app', 'src', 'main', 'assets', 'moment')
PARTS = ['engine', 'gl', 'kit', 'props', 'bgs', 'spins', 'looks', 'forms', 'tagc', 'picks']


def read(*path):
    with open(os.path.join(HERE, *path), encoding='utf-8') as f:
        return f.read()


def replace(src, old, new, count=1):
    """Replaces old with new, and fails unless old occurs exactly count times."""
    found = src.count(old)
    if found != count:
        raise SystemExit(f'build.py: expected {count} of {old[:70]!r}, found {found}')
    return src.replace(old, new)


parts = {p: read('parts', p + '.part') for p in PARTS}

# The card's picture is the find's, sent by the app; until then a blank, then the kind's placeholder.
parts['gl'] = replace(parts['gl'], "PHOTO.src = 'photos/camera.jpg';", 'PHOTO.src = APP_PHOTO0;')
parts['gl'] = replace(parts['gl'], "new THREE.TextureLoader().load('photos/camera.jpg')", 'new THREE.TextureLoader().load(APP_PHOTO0)')
# The stage is scaled up to the screen, so the 3D is drawn at the screen's own sharpness (lowered again if slow).
parts['gl'] = replace(parts['gl'], 'let DPR = Math.min(window.devicePixelRatio || 1, 2);',
                      'let DPR = Math.min((window.devicePixelRatio || 1) * Math.min(innerWidth / 360, innerHeight / 760), 2.25);')
# The shader warm-up is a promise the app waits for before it starts the moment.
parts['gl'] = replace(parts['gl'], 'setTimeout(async () => { if (playing) return; const r0 = rarity, W3',
                      'window.__warm = (async () => { await new Promise(r => setTimeout(r, 30)); if (playing) return; const r0 = rarity, W3')
parts['gl'] = replace(parts['gl'], "} catch (e) { console.warn('warm', e.message); } rarity = r0; }, 400);",
                      "} catch (e) { console.warn('warm', e.message); } rarity = r0; })();")

# The card says what the find is: its title in up to two lines, its kind (site or kind) and its number.
parts['looks'] = replace(parts['looks'], "g.fillText('Rangefinder camera,', nx, o.y); g.fillText('1970s', nx, o.y + (o.size || 16) * 1.2);",
                         "{ const TL = appLines(g, al === 'center' ? 172 : 176); g.fillText(TL[0], nx, o.y); if (TL[1]) g.fillText(TL[1], nx, o.y + (o.size || 16) * 1.2); }")
parts['looks'] = replace(parts['looks'], "['Camera', { icon: camIcon, fg: sub, bg: chipBg, stroke: o.chipStroke }]",
                         "[APPF.kind, { icon: appIcon(sub), fg: sub, bg: chipBg, stroke: o.chipStroke }]")
parts['looks'] = replace(parts['looks'], "['No. 024', { fg: sub, bg: chipBg, stroke: o.chipStroke }]", "[APPF.no, { fg: sub, bg: chipBg, stroke: o.chipStroke }]")
# The points show under the card and on the backpack, so the chip row keeps its room for the site's name.
parts['looks'] = replace(parts['looks'], ", ['+' + PTS[r], { fg: o.ptsFg || C, bg: chipBg, stroke: o.chipStroke, icon: (g, cx, cy) => gem2d(g, cx, cy, 10, GEMT[GEMR[r]]) }]];", '];')
parts['looks'] = replace(parts['looks'], 'const cy = o.y + (o.size || 16) * 1.2 + 10;', 'appFitChips(items); const cy = o.y + (o.size || 16) * 1.2 + 10;')

# A find may have no space yet; the backpack counts the library, and its label names the space when there is one.
parts['kit'] = replace(parts['kit'], "const tagsOf = ts => [{ k: 'space', label: ts.space.label, col: ts.space.col }, ...",
                       "const tagsOf = ts => [...(ts.space ? [{ k: 'space', label: ts.space.label, col: ts.space.col }] : []), ...")
parts['kit'] = replace(parts['kit'], 'tabular-nums">1,485</span>', 'tabular-nums${APPC.before == null ? \';display:none\' : \'\'}">${appCount(false)}</span>')
parts['kit'] = replace(parts['kit'], "bag.querySelector('.cnt').textContent = '1,486';", "bag.querySelector('.cnt').textContent = appCount(true);")
parts['kit'] = replace(parts['kit'], 'lab.innerHTML = `<i style="width:8px;height:8px;border-radius:50%;background:${space.col}"></i>${space.label}<span',
                       'lab.innerHTML = `${space ? `<i style="width:8px;height:8px;border-radius:50%;background:${space.col}"></i>${esc(space.label)}` : \'\'}<span')

js = '\n'.join([read('app-pre.js')] + [parts[p] for p in PARTS] + [read('m', 'app-star.js'), read('app.js')])
js = re.sub(r"\(['\"]photos/camera\.jpg['\"]\)", '(APP_PHOTO0)', js)
assert 'photos/camera.jpg' not in js, 'the shop photo is still referenced'

imports = """import * as THREE from 'three';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { SavePass } from 'three/addons/postprocessing/SavePass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
"""
os.makedirs(OUT, exist_ok=True)
tmp = os.path.join(HERE, 'build')
os.makedirs(tmp, exist_ok=True)
with open(os.path.join(tmp, 'entry.js'), 'w', encoding='utf-8') as f:
    f.write(imports + js)
r = subprocess.run([os.path.join(HERE, 'node_modules', '.bin', 'esbuild'), os.path.join(tmp, 'entry.js'), '--bundle', '--format=iife', '--minify',
                    '--target=chrome100', '--legal-comments=none', '--log-level=error', '--outfile=' + os.path.join(OUT, 'moment.js')], cwd=HERE)
if r.returncode:
    raise SystemExit('build.py: esbuild failed')

# The page: the prototype's styles, its phone screen without the shop, and the app's own fitting.
head, html = read('parts', 'head.css.part'), read('parts', 'html.part')
css = head[head.index('<style>') + 7:head.index('</style>')] + '\n' + html[html.index('<style>') + 7:html.index('</style>')]
css = replace(css, 'src: url(google_sans_flex.ttf)', 'src: url(/res/font/google_sans_flex.ttf)')
page = f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no">
<title>Share moment</title>
<style>{css}
html, body {{ margin: 0; height: 100%; overflow: hidden; background: transparent !important; }}
#bands {{ position: fixed; inset: 0; opacity: 0; pointer-events: none; }}
#flash {{ position: fixed; inset: 0; opacity: 0; pointer-events: none; background: #FFF; z-index: 5; }}
.scr {{ position: absolute; left: 0; top: 0; transform-origin: 0 0; overflow: hidden; border-radius: 0; background: transparent; cursor: default; }}
.shop {{ display: none; }}
</style></head>
<body><div id="bands"></div>
<div class="scr" id="scr" aria-label="Saved. Tap to skip">
<div class="cam" id="cam"><div class="shop" id="shop"><div id="page"></div><div class="share" id="share"><div class="me"><span></span></div></div></div>
<div class="layer" id="layer"></div>
<canvas class="fxb" id="fxb" width="720" height="1520"></canvas>
<canvas class="gl" id="gl"></canvas>
<div class="over" id="over"></div>
<canvas class="fx" id="fx" width="720" height="1520"></canvas>
</div>
<div class="hud" id="hud"></div>
</div>
<div id="flash"></div>
<script src="moment.js"></script>
</body></html>
"""
with open(os.path.join(OUT, 'index.html'), 'w', encoding='utf-8') as f:
    f.write(page)
print('moment.js', os.path.getsize(os.path.join(OUT, 'moment.js')), 'bytes; index.html', len(page), 'bytes')
