// Scripted client for the Godot playtest bridge. No model, no product score.
//
// From the repo root:
//   node playtest/probe-bridge.mjs
//
// Expects a debug portlight_godot.dll already built. Prints the editor version
// (this rig is 4.7 stable; CI is 4.7.2). Hello, Title, captains, the line Ada,
// Merchant, then Hire on a docked chart. Reset must return to Title.

import { spawn } from 'node:child_process';
import { createConnection } from 'node:net';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const godotDir = resolve(root, 'godot');
const PORT_WAIT_MS = 90_000;
const CALL_MS = 20_000;

function runVersion() {
  return new Promise((resolveVersion) => {
    const child = spawn('godot', ['--version'], { cwd: root, windowsHide: true });
    let out = '';
    child.stdout?.setEncoding('utf8');
    child.stderr?.setEncoding('utf8');
    child.stdout?.on('data', (chunk) => { out += chunk; });
    child.stderr?.on('data', (chunk) => { out += chunk; });
    child.on('exit', () => resolveVersion(out.trim() || 'unknown'));
    child.on('error', () => resolveVersion('godot not on PATH'));
  });
}

function startGame() {
  const child = spawn(
    'godot',
    ['--headless', '--path', godotDir, '--', '--playtest-port=7777'],
    { cwd: root, stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true },
  );
  let out = '';
  let err = '';
  child.stdout?.setEncoding('utf8');
  child.stderr?.setEncoding('utf8');
  child.stdout?.on('data', (chunk) => { out += chunk; });
  child.stderr?.on('data', (chunk) => { err += chunk; });
  const port = new Promise((res, rej) => {
    const timer = setTimeout(() => {
      rej(new Error(`no PLAYTEST_BRIDGE_PORT within ${PORT_WAIT_MS}ms\n${(err || out).slice(-2000)}`));
    }, PORT_WAIT_MS);
    const take = (text) => {
      const found = text.match(/PLAYTEST_BRIDGE_PORT=(\d+)/);
      if (found) {
        clearTimeout(timer);
        res(Number(found[1]));
      }
    };
    child.stdout?.on('data', () => take(`${out}\n${err}`));
    child.stderr?.on('data', () => take(`${out}\n${err}`));
    child.once('exit', (code) => {
      clearTimeout(timer);
      rej(new Error(`godot exited ${code} before the bridge listened\n${(err || out).slice(-2000)}`));
    });
  });
  return { child, port };
}

function connect(port) {
  return new Promise((res, rej) => {
    const socket = createConnection({ host: '127.0.0.1', port });
    const timer = setTimeout(() => {
      socket.destroy();
      rej(new Error(`connect timed out on ${port}`));
    }, CALL_MS);
    socket.setEncoding('utf8');
    socket.once('connect', () => { clearTimeout(timer); res(socket); });
    socket.once('error', (error) => { clearTimeout(timer); rej(error); });
  });
}

function call(socket, method, params) {
  return new Promise((res, rej) => {
    let buf = '';
    const timer = setTimeout(() => {
      socket.off('data', onData);
      rej(new Error(`${method} did not answer within ${CALL_MS}ms`));
    }, CALL_MS);
    const onData = (chunk) => {
      buf += chunk;
      const nl = buf.indexOf('\n');
      if (nl < 0) return;
      clearTimeout(timer);
      socket.off('data', onData);
      const msg = JSON.parse(buf.slice(0, nl));
      if (msg.error) rej(new Error(`${method}: ${msg.error.message ?? 'error'}`));
      else res(msg.result ?? {});
    };
    socket.on('data', onData);
    const id = Math.floor(Math.random() * 1_000_000);
    socket.write(JSON.stringify({ id, method, ...(params === undefined ? {} : { params }) }) + '\n');
  });
}

function ids(result) {
  const options = result?.actions?.options ?? [];
  return options.map((option) => option.id);
}

function assert(cond, message, result) {
  if (cond) return;
  const text = typeof result?.text === 'string' ? result.text.slice(0, 800) : '';
  throw new Error(`${message}\n${text}`);
}

const version = await runVersion();
console.log(`godot ${version}`);
const { child, port: portReady } = startGame();
let failed = false;
try {
  const port = await portReady;
  console.log(`bridge ${port}`);
  const socket = await connect(port);
  try {
    const hello = await call(socket, 'hello', { protocol: 1 });
    assert(hello.protocol === 1, 'hello was not protocol 1', hello);

    const title = await call(socket, 'observe');
    assert(String(title?.state?.screen) === 'title', 'first screen was not title', title);
    assert(ids(title).includes('newgame.captains'), 'title did not offer newgame.captains', title);

    const captains = await call(socket, 'act', { kind: 'choose', id: 'newgame.captains' });
    assert(String(captains?.state?.screen) === 'captains', 'captains page did not open', captains);

    await call(socket, 'act', { kind: 'line', line: 'Ada' });
    const chart = await call(socket, 'act', { kind: 'choose', id: 'newgame.start.merchant' });
    const offered = ids(chart);
    assert(offered.includes('chart.hire'), 'docked chart did not offer chart.hire', chart);
    const text = String(chart?.text ?? '');
    assert(/Ada/.test(text) && /Docked/.test(text), 'status did not show Ada docked', chart);
    assert(chart?.state?.hire_on_screen === true, 'hire_on_screen was not true', chart);

    const again = await call(socket, 'reset');
    assert(String(again?.state?.screen) === 'title', 'reset did not return to title', again);
    assert(!ids(again).includes('chart.hire'), 'reset left Hire on screen', again);

    await call(socket, 'quit');
    console.log('probe ok: title -> Ada merchant -> Hire -> reset title');
  } finally {
    socket.destroy();
  }
} catch (error) {
  failed = true;
  console.error(error instanceof Error ? error.message : error);
} finally {
  if (child.exitCode === null) child.kill();
}
process.exit(failed ? 1 : 0);
