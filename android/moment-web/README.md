# Share moment page

The shooting star the Android app plays after a share (`share-moment-shooting-star`) is the Inventory share-moment
prototype's own engine, run in a see-through WebView over the app that shared. This folder builds it into
`android/app/src/main/assets/moment/` (`index.html` and `moment.js`, both committed).

- `parts/*.part`: the prototype's engine (three.js, light, particles, the 3D card, points, backpack), copied as they are.
- `m/03-star.js`: the prototype's moment 03, kept to compare against; `m/app-star.js` is the copy the app plays.
- `app-pre.js` runs before the parts (what the find is, its title lines, chips and placeholder picture); `app.js` runs
  after them (the bridge to the app, the tag step handed to the app, skip, abort, sounds sent to the app).
- `build.py` joins them, points the few lines of the parts that drew the prototype's camera shop at the find, and
  bundles the result with esbuild. Each of those replacements must match exactly once, so a change to the parts that
  moves one fails the build.

Build, after changing anything here:

```bash
cd android/moment-web && npm ci && python3 build.py
```

Film it in headless Chromium at a phone's size, in moment time, with a stand-in for the app's bridge:

```bash
node tools/film.cjs /tmp/film 200 9000 5 412 915   # needs Playwright
```

The app's side is `ui/moment/WebMoment.kt` (the bridge) and `WebShootingStar.kt` (the blur, the find's picture and the
tag step on the card's back). When the phone has no WebView, the page fails, or it is not ready in 2.5 seconds, the
app's own drawing of the moment (`ShootingStar.kt`) plays instead.
