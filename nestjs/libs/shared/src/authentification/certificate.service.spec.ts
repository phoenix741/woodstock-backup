import { isIp } from './certificate.service';

describe('isIp', () => {
  // IPv4 Tests
  test('isIp returns true for valid IPv4 addresses', () => {
    expect(isIp('192.168.1.1')).toBe(true);
    expect(isIp('10.0.0.1')).toBe(true);
    expect(isIp('172.16.0.1')).toBe(true);
    expect(isIp('255.255.255.255')).toBe(true);
    expect(isIp('0.0.0.0')).toBe(true);
    expect(isIp('127.0.0.1')).toBe(true);
  });

  test('isIp returns false for invalid IPv4 addresses', () => {
    expect(isIp('256.0.0.1')).toBe(false);
    expect(isIp('192.168.1')).toBe(false);
    expect(isIp('192.168.1.1.5')).toBe(false);
    expect(isIp('192.168..1')).toBe(false);
    expect(isIp('192.168.1.')).toBe(false);
    expect(isIp('.192.168.1.1')).toBe(false);
    expect(isIp('a.b.c.d')).toBe(false);
    expect(isIp('-1.2.3.4')).toBe(false);
  });

  // IPv6 Tests
  test('isIp returns true for valid IPv6 addresses', () => {
    expect(isIp('2001:0db8:85a3:0000:0000:8a2e:0370:7334')).toBe(true);
    expect(isIp('::1')).toBe(true);
    expect(isIp('fe80::1ff:fe23:4567:890a')).toBe(true);
    expect(isIp('2001:db8::ff00:42:8329')).toBe(true);
    expect(isIp('::ffff:192.168.1.1')).toBe(true);
  });

  test('isIp returns false for invalid IPv6 addresses', () => {
    expect(isIp('2001:0db8:85a3:0000:0000:8a2e:0370:7334:')).toBe(false);
    expect(isIp('2001:db8::ff00:42:8329::')).toBe(false);
    expect(isIp('::ffff:192.168.1.256')).toBe(false);
    expect(isIp('2001:db8:::1')).toBe(false);
    expect(isIp('2001:db8::g')).toBe(false);
  });

  // Hostname Tests
  test('isIp returns false for hostnames', () => {
    expect(isIp('localhost')).toBe(false);
    expect(isIp('example.com')).toBe(false);
    expect(isIp('test.woodstock.org')).toBe(false);
    expect(isIp('sub.domain.example.co.uk')).toBe(false);
    expect(isIp('xn--bcher-kva.example')).toBe(false); // IDN (bücher.example)
    expect(isIp('server-01')).toBe(false);
    expect(isIp('www.example.com.')).toBe(false);
  });

  // Special Cases
  test('isIp returns false for special cases', () => {
    expect(isIp('')).toBe(false);
    expect(isIp('   ')).toBe(false);
    expect(isIp('null')).toBe(false);
    expect(isIp('undefined')).toBe(false);
    expect(isIp('192.168.1.1/24')).toBe(false); // CIDR notation
    expect(isIp('http://192.168.1.1')).toBe(false);
    expect(isIp('https://example.com')).toBe(false);
  });
});
