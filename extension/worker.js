import { API, SignedOut } from './modules/api.js';
import { SESSION_COOKIE, getHost } from './modules/config.js';
import { pathId } from './modules/ids.js';
import { TagController } from './modules/tags.js';
import { Notification, displaySignInNotice, showUpgradeNotification } from './modules/notification.js';
import { delay, getActiveTab } from './modules/util.js';

const version = chrome.runtime.getManifest().version;
const api = new API(version);
const tags = new TagController(api);

const CARD_KEY = 'card:';

/** Remembers, per tab, the card its drawer may act on; survives a service worker restart. */
const cards = {
  async get(tabId) {
    if (!Number.isInteger(tabId)) return null;
    const entry = (await chrome.storage.session.get(CARD_KEY + tabId))[CARD_KEY + tabId];
    if (!entry || entry.scope !== (await api.scope())) return null;
    return { ...entry.card, annotations: new Set(entry.card.annotations) };
  },
  async set(tabId, card) {
    const scope = await api.scope();
    if (!scope) return;
    await chrome.storage.session.set({
      [CARD_KEY + tabId]: { scope, card: { ...card, annotations: [...card.annotations] } },
    });
  },
  delete: (tabId) => chrome.storage.session.remove(CARD_KEY + tabId),
  async clear() {
    const keys = Object.keys(await chrome.storage.session.get(null)).filter((key) => key.startsWith(CARD_KEY));
    await chrome.storage.session.remove(keys);
  },
};

function forgetAccount() {
  cards.clear();
  tags.clear();
}

chrome.storage.onChanged.addListener((changes, area) => area === 'local' && changes.host && forgetAccount());
chrome.cookies.onChanged.addListener(({ cookie }) => cookie.name === SESSION_COOKIE && forgetAccount());
chrome.tabs.onRemoved.addListener((tabId) => cards.delete(tabId));
chrome.tabs.onUpdated.addListener((tabId, change) => change.url && cards.delete(tabId));

const savingTabs = new Set();
let processingUrl = null;

const HELP_VIDEO = 'https://mymind.helpscoutdocs.com/article/42-video-types-supported-in-mymind';
const HELP_TROUBLE = 'https://mymind.helpscoutdocs.com/article/41-trouble-shooting-with-the-browser-extension';

const MENUS = [
  { title: 'Add to Inventory', id: 'mymind-add', contexts: ['selection', 'image', 'audio', 'video', 'link'] },
  { title: 'Open Inventory', id: 'mymind-open', contexts: ['action'] },
];

chrome.runtime.onInstalled.addListener(async (details) => {
  await chrome.contextMenus.removeAll();
  for (const menu of MENUS) chrome.contextMenus.create(menu);
  if (details.reason !== 'install') return;
  openTab(`${await getHost()}/`);
  try {
    await api.postJSON('/browser-extension/activate', { version });
  } catch {
    // Not signed in yet; the ping is optional.
  }
});

chrome.action.onClicked.addListener((tab) => savePage(tab));

chrome.commands.onCommand.addListener(async (command, tab) => {
  if (command === 'save-page') await savePage(tab ?? (await getActiveTab()));
});

chrome.omnibox.setDefaultSuggestion({ description: 'Search Inventory for: %s' });
chrome.omnibox.onInputEntered.addListener(async (text, disposition) => {
  const query = text.trim();
  const host = await getHost();
  const url = query ? `${host}/everything?q=${encodeURIComponent(query)}` : `${host}/`;
  if (disposition === 'currentTab') chrome.tabs.update({ url });
  else chrome.tabs.create({ url, active: disposition !== 'newBackgroundTab' });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  const { menuItemId, mediaType, linkUrl, srcUrl, selectionText, pageUrl } = info;
  if (menuItemId === 'mymind-open') {
    openTab(`${await getHost()}/`);
  } else if (menuItemId === 'mymind-add') {
    if (mediaType && srcUrl) await saveObject({ media: { url: srcUrl, type: mediaType }, page: { url: pageUrl } }, tab);
    else if (linkUrl) await saveObject({ url: linkUrl }, tab);
    else if (selectionText) await saveSelection(info, tab);
    else if (pageUrl) await savePage(tab);
  }
});

