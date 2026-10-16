import { createHash } from 'node:crypto';

// Write one unmasked server WebSocket frame, including large payload lengths.
function frame(socket, opcode, payload) {
  let head;
  if (payload.length < 126) head = Buffer.from([0x80 | opcode, payload.length]);
  else if (payload.length <= 65535) {
    head = Buffer.alloc(4);
    head[0] = 0x80 | opcode;
    head[1] = 126;
    head.writeUInt16BE(payload.length, 2);
  } else {
    head = Buffer.alloc(10);
    head[0] = 0x80 | opcode;
    head[1] = 127;
    head.writeBigUInt64BE(BigInt(payload.length), 2);
  }
  if (!socket.destroyed) socket.write(Buffer.concat([head, payload]));
}

/**
 * Attach a local SignalR JSON hub and return its committed-event broadcaster.
 * Only sockets that completed the SignalR handshake receive invocations.
 * @param {import('node:http').Server} server Replica HTTP server; owns upgraded sockets.
 * @param {(req:import('node:http').IncomingMessage) => boolean} [allows] Whether an upgrade request may connect.
 * @returns {(target:string,payload:object) => void} Broadcast one invocation to connected clients.
 */
export function createHub(server, allows = () => true) {
  const clients = new Set();
  server.on('upgrade', (req, socket, head) => {
    if (new URL(req.url, 'http://localhost').pathname !== '/hub' || !req.headers['sec-websocket-key'] || !allows(req)) return socket.destroy();
    const accept = createHash('sha1').update(`${req.headers['sec-websocket-key']}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`).digest('base64');
    socket.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`);
    let buffer = Buffer.alloc(0);
    let text = '';
    const ping = setInterval(() => frame(socket, 1, Buffer.from('{"type":6}\x1e')), 15000);
    const cleanup = () => { clients.delete(socket); clearInterval(ping); };
    socket.on('close', cleanup);
    socket.on('error', cleanup);
    socket.on('data', consume);
    if (head.length) consume(head);

    // Decode masked frames incrementally; SignalR records can span WebSocket messages.
    function consume(chunk) {
      buffer = Buffer.concat([buffer, chunk]);
      while (buffer.length >= 2) {
        const opcode = buffer[0] & 15;
        if (!(buffer[1] & 128)) return socket.destroy();
        let length = buffer[1] & 127;
        let offset = 2;
        if (length === 126) {
          if (buffer.length < 4) return;
          length = buffer.readUInt16BE(2);
          offset = 4;
        } else if (length === 127) {
          if (buffer.length < 10) return;
          const large = buffer.readBigUInt64BE(2);
          if (large > 1_048_576n) return socket.destroy();
          length = Number(large);
          offset = 10;
        }
        if (length > 1_048_576) return socket.destroy();
        if (buffer.length < offset + 4 + length) return;
        const mask = buffer.subarray(offset, offset + 4);
        offset += 4;
        const payload = Buffer.from(buffer.subarray(offset, offset + length)).map((byte, i) => byte ^ mask[i % 4]);
        buffer = buffer.subarray(offset + length);
        if (opcode === 8) { frame(socket, 8, payload); return socket.end(); }
        if (opcode === 9) { frame(socket, 10, payload); continue; }
        if (opcode !== 1 && opcode !== 0) continue;
        text += payload.toString('utf8');
        if (text.length > 1_048_576) return socket.destroy();
        let end;
        while ((end = text.indexOf('\x1e')) >= 0) {
          const record = text.slice(0, end);
          text = text.slice(end + 1);
          let message;
          try { message = JSON.parse(record); } catch { return socket.destroy(); }
          if (!clients.has(socket) && message.protocol === 'json' && message.version === 1) {
            frame(socket, 1, Buffer.from('{}\x1e'));
            clients.add(socket);
          }
        }
      }
    }
  });
  return (target, payload) => {
    const message = Buffer.from(JSON.stringify({ type: 1, target, arguments: [payload] }) + '\x1e');
    for (const socket of clients) frame(socket, 1, message);
  };
}
