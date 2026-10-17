// Drives lib/hub.mjs's createHub with fake upgraded sockets for mm-core's differential tests, answering the bytes
// each step wrote, ended or destroyed.
import { EventEmitter } from 'node:events';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const { createHub } = await import(pathToFileURL(join(process.env.MYMIND_ROOT, 'lib', 'hub.mjs')).href);

/** Run `steps` against one hub; each answer lists `[socket, 'write' | 'end' | 'destroy', hex?]` events of that step. */
export function run(steps) {
  const server = new EventEmitter();
  let mode = 'all';
  const allows = (req) => {
    if (mode === 'all') return true;
    if (mode === 'none') return false;
    if (mode === 'throw') throw new Error('boom');
    return mode.includes(req.headers['x-user']);
  };
  const broadcast = createHub(server, allows);
  const sockets = new Map();
  const intervals = new Map();
  let events = [];
  const realSetInterval = globalThis.setInterval;
  const realClearInterval = globalThis.clearInterval;
  let created = [];
  globalThis.setInterval = (callback) => { created.push(callback); return { callback }; };
  globalThis.clearInterval = (handle) => { for (const [id, callback] of intervals) if (callback === handle?.callback) intervals.delete(id); };
  const answers = [];
  try {
    for (const step of steps) {
      events = [];
      if (step.op === 'upgrade') {
        const id = step.id;
        const socket = new EventEmitter();
        socket.destroyed = false;
        socket.write = (bytes) => events.push([id, 'write', Buffer.from(bytes).toString('hex')]);
        socket.end = () => events.push([id, 'end']);
        socket.destroy = () => { socket.destroyed = true; events.push([id, 'destroy']); };
        sockets.set(id, socket);
        created = [];
        server.emit('upgrade', { url: step.path, headers: step.headers }, socket, Buffer.from(step.head, 'hex'));
        if (created.length) intervals.set(id, created[0]);
      } else if (step.op === 'data') sockets.get(step.id).emit('data', Buffer.from(step.hex, 'hex'));
      else if (step.op === 'ping') intervals.get(step.id)?.();
      else if (step.op === 'allow') mode = step.mode;
      else if (step.op === 'broadcast') broadcast(step.target, step.payload);
      else if (step.op === 'close') sockets.get(step.id).emit('close');
      answers.push({ op: step.op, events });
    }
  } finally {
    globalThis.setInterval = realSetInterval;
    globalThis.clearInterval = realClearInterval;
  }
  return answers;
}
