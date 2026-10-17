// WHATWG URL parsing as Node does it, for mm_js::url's differential tests.
export function parse(input, base) {
  let url;
  try { url = base === null ? new URL(input) : new URL(input, base); } catch { return null; }
  return { href: url.href, origin: url.origin, protocol: url.protocol, host: url.host, hostname: url.hostname, port: url.port,
    pathname: url.pathname, search: url.search, hash: url.hash, params: [...url.searchParams] };
}
