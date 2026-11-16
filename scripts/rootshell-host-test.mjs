// Run the *actual pinned rootshell Worker*, with a local filesystem/socket
// broker matching its documented Swift protocol. This is a portability gate,
// not a claim of a real-iPhone installation test.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import net from 'node:net';
import tls from 'node:tls';
import { EventEmitter } from 'node:events';
import { Worker } from 'node:worker_threads';
import { createHash } from 'node:crypto';
import { gzipSync } from 'node:zlib';
import { spawnSync } from 'node:child_process';
import assert from 'node:assert/strict';

export const HOST_REV = 'f5adabe57ddb2959922e135e622fb9776641735f';
const ERRNO = { ENOENT: 44, EEXIST: 20, EACCES: 2, EPERM: 2, ENOTDIR: 54, EISDIR: 31, ENOTEMPTY: 55, ECONNREFUSED: 14, ETIMEDOUT: 73 };

export async function runWasm(workerSource, wasm, argv, { root, input = '', env = {}, timeout = 60000, failLogWritesAfter = Infinity } = {}) {
  root ??= fs.mkdtempSync(path.join(os.tmpdir(), 'siboot-rootshell-'));
  const handles = new Map([[3, { path: root, preopen: '.' }], [4, { path: root, preopen: '/' }]]);
  const sockets = new Map();
  let next = 5, nextSocket = 10, raw = false, stdout = '', stderr = '', calls = 0, logWrites = 0;
  let inputBytes = Buffer.from(input), inputOffset = 0;
  const rawModes = [];
  const resolve = (p, fd = 3) => {
    const base = handles.get(fd)?.path;
    if (!base) throw Object.assign(new Error('bad fd'), { code: 'EBADF' });
    const resolved = path.resolve(p.startsWith('/') ? root : base, p.replace(/^\/+/, ''));
    if (resolved !== root && !resolved.startsWith(root + path.sep)) throw Object.assign(new Error('sandbox escape'), { code: 'EACCES' });
    return resolved;
  };
  const stat = p => { const s = fs.statSync(p); return { type: s.isDirectory() ? 3 : 4, size: s.size, mtime: Math.floor(s.mtimeMs * 1e6) }; };
  const fileCall = p => {
    const h = handles.get(p.fd);
    switch (p.op) {
      case 'fd_prestat_get': if (!h?.preopen) return { errno: 8 }; return { type: 0, nameLen: h.preopen.length };
      case 'fd_prestat_dir_name': if (!h?.preopen) return { errno: 8 }; return { name: h.preopen };
      case 'path_open': {
        const name = resolve(p.path, p.dirfd);
        const exists = fs.existsSync(name);
        if (!exists && !(p.oflags & 1)) return { errno: 44 };
        if (exists && (p.oflags & 1) && (p.oflags & 4)) return { errno: 20 };
        if (!exists) fs.writeFileSync(name, '');
        if (p.oflags & 8) fs.truncateSync(name, 0);
        const directory = fs.statSync(name).isDirectory();
        const fd = next++;
        handles.set(fd, { path: name, native: directory ? null : fs.openSync(name, 'r+'), offset: p.fdflags & 1 ? fs.statSync(name).size : 0 });
        return { fd };
      }
      case 'fd_close': if (!h) return { errno: 8 }; if (h.native != null) fs.closeSync(h.native); handles.delete(p.fd); return {};
      case 'fd_read': case 'fd_pread': {
        if (!h || h.native == null) return { errno: 8 };
        const buffer = Buffer.alloc(Math.min(p.length, 1024 * 1024));
        const count = fs.readSync(h.native, buffer, 0, buffer.length, p.op === 'fd_pread' ? p.offset : h.offset);
        if (p.op === 'fd_read') h.offset += count;
        return { data: buffer.subarray(0, count).toString('base64') };
      }
      case 'fd_write': case 'fd_pwrite': {
        if (!h || h.native == null) return { errno: 8 };
        if (h.path.endsWith('.jsonl') && logWrites++ >= failLogWritesAfter) return { errno: 29 };
        const buffer = Buffer.from(p.data, 'base64');
        const count = fs.writeSync(h.native, buffer, 0, buffer.length, p.op === 'fd_pwrite' ? p.offset : h.offset);
        if (p.op === 'fd_write') h.offset += count;
        return { written: count };
      }
      case 'fd_seek': if (!h) return { errno: 8 }; h.offset = p.offset + (p.whence === 1 ? h.offset : p.whence === 2 ? fs.statSync(h.path).size : 0); return { offset: h.offset };
      case 'fd_sync': fs.fsyncSync(h.native); return {};
      case 'fd_filestat_get': return h ? stat(h.path) : { errno: 8 };
      case 'path_filestat_get': return stat(resolve(p.path, p.dirfd));
      case 'path_create_directory': fs.mkdirSync(resolve(p.path, p.dirfd)); return {};
      case 'path_remove_directory': fs.rmdirSync(resolve(p.path, p.dirfd)); return {};
      case 'path_unlink_file': fs.unlinkSync(resolve(p.path, p.dirfd)); return {};
      case 'path_rename': fs.renameSync(resolve(p.oldPath, p.oldDirfd), resolve(p.newPath, p.newDirfd)); return {};
      case 'fd_filestat_set_size': fs.ftruncateSync(h.native, p.size); return {};
      case 'fd_filestat_set_times': case 'path_filestat_set_times': return {};
      case 'fd_readdir': return { entries: fs.readdirSync(h.path, { withFileTypes: true }).slice(p.cookie).map(e => ({ name: e.name, isDir: e.isDirectory() })) };
      default: throw new Error(`untested filesystem operation ${p.op}`);
    }
  };
  const socketCall = async p => {
    const s = sockets.get(p.fd);
    switch (p.op) {
      case 'socket': { const fd = nextSocket++; sockets.set(fd, { timeout: 30000, buffer: Buffer.alloc(0), ended: false, changed: new EventEmitter() }); return { fd }; }
      case 'connect': case 'tls_connect': {
        assert(s);
        // Tests never contact an Apple Account or an external service.
        assert(['127.0.0.1', 'localhost', '::1'].includes(p.host), `unexpected test host ${p.host}`);
        s.stream = p.op === 'tls_connect' ? tls.connect({ host: p.host, port: p.port, rejectUnauthorized: true }) : net.connect({ host: p.host, port: p.port });
        s.stream.on('data', data => { s.buffer = Buffer.concat([s.buffer, data]); s.changed.emit('change'); });
        s.stream.on('end', () => { s.ended = true; s.changed.emit('change'); });
        s.stream.on('error', error => { s.error = ERRNO[error.code] ?? 29; s.changed.emit('change'); });
        await new Promise((resolve, reject) => { s.stream.once(p.op === 'tls_connect' ? 'secureConnect' : 'connect', resolve); s.stream.once('error', reject); });
        return {};
      }
      case 'set_recv_timeout': s.timeout = p.timeoutMs; return {};
      case 'send': {
        const bytes = Buffer.from(p.data, 'base64');
        const count = Math.min(bytes.length, 997); // Deliberate short writes.
        await new Promise((resolve, reject) => s.stream.write(bytes.subarray(0, count), e => e ? reject(e) : resolve()));
        return { sent: count };
      }
      case 'recv': {
        if (!s.buffer.length && !s.ended && !s.error) {
          await new Promise(resolve => { const done = () => { clearTimeout(timer); s.changed.removeListener('change', done); resolve(); }; const timer = setTimeout(done, s.timeout || 30000); s.changed.once('change', done); });
        }
        if (s.error) return { errno: s.error };
        if (!s.buffer.length) return s.ended ? { data: '' } : { errno: 6 };
        const bytes = s.buffer.subarray(0, Math.min(p.maxLen, 4093));
        s.buffer = s.buffer.subarray(bytes.length);
        return { data: bytes.toString('base64') };
      }
      case 'close': s?.stream?.destroy(); sockets.delete(p.fd); return {};
      default: throw new Error(`untested socket operation ${p.op}`);
    }
  };
  const sab = new SharedArrayBuffer(8 * 1024 * 1024), view = new Int32Array(sab);
  const bootstrap = `const { parentPort, workerData } = require('node:worker_threads'); globalThis.self=globalThis; globalThis.crypto=require('node:crypto').webcrypto; self.postMessage=m=>parentPort.postMessage(m); parentPort.on('message', data=>self.onmessage({data})); require('node:vm').runInThisContext(workerData.source);`;
  const worker = new Worker(bootstrap, { eval: true, workerData: { source: workerSource } });
  const reply = (id, result) => {
    const bytes = Buffer.from(JSON.stringify({ errno: 0, ...result, callID: id }));
    assert(bytes.length < sab.byteLength - 8);
    new Uint8Array(sab, 8, bytes.length).set(bytes);
    Atomics.store(view, 1, bytes.length); Atomics.store(view, 0, 1); Atomics.notify(view, 0);
  };
  return new Promise((resolve, reject) => {
    const finish = async (error, code) => {
      clearTimeout(timer);
      await worker.terminate();
      for (const h of handles.values()) if (h.native != null) fs.closeSync(h.native);
      for (const s of sockets.values()) s.stream?.destroy();
      error ? reject(error) : resolve({ code, stdout, stderr, rawModes, raw, calls, root });
    };
    const timer = setTimeout(() => finish(new Error(`rootshell WASM timed out: ${stdout.slice(-1500)} ${stderr}`)), timeout);
    worker.on('error', error => finish(error));
    worker.on('message', async message => {
      calls++;
      try {
        switch (message.kind) {
          case 'stdout': stdout += Buffer.from(message.data, 'base64'); break;
          case 'stderr': stderr += Buffer.from(message.data, 'base64'); break;
          case 'exit': await finish(null, message.code); break;
          case 'error': await finish(new Error(`${message.message}\n${stdout}\n${stderr}`)); break;
          case 'fs': reply(message.callID, fileCall(message.payload)); break;
          case 'socket': reply(message.callID, await socketCall(message.payload)); break;
          case 'terminal': raw = Boolean(message.payload.enabled); rawModes.push(raw); reply(message.callID, {}); break;
          case 'stdinRead': { const bytes = inputBytes.subarray(inputOffset, inputOffset + message.maxLen); inputOffset += bytes.length; reply(message.callID, { data: bytes.toString('base64') }); break; }
          default: throw new Error(`unexpected Worker message: ${message.kind}`);
        }
      } catch (error) {
        if (error.code) reply(message.callID, { errno: ERRNO[error.code] ?? 29 });
        else await finish(error);
      }
    });
    worker.postMessage({ kind: 'start', processID: 'siboot-test', sab, wasm: fs.readFileSync(wasm).toString('base64'), argv, env: { HOME: '/', PWD: '/', USER: 'mobile', TERM: 'dumb', ...env }, cwd: '/', sandboxRoot: '/' });
  });
}

