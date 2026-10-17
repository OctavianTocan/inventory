// Answers mm-core's differential tests with the JS backend. Reads one JSON request on stdin,
// {module, calls: [{fn, args}]}, where `module` is relative to the repository root, and prints one answer per call:
// {ok: value} or {error: {name, message, status, code}}. Values the JSON cannot hold are tagged: {$undefined: true},
// {$bigint: "digits"}, {$bytes: "base64"}, {$date: ms}.
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = process.env.MYMIND_ROOT ?? path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');

const tag = (value) => {
  if (value === undefined) return { $undefined: true };
  if (typeof value === 'bigint') return { $bigint: String(value) };
  if (value instanceof Uint8Array) return { $bytes: Buffer.from(value).toString('base64') };
  if (value instanceof ArrayBuffer) return { $bytes: Buffer.from(value).toString('base64') };
  if (value instanceof Date) return { $date: value.getTime() };
  if (Array.isArray(value)) return value.map(tag);
  if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, tag(item)]));
  return value;
};

const untag = (value) => {
  if (Array.isArray(value)) return value.map(untag);
  if (!value || typeof value !== 'object') return value;
  if (value.$undefined) return undefined;
  if ('$bigint' in value) return BigInt(value.$bigint);
  if ('$bytes' in value) return new Uint8Array(Buffer.from(value.$bytes, 'base64'));
  if ('$date' in value) return new Date(value.$date);
  return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, untag(item)]));
};

const request = JSON.parse(readFileSync(0, 'utf8'));
const module = await import(pathToFileURL(path.resolve(ROOT, request.module)).href);
const answers = [];
for (const call of request.calls) {
  try {
    answers.push({ ok: tag(await module[call.fn](...untag(call.args))) });
  } catch (error) {
    answers.push({ error: { name: error?.constructor?.name ?? 'Error', message: String(error?.message ?? error), status: error?.status ?? null, code: error?.code ?? null } });
  }
}
process.stdout.write(JSON.stringify(answers));
