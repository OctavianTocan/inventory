// lib/bucket-fs.mjs driven by a list of operations, for the native directory bucket's differential tests.
import { Readable } from 'node:stream';
import { directoryBucket } from '../../../../../lib/bucket-fs.mjs';

async function apply(bucket, op) {
  switch (op.op) {
    case 'write': {
      const bytes = Buffer.from(op.body, 'base64');
      const body = op.stream ? Readable.toWeb(Readable.from([bytes.subarray(0, 3), bytes.subarray(3)])) : bytes;
      await bucket.write(op.key, body, op.length ?? undefined, op.headers ?? undefined);
      return null;
    }
    case 'head': {
      const info = await bucket.head(op.key);
      return info && { size: info.size, headers: info.headers ?? null };
    }
    case 'read': {
      const object = await bucket.read(op.key, op.range ?? undefined);
      if (!object) return null;
      const chunks = [];
      for await (const chunk of object.body) chunks.push(Buffer.from(chunk));
      return { size: object.size, headers: object.headers ?? null, body: Buffer.concat(chunks).toString('base64') };
    }
    case 'bytes': {
      const bytes = await bucket.bytes(op.key);
      return bytes && bytes.toString('base64');
    }
    case 'remove':
      await bucket.remove(op.key);
      return null;
    case 'list':
      return bucket.list(op.prefix);
  }
}

export async function run(root, ops) {
  const bucket = directoryBucket(root);
  const out = [];
  for (const op of ops) {
    try { out.push({ ok: await apply(bucket, op) }); }
    catch (error) { out.push({ error: { status: error.status ?? null, invalid: error instanceof TypeError || error instanceof RangeError } }); }
  }
  return out;
}
