import { HostsQuery } from "@/generated/graphql";
import formatDuration from "date-fns/formatDuration";
import intervalToDuration from "date-fns/intervalToDuration";
import format from "date-fns/format";
import numeral from "numeral";

export function getState(host: HostsQuery["hosts"][0]) {
  if (!host.configuration?.schedule?.activated) {
    return "disabled";
  } else if (host.lastBackupState) {
    return host.lastBackupState;
  } else if (host.lastBackup?.completed) {
    return "failed";
  } else {
    return "idle";
  }
}

export function getColor(state: string) {
  switch (state) {
    case "waiting":
      return "#B3E5FC";
    case "active":
      return "#0288D1";
    case "failed":
      return "#F4511E";
    case "completed":
      return "#43A047";
    case "delayed":
      return "#FDD835";
    case "disabled":
      return "#E0E0E0";
    case "idle":
      return "#BDBDBD";
  }
}

export function toDay(age: number) {
  return (age / (24 * 3600000)).toFixed(2);
}

export function toMinutes(age: number) {
  return formatDuration(intervalToDuration({ start: 0, end: age }));
}

export function toDateTime(value: number) {
  return format(value, "MM/dd/yyyy HH:mm");
}

export function toDate(value: number) {
  return format(value, "MM/dd/yyyy");
}

export function toPercent(value?: number) {
  if (value === null || value === undefined) return "";
  return numeral(value / 100).format("0.00%");
}

export function toNumber(value?: number) {
  if (value === null || value === undefined) return "";
  return numeral(value).format("0,000");
}
