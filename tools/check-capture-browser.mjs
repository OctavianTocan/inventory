import { strictEqual, ok } from 'node:assert';
import http from 'node:http';
import net from 'node:net';
import dgram from 'node:dgram';
import { once } from 'node:events';
import { chromium } from 'playwright-core';
import { screenshotPage } from '../lib/capture-link.mjs';

/**
 * Check screenshot browser isolation using a launch double and host-only local TCP/UDP probes.
 * @param {Function} check Parent assertion runner.
 * @param {object} fixtures Fixture handle providing allowOrigin for isolated localhost servers.
 * @param {Buffer} png Synthetic PNG returned by the browser double.
 * @param {boolean} inProcess Skip real sockets/Chromium when the sandbox forbids them.
 * @returns {Promise<void>} Policy assertions and available host probes have completed.
 */
export async function checkCaptureBrowser(check, fixtures, png, inProcess) {
  await check('A1/A2: screenshot launch denies native networking while HTTP routes share one budget', async () => {
    const original = chromium.launch;
    let policy, routeHandler, closed = false;
    const budgets = [];
    chromium.launch = async options => {
      policy = options;
      return { close: async () => { closed = true; }, newContext: async () => ({
        addInitScript: async () => {},
        routeWebSocket: async () => {},
        route: async (_pattern, handler) => { routeHandler = handler; },
        newPage: async () => ({
          goto: async () => {
            for (let i = 0; i < 2; i++) await routeHandler({
              request: () => ({ method: () => 'GET', url: () => 'http://127.0.0.1:45459/cover.png' }),
              fulfill: async () => {}, abort: async () => {},
            });
          },
          evaluate: async () => {}, screenshot: async () => png,
        }),
      }) };
    };
    try {
      strictEqual(await screenshotPage('http://127.0.0.1:45459/', { offline: true, resource: async (_url, options) => {
        budgets.push(options.budget);
        return { bytes: png, status: 200, headers: { 'content-type': 'image/png' } };
      } }), png);
      strictEqual(budgets.length, 2); strictEqual(budgets[0], budgets[1]);
      strictEqual(policy.proxy.server, 'http://capture-denied.invalid:9');
      for (const flag of ['--proxy-bypass-list=<-loopback>', '--disable-quic', '--force-webrtc-ip-handling-policy=disable_non_proxied_udp', '--host-resolver-rules=MAP * ~NOTFOUND']) ok(policy.args.includes(flag));
      ok(closed);
    } finally { chromium.launch = original; }
  });

  await check('A1: real Chromium preconnect/WebRTC probes cannot bypass screenshot transport', async () => {
    if (inProcess) return false;
    const sockets = new Set();
    let tcpConnections = 0, udpPackets = 0;
    const tcp = net.createServer(socket => { tcpConnections++; sockets.add(socket); socket.on('close', () => sockets.delete(socket)); socket.destroy(); });
    const udp = dgram.createSocket('udp4');
    udp.on('message', () => { udpPackets++; });
    tcp.listen(0, '127.0.0.1'); udp.bind(0, '127.0.0.1');
    await Promise.all([once(tcp, 'listening'), once(udp, 'listening')]);
    const tcpPort = tcp.address().port, udpPort = udp.address().port;
    const server = http.createServer((req, res) => {
      if (req.url === '/font') { setTimeout(() => { res.writeHead(404); res.end(); }, 1200); return; }
      res.writeHead(200, { 'content-type': 'text/html' });
      res.end(`<!doctype html><link rel="preconnect" crossorigin href="http://127.0.0.1:${tcpPort}">
        <style>@font-face{font-family:probe;src:url(/font)}body{font-family:probe}</style><p>Offline network probe</p>
        <script>
          window.peer = new RTCPeerConnection({iceServers:[{urls:'stun:127.0.0.1:${udpPort}'}]});
          peer.createDataChannel('probe'); peer.createOffer().then(offer=>peer.setLocalDescription(offer));
          if(window.WebTransport) new WebTransport('https://127.0.0.1:${udpPort}/').closed.catch(()=>{});
        </script>`);
    });
    server.listen(0, '127.0.0.1'); await once(server, 'listening');
    const url = `http://127.0.0.1:${server.address().port}/`;
    fixtures.allowOrigin(new URL(url).origin);
    let control;
    try {
      control = await chromium.launch({ executablePath: process.env.VERIFIER_CHROMIUM ?? '/usr/lib/chromium/chromium', headless: true });
      const page = await control.newPage();
      await page.goto(url);
      await page.waitForTimeout(1500);
      // Some Chromium builds never preconnect to loopback, so only the WebRTC control is mandatory.
      if (!tcpConnections) console.log('      note: this Chromium does not preconnect to loopback; TCP probe not exercised');
      ok(udpPackets > 0, 'unguarded WebRTC positive control must reach the UDP sink');
      await control.close(); control = null;
      tcpConnections = 0; udpPackets = 0;
      const screenshot = await screenshotPage(url, { offline: true });
      strictEqual(screenshot.subarray(1, 4).toString(), 'PNG');
      strictEqual(tcpConnections, 0); strictEqual(udpPackets, 0);
    } finally {
      await control?.close();
      for (const socket of sockets) socket.destroy();
      await new Promise(resolve => server.close(resolve));
      await new Promise(resolve => tcp.close(resolve));
      await new Promise(resolve => udp.close(resolve));
    }
  });
}
