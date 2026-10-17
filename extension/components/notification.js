const mymind = (self.mymind ??= {});

(() => {
  const send = (message) => chrome.runtime.sendMessage(message);

  function el(tag, props = {}) {
    return Object.assign(document.createElement(tag), props);
  }

  /** The toast: status line, then (after a save) the tag and note drawer. */
  mymind.Notification = class {
    constructor(element) {
      this.element = element;
      this.hovering = false;
      element.onmouseenter = () => {
        this.cancelClose();
        this.hovering = true;
      };
      element.onmouseleave = () => {
        this.hovering = false;
        if (!this.isActive && this.status !== 'loading') this.closeAfter(2000);
      };
      this.onKeyDown = (event) => {
        if (event.key === 'Escape' && !this.isActive) this.close();
      };
      document.addEventListener('keydown', this.onKeyDown);

      this.wrap = el('div', { className: 'mymind-notification-wrap' });
      this.inner = el('div', { className: 'mymind-notification-inner' });
      this.iconEl = el('img');
      this.text = el('span');
      this.inner.append(this.iconEl, this.text);
      this.wrap.append(this.inner);
      element.append(this.wrap);
      this.inner.addEventListener('click', () => this.onInnerClick());
    }

    onInnerClick() {
      if (!this.redirect) return;
      send({ action: 'openNewTab', url: this.redirect });
      this.close();
    }

    get isActive() {
      return Boolean(document.activeElement?.closest('#mymind-shadow'));
    }

    get closing() {
      return this.element.classList.contains('closing');
    }

    get status() {
      return this._status;
    }

    set status(value) {
      this.element.classList.remove('loading', 'error', 'success');
      this.element.classList.add(value);
      this._status = value;
    }

    cancelClose() {
      this.element.classList.remove('scheduled-to-close');
      clearTimeout(this.removalTimeout);
      this.removalTimeout = null;
    }

    closeAfter(timeout = 4000) {
      if (this.hovering) return;
      this.cancelClose();
      this.element.style.setProperty('--time-to-close', `${timeout}ms`);
      this.element.classList.add('scheduled-to-close');
      this.removalTimeout = setTimeout(() => this.close(), timeout);
    }

    close() {
      if (this.closing) return;
      this.element.classList.add('closing');
      this.element.addEventListener('animationend', () => this.destroy());
    }

    update(props) {
      this.redirect = props.redirect;
      this.iconEl.src = props.icon;
      this.renderMessage(props.message);

      this.drawer?.destroy();
      this.drawer = null;

      if (props.drawer && props.status === 'success') {
        this.drawer = new mymind.DrawerFrame(this);
        this.wrap.append(this.drawer.element);
        this.drawer.element.animate(
          [{ marginTop: `-${this.drawer.height}px` }, { marginTop: '0px' }],
          { duration: 300, easing: 'ease-out', fill: 'forwards' },
        );
        this.closeAfter(4000);
      }

      if (props.status) this.status = props.status;
    }

    renderMessage(message) {
      this.text.replaceChildren();
      for (const part of [message].flat()) {
        if (typeof part === 'string') this.text.append(document.createTextNode(part));
        else if (part.br) this.text.append(document.createElement('br'));
        else if (part.link) this.text.append(el('a', { textContent: part.link, style: 'color:#FF5924;text-decoration:underline' }));
      }
    }

    destroy() {
      document.removeEventListener('keydown', this.onKeyDown);
      clearTimeout(this.removalTimeout);
      this.drawer?.destroy();
      mymind.notification = null;
      this.element.remove();
    }
  };

  /**
   * The tag and note drawer. Its content lives in an extension-origin iframe the saved page cannot read; this side
   * only places the frame and relays layout and timing signals.
   */
  mymind.DrawerFrame = class {
    constructor(notification) {
      this.notification = notification;
      this.height = 64;
      this.base = 104;
      this.full = 104;
      this.frameHover = false;

      this.element = el('div', { className: 'mymind-drawer' });
      this.slot = el('div', { className: 'mymind-drawer-slot' });
      this.frame = el('iframe', { className: 'mymind-drawer-frame', src: chrome.runtime.getURL('drawer.html') });
      this.frame.setAttribute('allowtransparency', 'true');
      this.element.append(this.frame, this.slot);
      this.slot.style.height = `${this.height}px`;

      this.onMessage = (event) => this.receive(event);
      window.addEventListener('message', this.onMessage);
      this.onParentHover = () => this.syncHover();
      notification.element.addEventListener('mouseenter', this.onParentHover);
      notification.element.addEventListener('mouseleave', this.onParentHover);
    }

    get hovered() {
      return this.notification.element.matches(':hover') || this.frameHover;
    }

    receive(event) {
      const data = event.data;
      if (event.source !== this.frame.contentWindow || data?.mymindDrawer !== true) return;
      const notification = this.notification;
      switch (data.type) {
        case 'layout':
          this.layout(data);
          break;
        case 'hover':
          this.frameHover = data.value === true;
          notification.hovering = this.hovered;
          if (notification.hovering) notification.cancelClose();
          else if (!notification.isActive && notification.status !== 'loading') notification.closeAfter(2000);
          this.syncHover();
          break;
        case 'cancelClose':
          notification.cancelClose();
          break;
        case 'focus':
          getSelection()?.removeAllRanges();
          notification.cancelClose();
          break;
        case 'closeAfter':
          notification.closeAfter(Number(data.ms) || 4000);
          break;
        case 'hoverReset':
          notification.hovering = false;
          break;
        case 'close':
          notification.close();
          break;
        case 'error':
          notification.update({
            message: 'Could not add a tag. Please try again later.',
            status: 'error',
            icon: chrome.runtime.getURL('mymind/icons/logo.svg'),
          });
          break;
        default:
      }
    }

    layout({ slot, base, full, suggesting, editing }) {
      const number = (value, fallback) => (Number.isFinite(value) && value >= 0 && value < 2000 ? value : fallback);
      this.height = number(slot, this.height);
      this.base = number(base, this.base);
      this.full = number(full, this.full);
      this.slot.style.height = `${this.height}px`;
      const element = this.notification.element;
      element.classList.toggle('suggesting', suggesting === true);
      const wasEditing = element.classList.contains('editing-note');
      element.classList.toggle('editing-note', editing === true);
      if (editing === true && !wasEditing) this.notification.cancelClose();
      else if (editing !== true && wasEditing) this.notification.closeAfter(2000);
      this.resize();
    }

    syncHover() {
      this.resize();
      this.frame.contentWindow?.postMessage({ mymindDrawer: true, type: 'hover', value: this.hovered }, chrome.runtime.getURL(''));
    }

    resize() {
      const open = this.hovered || this.notification.element.classList.contains('editing-note');
      this.frame.style.height = `${open ? this.full : this.base}px`;
    }

    destroy() {
      window.removeEventListener('message', this.onMessage);
      this.notification.element.removeEventListener('mouseenter', this.onParentHover);
      this.notification.element.removeEventListener('mouseleave', this.onParentHover);
      this.notification.element.classList.remove('suggesting', 'editing-note');
      this.element.remove();
    }
  };

  /** The "mind is full" toast. */
  mymind.UpgradeNotification = class extends mymind.Notification {
    constructor(element, upgradeUrl) {
      super(element);
      this.redirect = upgradeUrl;
      const url = (path) => chrome.runtime.getURL(`mymind/${path}`);
      element.innerHTML = `<div class="mymind-notification-wrap">
        <div class="mymind-notification-inner">
          <div class="mymind-notification-image" style="background-image:url('${url('icons/logo.svg')}');"></div>
          <span>Your mind is full. <br> Please upgrade your plan.</span>
        </div>
        <div class="mymind-notification-full" style="background-image:url('${url('images/full-mind-bg.png')}');">
          <img src="${url('images/become-a-mastermind.svg')}">
          <a target="_blank" href="${upgradeUrl}">SEE OUR PLANS</a>
        </div></div>`;
      element.classList.add('upgrade-notification');
      this.inner = element.querySelector('.mymind-notification-inner');
      this.inner.addEventListener('click', () => this.onInnerClick());
    }
  };

  mymind.showUpgradeNotification = async (upgradeUrl) => {
    if (mymind.notification) {
      if (mymind.notification.element.dataset.type === 'upgrade') return;
      mymind.notification.close();
    }
    const shadow = await mymind.getShadow();
    const element = el('mymind-notification');
    element.dataset.type = 'upgrade';
    shadow.append(element);
    mymind.notification = new mymind.UpgradeNotification(element, upgradeUrl);
    mymind.notification.closeAfter(4000);
  };

  mymind.getNotification = async () => {
    if (mymind.notification) return mymind.notification;
    const shadow = await mymind.getShadow();
    const element = el('mymind-notification');
    shadow.append(element);
    mymind.notification = new mymind.Notification(element);
    return mymind.notification;
  };

  mymind.getShadow = async () => {
    if (mymind.shadow) return mymind.shadow;
    const root = el('mymind-root', { id: 'mymind-shadow' });
    Object.assign(root.style, { display: 'block', position: 'fixed', zIndex: '2147483647', top: '0px', right: '0px' });
    const shadow = root.attachShadow({ mode: 'open' });

    const font = el('style', {
      textContent: `@font-face { font-family: 'Nunito'; src: url('${chrome.runtime.getURL('mymind/fonts/nunito.woff2')}') format('woff2'); }`,
    });
    document.body.append(font);

    const sheet = new CSSStyleSheet();
    sheet.replaceSync(await fetch(chrome.runtime.getURL('components/notification.css')).then((response) => response.text()));
    shadow.adoptedStyleSheets = [sheet];

    mymind.shadow = shadow;
    document.body.append(root);
    return shadow;
  };
})();
