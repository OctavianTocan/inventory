// Injected before any page script (every frame) on both live and replica, so the only thing
// that can differ between them is what the servers send.
(() => {
  if (window.__vt) return;
  // Runs before the fake clock is installed, so this stays on real time.
  const realNow = performance.now.bind(performance);
  const setTimeoutReal = setTimeout.bind(window);
  const rafReal = requestAnimationFrame.bind(window);

  // Work that finishes on real time, not virtual time: the driver must not advance the clock while any is
  // outstanding, or a fast replica and a slow live server would reach the same virtual moment in different states.
  const work = new Map();
  let lastActivity = realNow();
  const begin = (label) => {
    const token = Symbol(label);
    work.set(token, { label, since: realNow() });
    lastActivity = realNow();
    return () => {
      work.delete(token);
      lastActivity = realNow();
    };
  };
  const track = (proto, name, label) => {
    const original = proto[name];
    if (typeof original !== 'function') return;
    proto[name] = function (...args) {
      const end = begin(label);
      try {
        const result = original.apply(this, args);
        if (result && typeof result.then === 'function') result.then(end, end);
        else end();
        return result;
      } catch (error) {
        end();
        throw error;
      }
    };
  };
  track(window, 'fetch', 'fetch');
  track(ReadableStreamDefaultReader.prototype, 'read', 'stream read');
  track(HTMLImageElement.prototype, 'decode', 'image decode');
  track(window, 'createImageBitmap', 'image bitmap');
  // Rive (the "I've watched this" button) and other WASM runtimes compile off the main thread on real time.
  for (const name of ['compile', 'compileStreaming', 'instantiate', 'instantiateStreaming']) track(WebAssembly, name, `wasm ${name}`);
  if (document.fonts) track(document.fonts, 'load', 'font load');
  const send = XMLHttpRequest.prototype.send;
  XMLHttpRequest.prototype.send = function (...args) {
    const end = begin('xhr');
    this.addEventListener('loadend', end);
    try {
      return send.apply(this, args);
    } catch (error) {
      end();
      throw error;
    }
  };
  if (globalThis.Worker) {
    const postMessage = Worker.prototype.postMessage;
    Worker.prototype.postMessage = function (...args) {
      const end = begin('worker');
      const done = () => {
        this.removeEventListener('message', done);
        end();
      };
      this.addEventListener('message', done);
      // Not every message gets a reply; give the worker a moment, then stop waiting.
      Promise.resolve().then(() => setTimeoutReal(done, 1500));
      return postMessage.apply(this, args);
    };
  }
  for (const name of ['arrayBuffer', 'blob', 'json', 'text', 'formData', 'bytes']) track(Response.prototype, name, `response ${name}`);
  for (const name of ['values', Symbol.asyncIterator]) {
    const original = ReadableStream.prototype[name];
    if (typeof original !== 'function') continue;
    ReadableStream.prototype[name] = function (...args) {
      const iterator = original.apply(this, args);
      const next = iterator.next.bind(iterator);
      iterator.next = (...nextArgs) => {
        const end = begin('stream iteration');
        const result = next(...nextArgs);
        result.then(end, end);
        return result;
      };
      return iterator;
    };
  }
  if (globalThis.IDBDatabase) {
    const transaction = IDBDatabase.prototype.transaction;
    IDBDatabase.prototype.transaction = function (...args) {
      const tx = transaction.apply(this, args);
      const end = begin('indexeddb');
      for (const type of ['complete', 'abort', 'error']) tx.addEventListener(type, end);
      return tx;
    };
    const open = IDBFactory.prototype.open;
    IDBFactory.prototype.open = function (...args) {
      const request = open.apply(this, args);
      const end = begin('indexeddb open');
      for (const type of ['success', 'error', 'blocked']) request.addEventListener(type, end);
      return request;
    };
  }

  // One seeded stream per calling line: network timing can then never change which code gets which number.
  // Origins and columns are dropped because the replica rewrites URLs inside the same lines.
  const streams = new Map();
  const callerOf = () =>
    (new Error().stack.split('\n')[3] ?? '')
      .replace(/https?:\/\/[^/\s)]+/g, '')
      .replace(/:\d+\)?$/, '')
      .replace(/^.*\//, '');
  const seeded = (site) => {
    let seed = streams.get(site);
    if (seed === undefined) {
      seed = 0x9e3779b9;
      for (const c of site) seed = Math.imul(seed ^ c.charCodeAt(0), 0x01000193);
    }
    seed = (seed + 0x6d2b79f5) | 0;
    streams.set(site, seed);
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  Math.random = function random() {
    return seeded(callerOf());
  };
  if (globalThis.crypto?.randomUUID) {
    crypto.randomUUID = function randomUUID() {
      const site = `uuid:${callerOf()}`;
      const hex = Array.from({ length: 32 }, () => Math.floor(seeded(site) * 16).toString(16));
      hex[12] = '4';
      hex[16] = ((parseInt(hex[16], 16) & 3) | 8).toString(16);
      const h = hex.join('');
      return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
    };
  }

  // history.back() and friends resolve on a later task, whenever the browser gets to it.
  for (const name of ['back', 'forward', 'go']) {
    const original = History.prototype[name];
    History.prototype[name] = function (...args) {
      const end = begin('history');
      const done = () => {
        removeEventListener('popstate', done);
        end();
      };
      addEventListener('popstate', done);
      setTimeoutReal(done, 1000);
      return original.apply(this, args);
    };
  }

  // Smooth scrolling is the browser's own animation on real time, and the scroll events it fires land on
  // whichever real frame comes next. Scroll instantly instead (same destination on both sides); the app's own
  // animations are untouched.
  const instant = (options) => (options && typeof options === 'object' && options.behavior === 'smooth' ? { ...options, behavior: 'instant' } : options);
  for (const target of [Element.prototype, window]) {
    for (const name of ['scroll', 'scrollTo', 'scrollBy']) {
      const original = target[name];
      if (typeof original !== 'function') continue;
      target[name] = function (...args) {
        return original.apply(this, typeof args[0] === 'object' ? [instant(args[0])] : args);
      };
    }
  }
  const scrollIntoView = Element.prototype.scrollIntoView;
  Element.prototype.scrollIntoView = function (options) {
    return scrollIntoView.call(this, instant(options));
  };

  // Media never plays: whether autoplay has started (frame 0) or not (poster) would otherwise depend on when the
  // video bytes arrived. The poster, or the first frame when there is none, is what both sides show.
  HTMLMediaElement.prototype.play = function () {
    return Promise.resolve();
  };
  const noAutoplay = (node) => {
    if (node instanceof HTMLMediaElement) node.autoplay = false;
    if (node.querySelectorAll) for (const media of node.querySelectorAll('video[autoplay], audio[autoplay]')) media.autoplay = false;
  };
  new MutationObserver((records) => records.forEach((r) => r.addedNodes.forEach(noAutoplay))).observe(document, { childList: true, subtree: true });
  const hold = (e) => {
    const media = e.target;
    if (!(media instanceof HTMLMediaElement)) return;
    if (!media.paused) media.pause();
    if (media.currentTime !== 0) media.currentTime = 0;
  };
  for (const type of ['play', 'playing', 'loadeddata', 'timeupdate']) document.addEventListener(type, hold, true);

  // CSS animations, transitions and element.animate() run on the compositor's real clock. Each one is frozen
  // the moment it exists and moves only when the driver advances virtual time, by the same amount on both sides.
  // "The moment it exists" matters: anything caught later has already run for however long the network took.
  const tracked = new Map();
  const freeze = (animation) => {
    if (tracked.has(animation) || animation.timeline !== document.timeline) return;
    if (animation.playState === 'finished' || animation.playState === 'idle') return;
    const rate = animation.playbackRate || 1;
    animation.playbackRate = 0;
    animation.currentTime = rate < 0 ? (animation.effect?.getComputedTiming().endTime ?? 0) : 0;
    tracked.set(animation, { elapsed: 0, rate });
  };
  const roots = new Set([document]);
  // getAnimations() flushes style, so CSS transitions and animations a DOM change just caused exist by now.
  const discover = () => {
    for (const root of roots) for (const animation of root.getAnimations()) freeze(animation);
  };
  const observer = new MutationObserver(() => {
    lastActivity = realNow();
    discover();
  });
  const watch = (root) => {
    roots.add(root);
    observer.observe(root, { subtree: true, childList: true, attributes: true, characterData: true });
    for (const type of ['transitionrun', 'animationstart']) root.addEventListener(type, discover, true);
  };
  watch(document);
  const attachShadow = Element.prototype.attachShadow;
  Element.prototype.attachShadow = function (...args) {
    const root = attachShadow.apply(this, args);
    watch(root);
    return root;
  };
  const animate = Element.prototype.animate;
  Element.prototype.animate = function (...args) {
    const animation = animate.apply(this, args);
    freeze(animation);
    return animation;
  };
  window.__vt = {
    advance(ms) {
      discover();
      for (const [animation, state] of tracked) {
        if (animation.playState === 'idle') {
          tracked.delete(animation);
          continue;
        }
        if (animation.playState === 'paused') continue;
        state.elapsed += ms;
        const end = animation.effect?.getComputedTiming().endTime ?? Infinity;
        const time = state.elapsed * Math.abs(state.rate);
        if (time >= end) {
          tracked.delete(animation);
          animation.playbackRate = state.rate;
          animation.finish();
        } else {
          animation.currentTime = state.rate < 0 ? end - time : time;
        }
      }
      discover();
    },
    pending(quietMs = 30) {
      discover();
      // Streams that never end (long polls) stop counting after a while instead of blocking forever.
      const busy = [...new Set([...work.values()].filter((w) => realNow() - w.since < 15000).map((w) => w.label))];
      if (realNow() - lastActivity < quietMs) busy.push('recent activity');
      if (document.readyState !== 'complete') busy.push(`document ${document.readyState}`);
      if (document.fonts && document.fonts.status !== 'loaded') busy.push('fonts');
      for (const img of document.images) {
        if (img.currentSrc && !img.complete && img.loading !== 'lazy') busy.push(`img ${img.currentSrc.slice(-60)}`);
      }
      for (const video of document.querySelectorAll('video')) {
        if (!video.currentSrc || video.preload === 'none' || video.error || video.networkState === 3) continue;
        if (video.readyState < (video.poster ? 1 : 2)) busy.push(`video ${video.currentSrc.slice(-60)}`);
      }
      return busy;
    },
    clocks: () => [realNow(), performance.now()],
    // Real rendering frames. Mouse moves, scroll events and resize/intersection observers are delivered on
    // these, so the driver waits for them before moving virtual time on.
    // Render-blocking loads stop frames entirely, hence the cap.
    frames: (n = 2, capMs = 5000) =>
      new Promise((resolve) => {
        setTimeoutReal(resolve, capMs);
        (function tick(k) {
          k ? rafReal(() => tick(k - 1)) : resolve();
        })(n);
      }),
    async decodeImages() {
      await Promise.all([...document.images].filter((i) => i.complete && i.naturalWidth).map((i) => i.decode().catch(() => {})));
    },
  };
})();
