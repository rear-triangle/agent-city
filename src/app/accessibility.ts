export type Accessibility = {textScale:number;highContrast:boolean;listOnly:boolean;announcements:boolean};
const key='agent-city:accessibility';
export function loadAccessibility():Accessibility {
 try {
  const value=JSON.parse(localStorage.getItem(key)||'{}');
  return {textScale:[100,115,130,150].includes(value.textScale)?value.textScale:100,highContrast:value.highContrast===true,listOnly:value.listOnly===true,announcements:value.announcements!==false};
 } catch {return {textScale:100,highContrast:false,listOnly:false,announcements:true}}
}
export function applyAccessibility(shell:HTMLElement,value:Accessibility){
 shell.style.setProperty('--text-scale',String(value.textScale/100));
 shell.classList.toggle('high-contrast',value.highContrast);
 shell.classList.toggle('list-only',value.listOnly);
 shell.querySelectorAll('[role=status]').forEach(element=>element.setAttribute('aria-live',value.announcements?'polite':'off'));
}
export function saveAccessibility(value:Accessibility){localStorage.setItem(key,JSON.stringify(value))}