const FORBIDDEN = { error: { detail: 'Forbidden', status: 403 } };
const MAX_TAG_LENGTH = 100;
const MAX_NOTE_BYTES = 1_000_000;

/** Test seams: only the extension's own top-level page (not web-accessible) may drive saves by tab id. */
const testActions = {
  saveObject: (message) => saveObjectOfTab(message),
  savePageOfTab,
  saveSelectionOfTab,
};

/** What the toast code running in a tab's top frame may ask for. */
const contentActions = { openNewTab };

/** What the extension-origin drawer frame inside a tab may ask for; none of it names a card or annotation. */
const drawerActions = { drawerInit, getTagSuggestions, getRecentTags, addTag, deleteTag, saveObjectNote };

function isTestSender(sender) {
  return Boolean(sender.tab) && sender.frameId === 0 && sender.url === chrome.runtime.getURL('manifest.json');
}

function isDrawerSender(sender) {
  return Boolean(sender.tab) && sender.frameId !== 0 && sender.url?.split(/[?#]/)[0] === chrome.runtime.getURL('drawer.html');
}

function isContentSender(sender) {
  return Boolean(sender.tab) && sender.frameId === 0 && sender.id === chrome.runtime.id && !sender.url?.startsWith(chrome.runtime.getURL(''));
}

function actionFor(message, sender) {
  const name = typeof message?.action === 'string' ? message.action : '';
  if (isTestSender(sender) && Object.hasOwn(testActions, name)) return testActions[name];
  if (isDrawerSender(sender) && Object.hasOwn(drawerActions, name)) return drawerActions[name];
  if (isContentSender(sender) && Object.hasOwn(contentActions, name)) return contentActions[name];
  return null;
}

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  const action = actionFor(message, sender);
  if (!action) {
    sendResponse(FORBIDDEN);
    return false;
  }
  Promise.resolve().then(() => action(message, sender.tab)).then(sendResponse, (error) => sendResponse({ error: errorOf(error) }));
  return true;
});

async function saveObjectOfTab(message) {
  return saveObject(message, await chrome.tabs.get(message.tabId));
}

async function savePageOfTab(message) {
  await savePage(await chrome.tabs.get(message.tabId));
}

async function saveSelectionOfTab(message) {
  await saveSelection(message.context, await chrome.tabs.get(message.tabId));
}

function errorOf(error) {
  if (error instanceof SignedOut) return { detail: 'Please sign in.', status: 401 };
  return { detail: error?.message ?? 'Something went wrong', status: error?.status };
}

function openTab(url) {
  if (isWebUrl(url)) chrome.tabs.create({ url });
}

function isWebUrl(url) {
  try {
    const { protocol } = new URL(url);
    return protocol === 'http:' || protocol === 'https:';
  } catch {
    return false;
  }
}

async function openNewTab(message, tab) {
  openTab(await Notification.redirectOf(tab));
}

async function saveObject(data, tab) {
  tab ??= await getActiveTab();
  const source = data.page?.url ?? tab.url;
  if (data.selection) return createObject({ type: 'Content', content: data.selection.text, source }, tab);
  if (data.media) {
    if (data.media.url.startsWith('blob:')) {
      await new Notification(tab).update({
        message: `This ${data.media.type} cannot be saved. Click here to read more.`,
        status: 'error',
        redirect: HELP_VIDEO,
      });
      return;
    }
    const request = { type: 'Media', url: data.media.url, alt: data.media.alt, source };
    if (data.block) request.block = data.block;
    return createObject(request, tab);
  }
  if (data.url) {
    if (processingUrl === data.url) return;
    processingUrl = data.url;
    try {
      return await createObject({ type: 'WebPage', url: data.url, html: data.page?.html }, tab);
    } finally {
      processingUrl = null;
    }
  }
}

