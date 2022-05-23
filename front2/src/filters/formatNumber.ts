import numeral from "numeral";

export function formatPercent(value: number) {
  if (value === null || value === undefined) return "";
  return numeral(value / 100).format("0.00 %");
}

export function formatNumber(value: number) {
  if (value === null || value === undefined) return "";
  return numeral(value).format("0,000");
}