function performanceLog(result, directory = result.root) {
  const logs = fs.readdirSync(directory).filter(name => /^siboot-perf-.*\.jsonl$/.test(name));
  assert.equal(logs.length, 1, `expected one performance log in ${directory}`);
  const text = fs.readFileSync(path.join(directory, logs[0]), 'utf8');
  const records = text.trim().split('\n').map(line => JSON.parse(line));
  assert.equal(records[0].event, 'run_start');
  assert.equal(records[0].clock_resolution_us, 1000);
  assert.equal(records.at(-1).event, 'summary');
  for (let index = 0; index < records.length; index++) {
    assert.equal(records[index].schema, 1);
    assert(Number.isInteger(records[index].elapsed_us));
    if (index) assert(records[index].seq > records[index - 1].seq);
    if (records[index].linear_memory_bytes != null) assert(records[index].linear_memory_bytes >= 65536);
  }
  for (const secret of ['test-only-password', 'different local passphrase', 'wrong local passphrase', 'offline@example.invalid', ' 空 格pass ', 'never-log-this-secret', 'query-secret-never-record', 'header-secret-never-record', 'body-secret-never-record', 'identifier-secret-never-record', 'missing-secret.ipa', 'private-account@example.invalid']) {
    assert(!text.includes(secret), `sensitive test input appeared in log: ${secret}`);
  }
  assert(!text.includes(result.root), 'log must not contain the sandbox path');
  return { records, text, summary: records.at(-1) };
}

