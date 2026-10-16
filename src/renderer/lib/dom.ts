export function isEditingElement(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target instanceof HTMLInputElement)
    return !/^(checkbox|radio|button|range|submit|reset|color|file)$/.test(target.type);
  return target.isContentEditable || /^(TEXTAREA|SELECT)$/.test(target.tagName);
}
