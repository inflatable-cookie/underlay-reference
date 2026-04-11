import type { IconNodes } from "@poodle/svelte";

export const arrowUpDownIcon: IconNodes = [
  ["path", { d: "m21 16-4 4-4-4" }],
  ["path", { d: "M17 20V4" }],
  ["path", { d: "m3 8 4-4 4 4" }],
  ["path", { d: "M7 4v16" }]
];

export const squareCheckIcon: IconNodes = [
  ["rect", { width: "18", height: "18", x: "3", y: "3", rx: "2" }],
  ["path", { d: "m9 12 2 2 4-4" }]
];

export const trash2Icon: IconNodes = [
  ["path", { d: "M3 6h18" }],
  ["path", { d: "M8 6V4h8v2" }],
  ["path", { d: "M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" }],
  ["path", { d: "M10 11v6" }],
  ["path", { d: "M14 11v6" }]
];

export const uploadIcon: IconNodes = [
  ["path", { d: "M12 3v12" }],
  ["path", { d: "m7 8 5-5 5 5" }],
  ["path", { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }]
];

export const refreshCwIcon: IconNodes = [
  ["path", { d: "M21 12a9 9 0 0 0-15.5-6.4L3 8" }],
  ["path", { d: "M3 3v5h5" }],
  ["path", { d: "M3 12a9 9 0 0 0 15.5 6.4L21 16" }],
  ["path", { d: "M16 16h5v5" }]
];
