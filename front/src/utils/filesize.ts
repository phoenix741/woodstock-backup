/**
 * MIT License
 *
 * Copyright (c) 2019 hustcc
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

// Fork of https://github.com/hustcc/filesize.js

type SPEC = {
  readonly radix: number;
  readonly unit: string[];
};

const si = { radix: 1e3, unit: ['b', 'kb', 'Mb', 'Gb', 'Tb', 'Pb', 'Eb', 'Zb', 'Yb'] };
const iec = { radix: 1024, unit: ['b', 'Kib', 'Mib', 'Gib', 'Tib', 'Pib', 'Eib', 'Zib', 'Yib'] };
const jedec = { radix: 1024, unit: ['b', 'Kb', 'Mb', 'Gb', 'Tb', 'Pb', 'Eb', 'Zb', 'Yb'] };

export const SPECS: Record<string, SPEC> = {
  si,
  iec,
  jedec,
};

const abs = (n: number | bigint) => (n < 0n ? -n : n);

/**
 * Divide the bytes by the radix until it is less than the radix.
 * If the bytes is less than Number.MAX_SAFE_INTEGER, convert the bigint to number.
 */
function reduce(bytes: number | bigint, radix: number) {
  bytes = abs(bytes);
  let loop = 0;

  while (bytes >= radix) {
    if (bytes < Number.MAX_SAFE_INTEGER) {
      bytes = Number(bytes);
    }

    if (typeof bytes === 'number') {
      bytes /= radix;
    } else {
      bytes /= BigInt(radix);
    }
    ++loop;
  }

  return { bytes: Number(bytes), loop };
}

export default function (bytes: number | bigint | string, fixed = 1, spec?: string): string {
  if (typeof bytes === 'string') {
    bytes = BigInt(bytes);
  }

  const { radix, unit } = SPECS[spec ?? 'jedec'];

  const { bytes: reducedBytes, loop } = reduce(bytes, radix);

  return `${reducedBytes.toFixed(fixed)} ${unit[loop]}`;
}
