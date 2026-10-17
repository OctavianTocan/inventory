import { Editor } from './editor.js';

const ARROW = `<svg width="16" height="10" viewBox="0 0 16 10" fill="none"><path fill-rule="evenodd" clip-rule="evenodd" d="M8.69442 9.03474L15.7127 1.76706C16.0958 1.34424 16.0958 0.696719 15.7127 0.298594C15.3295 -0.0995313 14.6826 -0.0995313 14.2995 0.298594L8 6.84468L1.7005 0.298594C1.29361 -0.0995313 0.670475 -0.0995313 0.287346 0.298594C-0.0957822 0.696719 -0.0957822 1.34426 0.287346 1.76706L7.28126 9.03474C7.68815 9.43286 8.31129 9.43286 8.69442 9.03474Z" fill="#415777"/></svg>`;
const BLEED = 34;

const send = (message) => chrome.runtime.sendMessage(message);
const tell = (type, extra = {}) => parent.postMessage({ mymindDrawer: true, type, ...extra }, '*');

function el(tag, props = {}) {
  return Object.assign(document.createElement(tag), props);
}

/** Spinner and check shown while a note saves. */
class SavingIndicator {
  constructor(element) {
    this.element = element;
  }

  flash() {
    if (this.active) return;
    this.active = true;
    this.element.classList.add('is-shown');
    setTimeout(() => this.element.classList.add('is-done'), 1000);
    setTimeout(() => this.reset(), 3000);
  }

  async reset() {
    this.element.classList.remove('is-shown');
    await new Promise((resolve) => setTimeout(resolve, 300));
    this.element.classList.remove('is-done');
    this.active = false;
  }
}

/** Tag input, suggestion list and note editor of the toast, running inside the extension-origin frame. */
class Drawer {
  constructor(root) {
    this.tags = [];
    this.previousValue = '';
    this.hover = false;
    this.editing = false;

    this.tagsBar = el('div', { className: 'mymind-notification-tags' });
    this.tagList = el('div', { className: 'mymind-tag-list' });
    this.recentTrigger = el('a', { className: 'mymind-recent-trigger', innerHTML: ARROW });
    this.gradient = el('div', { className: 'mymind-notification-tags-gradient' });
    this.input = el('input', { type: 'text', placeholder: 'Add tags' });
    this.suggestions = el('div', { className: 'mymind-notification-suggestions' });
    this.suggestionList = el('div', { className: 'mymind-notification-suggestions-storage' });
    this.suggestionTitle = el('p', { className: 'mymind-notification-suggestions-title', innerText: 'Tag Suggestions' });
    this.note = el('div', { className: 'mymind-notification-note' });
    this.noteBody = el('div', { className: 'mymind-note' });
    const indicator = el('div', { className: 'mymind-saving-indicator' });
    indicator.append(el('div', { className: 'mymind-saving-spinner' }), el('span', { className: 'mymind-saving-check' }));
    this.indicator = new SavingIndicator(indicator);
    this.note.append(this.noteBody, indicator);

    this.tagsBar.append(this.tagList, this.input, this.recentTrigger);
    this.suggestions.append(this.suggestionTitle, this.suggestionList);
    const suggestionWrap = el('div', { className: 'notification-tags-suggestion-wrapper' });
    suggestionWrap.append(this.suggestions);
    this.clip = el('div', { id: 'clip' });
    this.clip.append(this.tagsBar, this.gradient, suggestionWrap);
    root.append(this.clip, this.note);

    this.editor = new Editor(this.noteBody);
    this.editor.controller.on('update', () => this.onNoteChange());
    this.editor.controller.on('focus', () => this.setEditing(true));
    this.editor.controller.on('blur', () => this.setEditing(false));
    this.note.addEventListener('mousedown', () => {
      if (document.activeElement?.tagName === 'INPUT') document.activeElement.blur();
    });

    this.input.addEventListener('keyup', (event) => this.onKeyUp(event));
    this.input.addEventListener('keydown', (event) => {
      if (event.key === 'Tab') event.target.blur();
    });
    this.input.addEventListener('blur', () => tell('closeAfter', { ms: 4000 }));
    this.suggestionList.addEventListener('click', (event) => this.onSuggestionClick(event));
    this.recentTrigger.addEventListener('click', (event) =>
      event.currentTarget.classList.contains('is-active') ? this.hideSuggestions() : this.showRecentTags());
    this.tagsBar.addEventListener('scroll', () => this.tagsBar.classList.toggle('show-gradient', this.tagsBar.scrollLeft > 50));
    this.tagList.addEventListener('click', (event) => {
      const target = event.target.closest('.remove-tag-icon');
      if (target) this.removeTag(target.closest('[data-id]').dataset.id);
    });

    document.documentElement.addEventListener('mouseenter', () => tell('hover', { value: true }));
    document.documentElement.addEventListener('mouseleave', () => tell('hover', { value: false }));
    window.addEventListener('message', (event) => {
      if (event.source === parent && event.data?.mymindDrawer === true && event.data.type === 'hover') this.setHover(event.data.value === true);
    });
    new ResizeObserver(() => this.report()).observe(this.clip);
    new ResizeObserver(() => this.report()).observe(this.note);

    setTimeout(() => {
      this.input.focus();
      tell('focus');
      this.input.addEventListener('focus', () => tell('cancelClose'));
    }, 100);
  }

  async load() {
    const init = await send({ action: 'drawerInit' });
    if (init?.error) return;
    if (init.tags.length) this.recentTrigger.classList.add('is-hidden');
    for (const tag of init.tags) this.addTag(tag);
    if (init.prose) this.editor.setProse(init.prose);
    this.report();
  }

