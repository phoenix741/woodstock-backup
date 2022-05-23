import { format } from "date-fns";

export function formatDate(value?: number | null): string {
  if (value === null || value === undefined) return "";
  return format(value, "MM/dd/yyyy HH:mm");
}

export function formatAge(value?: number | null): string {
  if (value === null || value === undefined) return "";
  return (value / (24 * 3600000)).toFixed(2);
}

export function formatDuration(value?: number | null): string {
  if (value === null || value === undefined) return "";
  return (value / (1000 * 60)).toFixed(2);
}
