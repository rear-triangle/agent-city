/** Retain keyboard position when a live snapshot replaces a control's DOM node. */
export function retainFocus(container: HTMLElement): () => void {
 const active = document.activeElement;
 if (!(active instanceof HTMLElement) || active === document.body || !container.contains(active)) return () => {};
 const attributes = ['id', 'name', 'data-action', 'data-id', 'data-select', 'data-tab'];
 const identity = attributes.flatMap(name => active.hasAttribute(name) ? [[name, active.getAttribute(name)!]] : []);
 const text = active.textContent;
 return () => {
  const replacement = active.isConnected ? active : [...container.querySelectorAll<HTMLElement>(active.tagName)]
   .find(element => identity.length ? identity.every(([name, value]) => element.getAttribute(name) === value) : element.textContent === text);
  (replacement||container.querySelector<HTMLElement>('[data-action="close"]'))?.focus({preventScroll:true});
 };
}
