import path from 'node:path';
import { pathToFileURL } from 'node:url';

const { MIME } = await import(pathToFileURL(path.join(process.env.MYMIND_ROOT, 'lib/media-files.mjs')).href);

/** The content-type table in declaration order. */
export function mimeTable() {
  return Object.entries(MIME);
}
