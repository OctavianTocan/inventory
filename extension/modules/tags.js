import { SignedOut } from './api.js';

const STORAGE_KEY = 'tagCache';

/** Tag suggestions backed by the replica, cached per host and account so a cold worker answers at once. */
export class TagController {
  /**
   * @param {import('./api.js').API} api - Client used to load the tags and to name the current account.
   */
  constructor(api) {
    this.api = api;
    this.reset();
  }

  reset() {
    this.scope = null;
    this.tags = [];
    this.recentTags = [];
    this.initialized = null;
    this.hydration = null;
    this.loading = null;
  }

  /**
   * Forgets every cached tag, in memory and in storage.
   * @returns {Promise<void>} Resolves once storage is cleared.
   */
  async clear() {
    this.reset();
    try {
      await chrome.storage.local.remove(STORAGE_KEY);
    } catch {
      // Nothing was persisted.
    }
  }

  /**
   * Makes sure tags of the current account are loaded: cached ones are served while the network refreshes them.
   * @returns {Promise<void>} Resolves when there is something to serve.
   * @throws {SignedOut | Error} When signed out, or when nothing is cached and the first load fails.
   */
  async ensureInitialized() {
    const scope = await this.api.scope();
    if (!scope) {
      await this.clear();
      throw new SignedOut();
    }
    if (scope !== this.scope) {
      this.reset();
      this.scope = scope;
    }
    if (this.initialized) return;
    this.hydration ??= this.hydrate(scope);
    await this.hydration;
    if (this.tags.length) {
      this.loading ??= this.load(scope).catch(() => {}).finally(() => { this.loading = null; });
      return;
    }
    this.loading ??= this.load(scope);
    try {
      await this.loading;
    } finally {
      this.loading = null;
    }
  }

  /** Marks the tags stale and refreshes them in the background. */
  invalidate() {
    this.initialized = null;
    this.ensureInitialized().catch(() => {});
  }

  async load(scope) {
    const tags = await this.api.getJSON('/tags/manual');
    const recentTags = await this.api.getJSON('/tags/recent?take=5');
    if (scope !== this.scope) return;
    this.tags = tags;
    this.recentTags = recentTags;
    this.initialized = new Date();
    this.normalize();
    this.persist();
  }

  normalize() {
    for (const tag of this.tags) tag.normalizedName = tag.name.toLowerCase();
  }

  async hydrate(scope) {
    try {
      const cache = (await chrome.storage.local.get(STORAGE_KEY))[STORAGE_KEY];
      if (!cache || cache.scope !== scope || scope !== this.scope) return;
      this.tags = cache.tags ?? [];
      this.recentTags = cache.recentTags ?? [];
      this.normalize();
    } catch {
      // No usable cache; the network load runs instead.
    }
  }

  persist() {
    try {
      chrome.storage.local.set({ [STORAGE_KEY]: { scope: this.scope, tags: this.tags, recentTags: this.recentTags } });
    } catch {
      // The cache just does not survive this worker restart.
    }
  }

  /**
   * @param {string} prefix - What the user typed.
   * @returns {{name: string, count: number}[]} Up to 8 used tags starting with the prefix, most used first.
   */
  query(prefix) {
    const wanted = prefix.toLowerCase();
    return this.tags.filter((tag) => tag.count > 0 && tag.normalizedName.startsWith(wanted)).slice(0, 8);
  }

  find(name) {
    const wanted = name.toLowerCase();
    return this.tags.find((tag) => tag.normalizedName === wanted) ?? null;
  }

  /**
   * Counts a tag that was just added to a card.
   * @param {string} name - The tag name.
   */
  add(name) {
    const tag = this.find(name);
    if (tag) tag.count++;
    else this.tags.push({ name, normalizedName: name.toLowerCase(), count: 1 });
    this.persist();
  }
}
