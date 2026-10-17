/** The note editor in the toast: tiptap with the formatting mymind notes allow. */
export class Editor {
  /**
   * @param {HTMLElement} element - Host element the editor mounts in.
   */
  constructor(element) {
    this.element = element;
    this.controller = new tiptap.Editor({
      element,
      extensions: [
        tiptap.StarterKit,
        tiptap.Link,
        tiptap.Highlight,
        tiptap.TaskList,
        tiptap.TaskItem,
        tiptap.SubScript,
        tiptap.SuperScript,
        tiptap.Placeholder.configure({ placeholder: 'Type here to add a mind note...' }),
      ],
    });
    element.classList.add('prose', 'editable');
    element.classList.toggle('empty', this.isEmpty);
  }

  /** @returns {object} The note as ProseMirror JSON. */
  getProse() {
    return this.controller.getJSON();
  }

  /** @returns {boolean} Whether the note has no text. */
  get isEmpty() {
    const dom = this.controller.view.dom;
    if (dom.childElementCount === 0) return true;
    if (dom.childElementCount === 1 && dom.firstElementChild.tagName === 'P') return !dom.firstElementChild.textContent.length;
    return false;
  }

  /**
   * @param {object} prose - ProseMirror JSON to show.
   */
  setProse(prose) {
    this.controller.commands.setContent(prose);
    this.element.classList.toggle('empty', this.isEmpty);
  }
}
