import { getHost } from './config.js';

const SCRIPTS = ['components/notification.js'];

/** The in-page toast of one tab; every call injects the page-side code first when it is missing. */
export class Notification {
  static count = 0;

  /**
   * @param {chrome.tabs.Tab} tab - A tab.
   * @returns {Promise<string | undefined>} Where clicking that tab's toast goes, as last set by the worker.
   */
  static async redirectOf(tab) {
    const key = `redirect:${tab?.id}`;
    return (await chrome.storage.session.get(key))[key];
  }

  static rememberRedirect(tab, url) {
    return chrome.storage.session.set({ [`redirect:${tab.id}`]: url ?? null });
  }

  /**
   * @param {chrome.tabs.Tab} tab - Tab that shows the toast.
   */
  constructor(tab) {
    this.tab = tab;
  }

  /** Injects the page-side toast code once per page. */
  async initialize() {
    const target = { tabId: this.tab.id };
    const [injected] = await chrome.scripting.executeScript({ target, func: () => window.mymind?.Notification !== undefined });
    if (!injected.result) await chrome.scripting.executeScript({ target, files: SCRIPTS });
  }

  /**
   * Shows or replaces the toast.
 * @param {{message: string | (string | {br: true} | {link: string})[], status?: string, icon?: string, redirect?: string, drawer?: boolean}} props - Toast content; `icon` names a file in `icons/`, `drawer` adds the tag and note frame.
   * @returns {Promise<void>} Resolves once the page has been told.
   */
  async update(props) {
    Notification.count += 1;
    await Notification.rememberRedirect(this.tab, props.redirect);
    Object.assign(props, { id: Notification.count, icon: chrome.runtime.getURL(`icons/${props.icon ?? 'logo.svg'}`) });
    await this.initialize();
    try {
      await chrome.scripting.executeScript({
        target: { tabId: this.tab.id },
        func: async (value) => (await mymind.getNotification()).update(value),
        args: [props],
      });
    } catch (error) {
      console.log('update notification error', error);
    }
  }
}

/**
 * Shows the "mind is full" toast.
 * @param {chrome.tabs.Tab} tab - Tab that shows it.
 * @returns {Promise<void>} Resolves once the page has been told.
 */
export async function showUpgradeNotification(tab) {
  await new Notification(tab).initialize();
  const upgrade = `${await getHost()}/upgrade?source=extension`;
  await Notification.rememberRedirect(tab, upgrade);
  await chrome.scripting.executeScript({
    target: { tabId: tab.id },
    func: (url) => window.mymind.showUpgradeNotification(url),
    args: [upgrade],
  });
}

/**
 * Tells the user to sign in.
 * @param {chrome.tabs.Tab} tab - Tab that shows the toast.
 * @returns {Promise<void>} Resolves once the page has been told.
 */
export async function displaySignInNotice(tab) {
  await new Notification(tab).update({
    message: ['Could not save to mymind. ', { br: true }, ' Please make sure to ', { link: 'sign in.' }],
    status: 'error',
    redirect: `${await getHost()}/signin`,
  });
}