async function createObject(request, tab) {
  await cards.delete(tab.id);
  const notification = new Notification(tab);
  const host = await getHost();
  await notification.update({ message: 'One moment, saving...', status: 'loading', icon: 'loading-start.svg' });
  const started = performance.now();
  let response;
  try {
    response = await api.postJSON('/objects', request);
  } catch (error) {
    if (error instanceof SignedOut) {
      await displaySignInNotice(tab);
      return { error: { status: 401 } };
    }
    await notification.update({
      message: 'Could not save to Inventory. Please try again or check your internet connection.',
      status: 'error',
      redirect: HELP_TROUBLE,
    });
    return { error: { status: 0 } };
  }
  if (response.status === 200 || response.status === 201) {
    const result = await response.json();
    const elapsed = performance.now() - started;
    if (elapsed < 1000) await delay(1000 - elapsed);
    const existing = response.status === 200;
    await cards.set(tab.id, {
      id: result.id,
      annotations: new Set((result.tags ?? []).map((tag) => tag.id).filter(Boolean)),
      tags: result.tags ?? [],
      prose: result.note?.prose ?? null,
    });
    notification.update({
      drawer: true,
      message: existing ? 'Already in your mind' : 'Saved to your mind',
      status: 'success',
      redirect: `${host}/everything#${result.id}`,
      icon: 'loading-finish.svg',
    });
    if (!existing) tags.invalidate();
    result.status = response.status;
    return result;
  }
  if (response.status === 402) {
    await showUpgradeNotification(tab);
    return { error: { detail: 'Payment Required', status: 402 } };
  }
  if (response.status === 401 || (await isRefusedSession(response))) {
    notification.update({ message: 'Your session is not valid. Please sign in.', status: 'error', redirect: `${host}/signin` });
    return { error: { status: 401 } };
  }
  if (response.status === 403) {
    await notification.update({ message: 'Forbidden', status: 'error' });
    return { error: { status: 403 } };
  }
  const problem = api.hasProblem(response) ? await response.json() : null;
  notification.update({
    message: problem?.detail ?? 'Something went wrong. Please try again.',
    status: 'error',
    redirect: `${host}/`,
  });
  return { error: { detail: problem?.detail, status: response.status } };
}

async function isRefusedSession(response) {
  return response.status === 403 && (await response.clone().text()) === 'cross-origin write refused';
}

async function boundCard(tab) {
  const card = await cards.get(tab?.id);
  if (!card) throw Object.assign(new Error('Forbidden'), { status: 403 });
  return card;
}

async function drawerInit(message, tab) {
  const card = await boundCard(tab);
  return { tags: card.tags.map(({ id, name }) => ({ id, name })), prose: card.prose };
}

async function addTag(message, tab) {
  const card = await boundCard(tab);
  const name = typeof message.name === 'string' ? message.name.trim() : '';
  if (!name || name.length > MAX_TAG_LENGTH) return { error: { detail: 'That tag name is not valid.', status: 400 } };
  const response = await api.postJSON(`/objects/${pathId(card.id)}/tags`, { name });
  if (!response.ok) return { error: await api.getProblem(response) };
  const result = await response.json();
  if (result.id) card.annotations.add(result.id);
  card.tags.push({ id: result.id, name: result.name ?? name });
  tags.add(name);
  await cards.set(tab.id, card);
  return { id: result.id, name: result.name ?? name, status: response.status };
}

async function deleteTag(message, tab) {
  const card = await boundCard(tab);
  if (!card.annotations.has(message.id)) return FORBIDDEN;
  const response = await api.delete(`/annotations/${pathId(message.id)}`);
  if (!response.ok) return { error: await api.getProblem(response) };
  card.annotations.delete(message.id);
  card.tags = card.tags.filter((tag) => tag.id !== message.id);
  await cards.set(tab.id, card);
  return { status: response.status };
}

