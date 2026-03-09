import { intervalToDuration, type Duration } from 'date-fns';
import { getAppLocale } from './locale';

type DateValue = string | number | Date;
type DurationUnit = 'year' | 'month' | 'day' | 'hour' | 'minute' | 'second';

type DurationOptions = {
  unitDisplay?: Intl.NumberFormatOptions['unitDisplay'];
  maxParts?: number;
  listStyle?: Intl.ListFormatStyle;
};

const dateTimeFormatterCache = new Map<string, Intl.DateTimeFormat>();
const numberFormatterCache = new Map<string, Intl.NumberFormat>();
const listFormatterCache = new Map<string, Intl.ListFormat>();

const durationUnits: Array<[keyof Duration, DurationUnit]> = [
  ['years', 'year'],
  ['months', 'month'],
  ['days', 'day'],
  ['hours', 'hour'],
  ['minutes', 'minute'],
  ['seconds', 'second'],
];

function getDateTimeFormatter(locale: string, options: Intl.DateTimeFormatOptions) {
  const cacheKey = `${locale}:${JSON.stringify(options)}`;
  const cachedFormatter = dateTimeFormatterCache.get(cacheKey);
  if (cachedFormatter) {
    return cachedFormatter;
  }

  const formatter = new Intl.DateTimeFormat(locale, options);
  dateTimeFormatterCache.set(cacheKey, formatter);
  return formatter;
}

function getNumberFormatter(locale: string, options: Intl.NumberFormatOptions = {}) {
  const cacheKey = `${locale}:${JSON.stringify(options)}`;
  const cachedFormatter = numberFormatterCache.get(cacheKey);
  if (cachedFormatter) {
    return cachedFormatter;
  }

  const formatter = new Intl.NumberFormat(locale, options);
  numberFormatterCache.set(cacheKey, formatter);
  return formatter;
}

function getListFormatter(locale: string, style: Intl.ListFormatStyle) {
  const cacheKey = `${locale}:${style}`;
  const cachedFormatter = listFormatterCache.get(cacheKey);
  if (cachedFormatter) {
    return cachedFormatter;
  }

  const formatter = new Intl.ListFormat(locale, { style, type: 'conjunction' });
  listFormatterCache.set(cacheKey, formatter);
  return formatter;
}

function normalizeDate(value: DateValue) {
  return value instanceof Date ? new Date(value) : new Date(value);
}

function isValidDate(date: Date) {
  return !Number.isNaN(date.getTime());
}

function normalizeDuration(durationInMs: number) {
  const duration = intervalToDuration({ start: 0, end: Math.max(0, durationInMs) });

  if (duration.seconds) {
    duration.minutes = (duration.minutes ?? 0) + 1;
    duration.seconds = 0;
  }

  if (duration.years) {
    duration.months = (duration.months ?? 0) + 1;
    duration.days = 0;
    duration.hours = 0;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.months) {
    duration.days = (duration.days ?? 0) + 1;
    duration.hours = 0;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.days) {
    duration.hours = (duration.hours ?? 0) + 1;
    duration.minutes = 0;
    duration.seconds = 0;
  } else if (duration.hours) {
    duration.minutes = (duration.minutes ?? 0) + 1;
  }

  return duration;
}

function formatUnit(value: number, unit: DurationUnit, unitDisplay: Intl.NumberFormatOptions['unitDisplay']) {
  return getNumberFormatter(getAppLocale(), {
    style: 'unit',
    unit,
    unitDisplay,
    maximumFractionDigits: 0,
  }).format(value);
}

export function parseDateTime(value: DateValue) {
  return normalizeDate(value);
}

export function formatDateValue(value: DateValue, options: Intl.DateTimeFormatOptions = { dateStyle: 'short' }) {
  const date = normalizeDate(value);
  if (!isValidDate(date)) {
    return '';
  }

  return getDateTimeFormatter(getAppLocale(), options).format(date);
}

export function formatDateTimeValue(
  value: DateValue,
  options: Intl.DateTimeFormatOptions = { dateStyle: 'short', timeStyle: 'short' },
) {
  return formatDateValue(value, options);
}

export function formatNumberValue(value?: number | bigint, options: Intl.NumberFormatOptions = {}) {
  if (value === null || value === undefined) {
    return '';
  }

  return getNumberFormatter(getAppLocale(), options).format(value);
}

export function formatDecimalValue(value?: number, fractionDigits = 1) {
  if (value === null || value === undefined) {
    return '';
  }

  return formatNumberValue(value, {
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  });
}

export function formatPercentValue(value?: number, fractionDigits = 2) {
  if (value === null || value === undefined) {
    return '';
  }

  return formatNumberValue(value / 100, {
    style: 'percent',
    minimumFractionDigits: fractionDigits,
    maximumFractionDigits: fractionDigits,
  });
}

export function formatDurationValue(durationInMs: number, options: DurationOptions = {}) {
  const locale = getAppLocale();
  const { unitDisplay = 'long', maxParts, listStyle = 'long' } = options;
  const duration = normalizeDuration(durationInMs);
  const parts = durationUnits
    .flatMap(([key, unit]) => {
      const value = duration[key] ?? 0;
      return value > 0 ? [formatUnit(value, unit, unitDisplay)] : [];
    });

  if (parts.length === 0) {
    return '';
  }

  const visibleParts = typeof maxParts === 'number' ? parts.slice(0, maxParts) : parts;
  return getListFormatter(locale, listStyle).format(visibleParts);
}
