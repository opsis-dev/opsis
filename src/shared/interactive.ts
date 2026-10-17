const INTERACTIVE_TAGS = ["button", "a", "input", "textarea", "select"];

const INTERACTIVE_ROLES = [
  "button",
  "link",
  "menuitem",
  "tab",
  "option",
  "checkbox",
  "switch",
  "combobox",
];

export const INTERACTIVE_SELECTOR = [
  ...INTERACTIVE_TAGS,
  ...INTERACTIVE_ROLES.map((role) => `[role=${role}]`),
].join(",");