async function getTagSuggestions(message) {
  if (typeof message.query !== 'string') return [];
  await tags.ensureInitialized();
  return tags.query(message.query.slice(0, MAX_TAG_LENGTH));
}

async function getRecentTags() {
  await tags.ensureInitialized();
  return tags.recentTags;
}

async function saveObjectNote(message, tab) {
  const card = await boundCard(tab);
  const body = JSON.stringify(message.prose ?? null);
  if (typeof message.prose !== 'object' || message.prose === null || body.length > MAX_NOTE_BYTES) {
    return { error: { detail: 'That note is not valid.', status: 400 } };
  }
  const response = await api.putJSON(`/objects/${pathId(card.id)}/note`, message.prose);
  if (!response.ok) return { error: await api.getProblem(response) };
  card.prose = message.prose;
  await cards.set(tab.id, card);
  return { status: response.status };
}

async function savePage(tab) {
  if (savingTabs.has(tab.id)) return;
  savingTabs.add(tab.id);
  try {
    await capturePage(tab);
  } finally {
    savingTabs.delete(tab.id);
  }
}

async function isMindPage(url) {
  try {
    const target = new URL(url);
    const host = new URL(await getHost());
    return target.host === host.host && /^\/(everything|spaces|serendipity)\/?$/.test(target.pathname);
  } catch {
    return false;
  }
}

async function capturePage(tab) {
  if (await isMindPage(tab.url)) {
    flashBadge(tab.id, '\u{1F92F}');
    await new Notification(tab).update({
      message: 'Saving your mind from inside of your mind would cause an inception. \u{1F300}',
      status: 'error',
    });
    return;
  }
  let injected;
  try {
    [injected] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => {
        const html = document.documentElement.outerHTML;
        try {
          document.dispatchEvent(new CustomEvent('mymind-extension:page:saving', { detail: {} }));
        } catch {
          // A page that blocks custom events still saves.
        }
        return { url: location.href, page: { url: location.href, html: html.length < 5_000_000 ? html : null } };
      },
    });
  } catch (error) {
    console.error('cannot save this page', error);
    await flashBadge(tab.id, '✕');
    return;
  }
  let screenshot = null;
  try {
    screenshot = await chrome.tabs.captureVisibleTab(tab.windowId, { format: 'jpeg', quality: 87 });
  } catch (error) {
    console.error('error capturing screenshot', error);
  }
  const object = await saveObject(injected.result, tab);
  if (object?.id && screenshot) {
    const blob = await (await fetch(screenshot)).blob();
    await api.postBlob(`/objects/${pathId(object.id)}/attachments?type=screenshot`, blob).catch(() => {});
  }
}

async function flashBadge(tabId, text, color = '#FF5924') {
  try {
    await chrome.action.setBadgeBackgroundColor({ color, tabId });
    await chrome.action.setBadgeText({ text, tabId });
    await delay(2000);
    await chrome.action.setBadgeText({ text: '', tabId });
  } catch {
    // The tab closed.
  }
}

function readSelection() {
  const selection = window.getSelection();
  const text = selection?.toString().trim() ?? '';
  let htmlFragment = '';
  try {
    if (selection && selection.rangeCount > 0) {
      const holder = document.createElement('div');
      holder.appendChild(selection.getRangeAt(0).cloneRange().cloneContents());
      htmlFragment = holder.innerHTML;
    }
  } catch (error) {
    return { error: { message: error.message } };
  }
  return { selection: { text, htmlFragment }, page: { url: location.href, html: document.documentElement.outerHTML } };
}

async function saveSelection(context, tab) {
  const [injected] = await chrome.scripting.executeScript({ target: { tabId: tab.id }, func: readSelection });
  const request = injected.result;
  if (request.error) {
    console.error('error saving selection', request.error);
    return;
  }
  if (!request.selection.text && context.selectionText) request.selection = { text: context.selectionText.trim() };
  if (!request.selection.text) return;
  if (context.pageUrl) request.page.url = context.pageUrl;
  await saveObject(request, tab);
}
