import filesize from "filesize.js";

export function formatFilesize(value?: number | null) {
  if (value === null || value === undefined) return "";
  return filesize(value);
}