if (process.argv[1] === new URL(import.meta.url).pathname) {
  const [workerPath, wasm, probe] = process.argv.slice(2);
  assert(workerPath && wasm, 'usage: node rootshell-host-test.mjs WASM_WORKER_JS SIBOOT_WASM');
  const source = fs.readFileSync(workerPath, 'utf8');
  const module = new WebAssembly.Module(fs.readFileSync(wasm));
  for (const imp of WebAssembly.Module.imports(module)) {
    assert(['wasi_snapshot_preview1', 'rootshell_socket', 'rootshell_terminal'].includes(imp.module), `unexpected import ${imp.module}.${imp.name}`);
    assert(!/thread|pthread|sock_/.test(imp.name), `unported import ${imp.name}`);
  }
  for (const arg of ['--version', '--help']) {
    const result = await runWasm(source, wasm, ['./siboot.wasm', arg]);
    assert.equal(result.code, 0, result.stderr);
    const version = JSON.parse(fs.readFileSync(new URL('../.github/siboot-release.json', import.meta.url), 'utf8')).version;
    assert(result.stdout.includes('siboot ' + version), result.stdout);
    assert.match(result.stdout, /Jay-Void-BA/);
    assert.match(result.stdout, /FrizzleM\/siboot/);
    assert(!result.stderr.includes('unhandled'));
    console.log(`PASS rootshell Worker ${arg}`);
  }
  {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'siboot-location-'));
    fs.mkdirSync(path.join(root, 'bin'));
    fs.copyFileSync(wasm, path.join(root, 'bin/siboot.wasm'));
    const result = await runWasm(source, wasm, ['./bin/siboot.wasm', '--ipa', '/missing.ipa'], { root });
    assert.equal(result.code, 1, result.stdout + result.stderr);
    assert(fs.existsSync(path.join(root, 'bin/.siboot/tmp')), 'storage must be beside the WASM, even in a subdirectory');
    assert(!fs.existsSync(path.join(root, '.siboot')));
    const bare = await runWasm(source, wasm, ['siboot.wasm', '--ipa', '/missing.ipa'], { root });
    assert.equal(bare.code, 1, bare.stdout + bare.stderr);
    assert(!fs.existsSync(path.join(root, '.siboot')), 'bare module names must use rootshell HOME/bin lookup');
    assert(!fs.readdirSync(path.join(root, 'bin')).some(name => name.endsWith('.jsonl')), 'logging must default to off');
    const logged = await runWasm(source, wasm, ['./bin/siboot.wasm', '--perf-log', '--home', '/other-state', '--apple-id', 'private-account@example.invalid', '--ipa', '/missing-secret.ipa'], { root });
    assert.equal(logged.code, 1, logged.stdout + logged.stderr);
    const log = performanceLog(logged, path.join(root, 'bin'));
    assert.equal(log.summary.outcome, 'error');
    assert(!fs.readdirSync(root).some(name => name.endsWith('.jsonl')), 'log must stay beside the module even with --home');
    await runWasm(source, wasm, ['./bin/siboot.wasm', '--perf-log', '--ipa', '/missing.ipa'], { root });
    assert.equal(fs.readdirSync(path.join(root, 'bin')).filter(name => name.endsWith('.jsonl')).length, 2);
    console.log('PASS rootshell Worker storage beside the executable and invalid IPA rejection before login');
  }
  const english = await runWasm(source, wasm, ['./siboot.wasm', '--lang', 'en', '--help']);
  assert.equal(english.code, 0, english.stderr);
  assert(english.stdout.includes('Passwords are hidden. Ctrl-V toggles viewing while typing.'));
  const cancelRun = await runWasm(source, wasm, ['./siboot.wasm', '--perf-log']);
  assert.equal(cancelRun.code, 1);
  assert.equal(performanceLog(cancelRun).summary.outcome, 'cancelled');
  const cannotLog = await runWasm(source, wasm, ['./siboot.wasm', '--perf-log', '--lang', 'en'], { failLogWritesAfter: 0 });
  assert.equal(cannotLog.code, 1);
  assert(cannotLog.stderr.includes("Couldn't create the performance log."));
  console.log('PASS rootshell Worker opt-in performance logs, unique filenames, failure/cancellation summaries and storage errors');
  if (probe) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'siboot-offline-'));
    const fixture = spawnSync('python3', [new URL('./make_rootshell_fixture.py', import.meta.url).pathname, root], { encoding: 'utf8' });
    assert.equal(fixture.status, 0, fixture.stderr);
    const offline = await runWasm(source, probe, ['./runtime_check.wasm', '--perf-log'], { root, timeout: 180000 });
    assert.equal(offline.code, 0, offline.stdout + offline.stderr);
    for (const marker of ['VAULT_OK', 'CSR_OK', 'COMPLETE_SIGNING_OK', 'INVALID_LINKS_REJECTED', 'RENEW_PAIRING_OK']) assert(offline.stdout.includes(marker), offline.stdout);
    const offlineLog = performanceLog(offline);
    for (const operation of ['vault.scrypt', 'sign.csr', 'ipa.extract', 'sign.macho', 'sign.macho.encode', 'sign.resource', 'sign.resource.hash', 'sign.verify_seals']) {
      assert(offlineLog.records.some(record => record.operation === operation || record.metrics?.[operation]), `missing performance measurement: ${operation}`);
    }
    console.log('PASS rootshell Worker encrypted vault, PKCS#10, complete CMS signing, symlinks, renewal and pairing storage');
    const password = await runWasm(source, probe, ['./runtime_check.wasm', 'password', '--perf-log'], { input: ' 空 格中\x7fpass \x16\x16\r' });
    assert.equal(password.code, 0, password.stdout + password.stderr);
    assert(password.stdout.includes('PASSWORD_OK'));
    assert.deepEqual(password.rawModes, [true, false]);
    assert.equal(password.raw, false);
    assert(performanceLog(password).records.some(record => record.operation === 'ui.wait'));
    console.log('PASS rootshell Worker password reveal/hide and Unicode/whitespace preservation');
    const cancelled = await runWasm(source, probe, ['./runtime_check.wasm', 'password-cancel', '--perf-log'], { input: 'never-log-this-secret\x03' });
    assert.equal(cancelled.code, 0, cancelled.stderr);
    assert(cancelled.stdout.includes('PASSWORD_CANCEL_OK'));
    assert(!cancelled.stdout.includes('never-log-this-secret'));
    assert.deepEqual(cancelled.rawModes, [true, false]);
    assert(performanceLog(cancelled).records.some(record => record.operation === 'ui.wait' && record.outcome === 'cancelled'));
    const untrusted = tls.createServer({ key: fs.readFileSync(path.join(root, 'test-key.pem')), cert: fs.readFileSync(path.join(root, 'test-cert.pem')) });
    untrusted.on('tlsClientError', () => {});
    await new Promise(resolve => untrusted.listen(0, '127.0.0.1', resolve));
    try {
      const result = await runWasm(source, probe, ['./runtime_check.wasm', 'tls-reject', String(untrusted.address().port)]);
      assert.equal(result.code, 0, result.stdout + result.stderr);
      assert(result.stdout.includes('UNTRUSTED_TLS_REJECTED'));
      console.log('PASS rootshell Worker TLS rejects an untrusted certificate');
    } finally { untrusted.close(); }

    const server = net.createServer(connection => {
      let pending = Buffer.alloc(0), upgraded = false, ipMode = false;
      const streams = new Map();
      const checksum = buffer => { let sum = 0; for (let i = 0; i < buffer.length; i += 2) sum += (buffer[i] << 8) + (buffer[i + 1] || 0); while (sum >>> 16) sum = (sum & 65535) + (sum >>> 16); return (~sum) & 65535; };
      const tcpReply = (request, seq, ack, flags, payload = Buffer.alloc(0)) => {
        const ihl = (request[0] & 15) * 4;
        const tcp = Buffer.alloc(20 + payload.length);
        tcp.writeUInt16BE(request.readUInt16BE(ihl + 2), 0); tcp.writeUInt16BE(request.readUInt16BE(ihl), 2);
        tcp.writeUInt32BE(seq >>> 0, 4); tcp.writeUInt32BE(ack >>> 0, 8); tcp[12] = 0x50; tcp[13] = flags; tcp.writeUInt16BE(65535, 14); payload.copy(tcp, 20);
        const pseudo = Buffer.concat([request.subarray(16, 20), request.subarray(12, 16), Buffer.from([0, 6, tcp.length >>> 8, tcp.length & 255]), tcp]);
        tcp.writeUInt16BE(checksum(pseudo), 16);
        const ip = Buffer.alloc(20); ip[0] = 0x45; ip.writeUInt16BE(20 + tcp.length, 2); ip[8] = 64; ip[9] = 6;
        request.copy(ip, 12, 16, 20); request.copy(ip, 16, 12, 16); ip.writeUInt16BE(checksum(ip), 10);
        connection.write(Buffer.concat([ip, tcp]));
      };
      connection.on('error', () => {});
      connection.on('data', data => {
        pending = Buffer.concat([pending, data]);
        if (pending.subarray(0, 4).toString() === 'IP4!') { pending = pending.subarray(4); ipMode = true; }
        if (ipMode) {
          while (pending.length >= 20) {
            const length = pending.readUInt16BE(2); if (pending.length < length) return;
            const packet = pending.subarray(0, length); pending = pending.subarray(length);
            const ihl = (packet[0] & 15) * 4, offset = ihl + ((packet[ihl + 12] >>> 4) * 4), flags = packet[ihl + 13];
            const port = packet.readUInt16BE(ihl), seq = packet.readUInt32BE(ihl + 4);
            if (flags & 2) { streams.set(port, 10001); tcpReply(packet, 10000, seq + 1, 0x12); }
            else if (packet.length > offset) { const payload = packet.subarray(offset); tcpReply(packet, streams.get(port), seq + payload.length, 0x18, payload); streams.set(port, (streams.get(port) + payload.length) >>> 0); }
          }
          return;
        }
        if (pending.subarray(0, 4).toString() === 'ECHO') {
          pending = Buffer.alloc(0); setTimeout(() => connection.end('ECHO'), 80); return;
        }
        if (!upgraded) {
          const end = pending.indexOf('\r\n\r\n'); if (end < 0) return;
          const head = pending.subarray(0, end).toString(); pending = pending.subarray(end + 4);
          if (head.startsWith('GET /bag?')) {
            const body = gzipSync(Buffer.from('{"ok":true,"token":"body-secret-never-record"}'));
            connection.write('HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Encoding: gzip\r\n\r\n');
            connection.write(body.length.toString(16) + '\r\n'); connection.write(body); connection.end('\r\n0\r\n\r\n'); return;
          }
          assert(head.startsWith('GET /provision '), head);
          const key = /sec-websocket-key:\s*(.+)/i.exec(head)[1].trim();
          const accept = createHash('sha1').update(key + '258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
          connection.write(`HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ${accept}\r\n\r\n`); upgraded = true;
        }
        if (upgraded && pending.length >= 6) {
          const opcode = pending[0] & 15, length = pending[1] & 127;
          if (opcode === 8) { connection.end(Buffer.from([0x88, 0])); return; }
          assert(length < 126 && pending[1] & 128);
          if (pending.length < 6 + length) return;
          const mask = pending.subarray(2, 6);
          const body = pending.subarray(6, 6 + length).map((b, i) => b ^ mask[i % 4]);
          assert.equal(body.toString(), 'identifier-secret-never-record'); pending = pending.subarray(6 + length);
          connection.write(Buffer.concat([Buffer.from([0x81, 11]), Buffer.from('provisioned')]));
        }
      });
    });
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    try {
      const network = await runWasm(source, probe, ['./runtime_check.wasm', 'network', String(server.address().port), '--perf-log']);
      assert.equal(network.code, 0, network.stdout + network.stderr);
      assert(network.stdout.includes('HTTP_WEBSOCKET_COOPERATIVE_TCP_OK'), network.stdout);
      assert(network.stdout.includes('SOFTWARE_TCP_STACK_OK'), network.stdout);
      const networkLog = performanceLog(network);
      const receive = networkLog.summary.metrics['socket.recv'];
      assert(receive.bytes >= 65536 && receive.calls > 0);
      assert(receive.idle_timeouts > 0 && receive.errors > 0, 'cooperative idle polls and blocking timeouts must be distinct');
      assert(networkLog.records.some(record => record.operation === 'http.request' && record.data?.status === 200));
      assert(networkLog.records.some(record => record.operation === 'websocket.connect'));
      const logFailure = await runWasm(source, wasm, ['./siboot.wasm', 'check', '--device-ip', '127.0.0.1', '--port', String(server.address().port), '--perf-log', '--lang', 'en'], { failLogWritesAfter: 1 });
      assert.equal(logFailure.code, 0, logFailure.stdout + logFailure.stderr);
      assert(logFailure.stderr.includes('The performance log could not be completely saved.'));
      console.log('PASS rootshell Worker HTTP, anisette WebSocket, short writes, concurrent TCP/timers and 64 KiB software TCP transfer');
    } finally { server.close(); }
  }
  console.log(`WASM SHA-256 ${createHash('sha256').update(fs.readFileSync(wasm)).digest('hex')}`);
}