  setHover(value) {
    this.hover = value;
    document.body.classList.toggle('hover', value);
    this.report();
  }

  setEditing(value) {
    this.editing = value;
    document.body.classList.toggle('editing-note', value);
    this.report();
  }

  get isSuggesting() {
    return document.body.classList.contains('suggesting');
  }

  setSuggesting(value) {
    document.body.classList.toggle('suggesting', value);
    this.report();
  }

  report() {
    const slot = this.clip.offsetHeight;
    const noteHeight = getComputedStyle(this.note).display === 'none' ? 0 : this.note.offsetHeight;
    tell('layout', { slot, base: slot, full: slot + noteHeight + BLEED, suggesting: this.isSuggesting, editing: this.editing });
  }

  get autocompleting() {
    return this.suggestionTitle.innerText.toLowerCase() === 'tag suggestions';
  }

  onNoteChange() {
    clearTimeout(this.noteTimeout);
    this.noteTimeout = setTimeout(async () => {
      const result = await send({ action: 'saveObjectNote', prose: this.editor.getProse() });
      this.noteTimeout = null;
      if (result?.error) {
        console.warn('note save failed', result.error);
        return;
      }
      this.indicator.flash();
    }, 1200);
  }

  onSuggestionClick(event) {
    const value = event.target.dataset.value;
    if (!value) return;
    this.createTag(value);
    this.showSuggestions();
    this.input.focus();
    tell('hoverReset');
  }

  async getSuggestions(value) {
    this.suggestionTitle.innerText = 'Tag Suggestions';
    this.recentTrigger.classList.remove('is-active');
    this.showSuggestions(await send({ action: 'getTagSuggestions', query: value }));
  }

  async showRecentTags() {
    this.recentTrigger.classList.add('is-active');
    this.setSuggesting(true);
    this.suggestionTitle.innerText = 'Recent Tags';
    this.showSuggestions(await send({ action: 'getRecentTags' }));
  }

  hasTag(name) {
    const wanted = name.trim().toLowerCase();
    return Array.from(this.tagList.children).some((child) => child.innerText.trim().toLowerCase() === wanted);
  }

  async removeTag(id) {
    const tag = this.tags.find((item) => item.id === id);
    if (tag) {
      tag.element.remove();
      this.tags.splice(this.tags.indexOf(tag), 1);
    }
    await send({ action: 'deleteTag', id });
  }

  async createTag(value) {
    if (this.hasTag(value)) {
      this.input.value = '';
      return;
    }
    const result = await send({ action: 'addTag', name: value });
    if (result?.error) {
      tell('error');
      return;
    }
    this.addTag(result);
    this.input.value = '';
  }

  addTag(tag) {
    const tagEl = el('div', { className: 'mymind-notification-tag', innerText: tag.name });
    if (tag.id) tagEl.dataset.id = tag.id;
    tagEl.append(el('span', { className: 'remove-tag-icon' }));
    this.tagList.append(tagEl);
    if (this.tagList.offsetWidth > this.tagsBar.offsetWidth / 2) {
      this.tagsBar.scroll({ top: 0, left: this.tagList.offsetWidth + this.input.offsetWidth, behavior: 'smooth' });
    }
    this.tags.push({ id: tag.id, name: tag.name, element: tagEl });
  }

  hideSuggestions() {
    this.recentTrigger.classList.remove('is-active');
    this.suggestionList.innerHTML = '';
    this.suggestions.classList.remove('active');
    this.note.classList.remove('active');
    this.setSuggesting(false);
  }

  showSuggestions(items) {
    if (items?.length) {
      this.suggestionList.innerHTML = '';
      for (const item of items) {
        const row = el('div', { className: 'mymind-notification-suggestion', innerText: item.name });
        row.dataset.value = item.name;
        this.suggestionList.append(row);
      }
      this.setSuggesting(true);
      this.suggestions.classList.add('active');
      this.note.classList.add('active');
    } else if (this.autocompleting) {
      this.hideSuggestions();
    }
  }

  onKeyUp(event) {
    event.stopPropagation();
    tell('cancelClose');
    const value = event.target.value.trim();
    this.recentTrigger.classList.add('is-hidden');

    if (value === '') {
      if (['Backspace', 'Delete'].includes(event.key) && this.tags.length && this.previousValue === '') {
        this.removeTag(this.tags[this.tags.length - 1].id);
      }
      this.showSuggestions();
    } else if (event.key !== 'Enter' && value !== this.previousValue) {
      this.getSuggestions(value);
    }
    this.previousValue = value;

    if (['ArrowDown', 'ArrowUp', 'Enter'].includes(event.key)) {
      event.preventDefault();
      const rows = Array.from(this.suggestionList.children);
      const at = rows.findIndex((row) => row.classList.contains('active'));
      const current = at !== -1 ? rows[at] : null;
      const move = (to) => {
        if (!to) return;
        to.classList.add('active');
        current.classList.remove('active');
      };
      if (event.key === 'ArrowDown') {
        if (current) move(rows[at + 1]);
        else if (value === '' && !this.recentTrigger.classList.contains('is-active')) this.showRecentTags();
        else rows[0]?.classList.add('active');
      } else if (event.key === 'ArrowUp') {
        if (current) move(rows[at - 1]);
      } else {
        const name = current?.dataset.value || value;
        if (name) {
          this.createTag(name);
          this.showSuggestions();
          this.previousValue = '';
        }
      }
    } else if (event.key === 'Escape') {
      event.preventDefault();
      if (!this.isSuggesting) {
        this.input.blur();
        tell('close');
      } else {
        this.hideSuggestions();
      }
    }
  }
}

const drawer = new Drawer(document.getElementById('app'));
drawer.load();
drawer.report();
