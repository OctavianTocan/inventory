// Headers carry ASCII only, so call arguments travel as base64url JSON.
const encodeArgs = (args) => Buffer.from(JSON.stringify(args)).toString('base64url');

// A body fetch accepts: bytes and web streams as they are, async iterables as a stream.
function requestBody(body) {
  if (body === undefined || body instanceof Uint8Array || body instanceof ReadableStream) return body;
  const iterator = body[Symbol.asyncIterator]();
  return new ReadableStream({
    async pull(controller) {
      const { done, value } = await iterator.next();
      if (done) controller.close();
      else controller.enqueue(value);
    },
  });
}

/**
 * The media container's operations for a library on Cloudflare, sent through the `Media` Durable Object: the media
 * processor `createHostedMedia` needs, and the link capture's page screenshots and validated HTTP fetches.
 * @param {object} env Worker environment with the `MEDIA` Durable Object binding.
 * @returns {{processor: {prepare: Function, rendition: Function, image: Function}, resource: Function,
 *   screenshot: Function}} Implementations with the signatures of `createProcessor` (`lib/media-processor.mjs`),
 *   `fetchResource` and `screenshotPage` (`lib/capture-link.mjs`); errors keep the `status` and `code` the
 *   container's error had.
 */
export function mediaClient(env) {
  async function call(operation, args, body) {
    const stub = env.MEDIA.get(env.MEDIA.idFromName('media'));
    const response = await stub.fetch(`http://media/${operation}`, { method: 'POST', headers: { 'x-media-args': encodeArgs(args) }, body: requestBody(body) });
    if (response.ok) return response;
    const failure = await response.json().catch(() => ({}));
    const error = new Error(failure.error ?? `Media container answered ${response.status}`);
    if (failure.status) error.status = failure.status;
    if (failure.code) error.code = failure.code;
    throw error;
  }
  const bytes = async (response) => Buffer.from(await response.arrayBuffer());
  return {
    processor: {
      prepare: async (body, type, name, user, id) => (await call('prepare', { type, name, user, id }, body)).json(),
      async rendition(original, relative) {
        const response = await call('rendition', { name: original.name, relative }, original.body);
        if (response.status === 204) return null;
        const body = await bytes(response);
        return { body, size: body.length };
      },
      image: async (body) => ({ mime: 'image/png', data: await bytes(await call('image', {}, body)) }),
    },
    async resource(url, { offline = false, limit } = {}) {
      const response = await call('resource', { url, offline, limit });
      return { ...JSON.parse(Buffer.from(response.headers.get('x-resource'), 'base64url').toString()), bytes: await bytes(response) };
    },
    screenshot: async (url, { offline = false } = {}) => bytes(await call('screenshot', { url, offline })),
  };
}
