// Scripted client for the Godot playtest bridge. No model, no product score.
//
// From the repo root:
//   node playtest/probe-bridge.mjs
//
// Expects a debug portlight_godot.dll already built. Prints the editor version
// (this rig is 4.7 stable; CI is 4.7.2). Hello, Title, captains, the line Ada,
// Merchant, then a scripted spine: contracts, grain, Hire, sail to Al-Manar,
// journal open and close. Reset must return to Title. The last line names
// what was asserted. It is not a product score.

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
    let settled = false;
    const finish = (fn) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      fn();
    };
    const timer = setTimeout(() => {
      finish(() => rej(new Error(`no PLAYTEST_BRIDGE_PORT within ${PORT_WAIT_MS}ms\n${(err || out).slice(-2000)}`)));
    }, PORT_WAIT_MS);
    const take = (text) => {
      const found = text.match(/PLAYTEST_BRIDGE_PORT=(\d+)/);
      if (found) finish(() => res(Number(found[1])));
    };
    child.stdout?.on('data', () => take(`${out}\n${err}`));
    child.stderr?.on('data', () => take(`${out}\n${err}`));
    child.once('exit', (code) => {
      finish(() => rej(new Error(`godot exited ${code} before the bridge listened\n${(err || out).slice(-2000)}`)));
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
      if (msg.id !== id) {
        rej(new Error(`${method}: reply id ${msg.id} was not ${id}`));
        return;
      }
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
  const text = typeof result?.text === 'string' ? result.text.slice(-2500) : '';
  throw new Error(`${message}\n${text}`);
}

function openDesks(result) {
  const open = result?.state?.open;
  return Array.isArray(open) ? open.map(String) : [];
}

function readLine(socket, ms) {
  return new Promise((res, rej) => {
    let buf = '';
    const timer = setTimeout(() => {
      socket.off('data', onData);
      rej(new Error(`no line within ${ms}ms`));
    }, ms);
    const onData = (chunk) => {
      buf += chunk;
      const nl = buf.indexOf('\n');
      if (nl < 0) return;
      clearTimeout(timer);
      socket.off('data', onData);
      res(buf.slice(0, nl));
    };
    socket.on('data', onData);
  });
}

async function choose(socket, id) {
  return call(socket, 'act', { kind: 'choose', id });
}

// Day's report closes on Escape. Hunt and Crew do not. An encounter is
// stepped with whatever dismiss it actually offers. Stance buttons are
// recorded when present and are not required on a choice or naval screen.
async function clearOverlays(socket, result, notes) {
  let current = result;
  for (let step = 0; step < 8; step += 1) {
    const screen = String(current?.state?.screen ?? '');
    const open = openDesks(current);
    if (screen === 'day-report' || open.includes('day-report')) {
      const after = await call(socket, 'act', { kind: 'key', key: 'escape' });
      const still = String(after?.state?.screen) === 'day-report'
        || openDesks(after).includes('day-report');
      assert(!still, 'escape left day-report open', after);
      notes.dayReport = 'escape-closed';
      current = after;
      continue;
    }
    if (screen !== 'encounter' && !open.includes('encounter')) return current;
    const options = ids(current);
    if (options.some((id) => id.startsWith('encounter.stance.'))) notes.stance = true;
    const prefer = ['encounter.auto_resolve', 'encounter.leave', 'encounter.spare', 'encounter.take_all'];
    const pick = prefer.find((id) => options.includes(id))
      ?? options.find((id) => id.startsWith('encounter.choice.')
        || id.startsWith('encounter.naval.')
        || id.startsWith('encounter.combat.')
        || id === 'encounter.board'
        || id === 'encounter.duel');
    assert(pick, 'encounter offered no way forward', current);
    notes.encounter = notes.encounter ?? pick;
    current = await choose(socket, pick);
  }
  throw new Error('an overlay did not clear');
}

async function voyage(socket, chart) {
  const notes = {};
  assert(ids(chart).includes('chart.contracts.open'), 'docked chart did not offer contracts', chart);
  const opened = await choose(socket, 'chart.contracts.open');
  assert(String(opened?.state?.screen) === 'contracts', 'contracts did not open', opened);
  assert(ids(opened).includes('chart.contracts.close'), 'contracts did not offer close', opened);
  const closed = await choose(socket, 'chart.contracts.close');
  assert(String(closed?.state?.screen) === 'chart', 'contracts did not return to the chart', closed);
  assert(!openDesks(closed).includes('contracts'), 'contracts stayed open', closed);

  const again = await choose(socket, 'chart.contracts.open');
  const acceptId = ids(again).find((id) => id.startsWith('contracts.accept.'));
  assert(acceptId, 'contracts offered no accept', again);
  const accepted = await choose(socket, acceptId);
  assert(!ids(accepted).includes(acceptId), `${acceptId} was still offered after accept`, accepted);
  const desk = String(accepted?.state?.screen) === 'contracts'
    ? await choose(socket, 'chart.contracts.close')
    : accepted;
  assert(String(desk?.state?.screen) === 'chart', 'chart was not back after accept', desk);

  const silverBefore = Number(desk?.state?.silver);
  const market = await choose(socket, 'chart.market');
  assert(ids(market).includes('chart.buy.grain'), 'market did not offer grain', market);
  // One press buys one unit. This seed's cargo-damage event removes up to
  // three, which empties a one-unit hold before the Al-Manar sale.
  let bought = await choose(socket, 'chart.buy.grain');
  assert(Number(bought?.state?.silver) < silverBefore, 'buying grain did not spend silver', bought);
  for (let extra = 0; extra < 3; extra += 1) {
    const before = Number(bought?.state?.silver);
    bought = await choose(socket, 'chart.buy.grain');
    assert(Number(bought?.state?.silver) < before, 'further grain buy did not spend silver', bought);
  }

  const beforeHire = String(bought?.text ?? '');
  const hired = await choose(socket, 'chart.hire');
  const hiredText = String(hired?.text ?? '');
  const hiredOk = /Hired 1 sailor/.test(hiredText);
  const unaffordable = /Need \d+ silver for \d+ Sailor/.test(hiredText);
  assert(
    hiredOk || unaffordable,
    'hire neither logged a sailor nor an unaffordable reason',
    hired,
  );
  assert(hiredText !== beforeHire, 'hire returned the same observation', hired);

  const sailed = await choose(socket, 'chart.sail.al_manar');
  assert(/Departed for Al-Manar/.test(String(sailed?.text ?? '')), 'depart log missing', sailed);
  assert(String(sailed?.state?.docked ?? '') === '', 'sail left the ship docked', sailed);

  const startDay = Number(chart?.state?.day);
  let here = sailed;
  for (let step = 0; step < 40 && String(here?.state?.docked) !== 'al_manar'; step += 1) {
    here = await clearOverlays(socket, here, notes);
    if (String(here?.state?.docked) === 'al_manar') break;
    assert(String(here?.state?.screen) === 'chart', 'expected the chart while under way', here);
    assert(ids(here).includes('chart.next_day'), 'next day was not offered', here);
    here = await choose(socket, 'chart.next_day');
  }
  here = await clearOverlays(socket, here, notes);
  assert(String(here?.state?.docked) === 'al_manar', 'did not dock at Al-Manar', here);
  assert(Number(here?.state?.day) > startDay, 'the day counter did not advance', here);
  assert(/Docked at Al-Manar/.test(String(here?.text ?? '')), 'arrival text missing', here);

  if (!ids(here).includes('chart.sell.grain')) {
    here = await choose(socket, 'chart.market');
  }
  assert(ids(here).includes('chart.sell.grain'), 'Al-Manar market did not offer grain', here);
  const silverAtPort = Number(here?.state?.silver);
  const sold = await choose(socket, 'chart.sell.grain');
  const soldText = String(sold?.text ?? '');
  const grainLine = soldText.split('\n').find((line) => /grain/i.test(line)) ?? '';
  assert(
    Number(sold?.state?.silver) !== silverAtPort,
    `selling grain did not move silver (${silverAtPort} -> ${sold?.state?.silver}); ${grainLine}; encounter ${notes.encounter ?? 'none'}`,
    sold,
  );

  // Seed 1's first offer is not fulfilled by a grain sale at Al-Manar.
  // When complete is absent, the press is the offered-check. When it is
  // present, the second press is the idempotency guard.
  const completing = await choose(socket, 'chart.contracts.open');
  const completeId = `contracts.complete.${acceptId.slice('contracts.accept.'.length)}`;
  let afterComplete;
  let completeNote;
  if (ids(completing).includes(completeId)) {
    const completed = await choose(socket, completeId);
    assert(!ids(completed).includes(completeId), `${completeId} was still offered`, completed);
    let idempotent = false;
    try {
      await choose(socket, completeId);
    } catch (error) {
      idempotent = /not on screen/.test(error instanceof Error ? error.message : '');
    }
    assert(idempotent, 'a second complete was not an offered-check miss', completed);
    afterComplete = String(completed?.state?.screen) === 'contracts'
      ? await choose(socket, 'chart.contracts.close')
      : completed;
    completeNote = 'complete idempotent';
  } else {
    let missed = false;
    try {
      await choose(socket, completeId);
    } catch (error) {
      missed = /not on screen/.test(error instanceof Error ? error.message : '');
    }
    assert(missed, `${completeId} was absent but the press was not an offered-check miss`, completing);
    afterComplete = String(completing?.state?.screen) === 'contracts'
      ? await choose(socket, 'chart.contracts.close')
      : completing;
    completeNote = 'complete not offered';
  }
  assert(String(afterComplete?.state?.screen) === 'chart', 'chart was not back after contracts', afterComplete);

  const journal = await choose(socket, 'chart.journal.open');
  assert(String(journal?.state?.screen) === 'journal', 'journal did not open', journal);
  const back = await choose(socket, 'chart.journal.close');
  assert(String(back?.state?.screen) === 'chart', 'journal did not close', back);

  const parts = ['contracts', 'grain', 'hire', 'sail Al-Manar', completeNote, 'journal'];
  if (/Rough seas damaged/.test(soldText)) parts.push('rough seas');
  if (notes.encounter) parts.push(`encounter ${notes.encounter}`);
  if (notes.dayReport) parts.push(notes.dayReport);
  if (notes.stance) parts.push('stances offered');
  return parts.join(' -> ');
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

    const extra = await connect(port);
    try {
      const line = await readLine(extra, CALL_MS);
      const msg = JSON.parse(line);
      assert(
        /already has a client/.test(msg?.error?.message ?? ''),
        'second client was not refused',
        { text: line },
      );
    } finally {
      extra.destroy();
    }
    socket.write('[]\n');
    const bad = JSON.parse(await readLine(socket, CALL_MS));
    assert(
      /one JSON object per line/.test(bad?.error?.message ?? ''),
      'a bad line got no error',
      { text: JSON.stringify(bad) },
    );

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

    const spine = await voyage(socket, chart);
    const again = await call(socket, 'reset');
    assert(String(again?.state?.screen) === 'title', 'reset did not return to title', again);
    assert(!ids(again).includes('chart.hire'), 'reset left Hire on screen', again);

    await call(socket, 'quit');
    console.log(`probe ok: title -> Ada merchant -> ${spine} -> reset title -> second client refused -> bad line answered`);
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
