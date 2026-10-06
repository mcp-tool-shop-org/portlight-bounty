// Scripted client for the Godot playtest bridge. No model, no product score.
//
// From the repo root:
//   node playtest/probe-bridge.mjs
//
// Expects a debug portlight_godot library already built and `godot` on PATH.
// Prints the editor version (CI is 4.7.2). Spine: hello, second client
// refused, bad line answered, Title, captains, the line Ada, Merchant (chart
// shows Ada docked and Hire). Then Contracts open and close, read the board
// cards, accept the cheapest run the market stocks, buy the order plus a
// margin, Hire, sail to the destination while the strip counts down. The
// docking day opens the Arrival card: it is read from state.day_report
// before anything presses Escape, then dismissed. Sell there. The sale is
// the delivery: the contract leaves Active, silver
// goes up, and the strip clears. Complete is not offered (R10), so that
// press is the offered-check. Journal open and close, then reset to Title.
// The last line names what was asserted. It is not a product score.

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

// Encounters are stepped without touching day-report, so an Arrival card
// survives until it has been read.
async function clearEncounterOnly(socket, result, notes) {
  let current = result;
  for (let step = 0; step < 8; step += 1) {
    if (!openDesks(current).includes('encounter') && String(current?.state?.screen) !== 'encounter') {
      return current;
    }
    const options = ids(current);
    const pick = ['encounter.auto_resolve', 'encounter.leave', 'encounter.spare', 'encounter.take_all']
      .find((id) => options.includes(id))
      ?? options.find((id) => /^encounter\.(choice|naval|combat)\./.test(id)
        || id === 'encounter.board'
        || id === 'encounter.duel');
    assert(pick, 'encounter offered no way forward', current);
    notes.encounter = notes.encounter ?? pick;
    current = await choose(socket, pick);
  }
  throw new Error('encounter did not clear');
}

function isDayReport(result) {
  return String(result?.state?.screen) === 'day-report' || openDesks(result).includes('day-report');
}

// rank_visit_movers copy, e.g. "Grain 12 to 16 (+33%)".
const MOVER_RE = /^(.+) (\d+) to (\d+) \(([+-]?\d+)%\)$/;
// arrival_contract_lines copy; the hint is dropped when nothing is needed.
const ARRIVAL_CONTRACT_RE = / - \d+\/\d+ - (\d+ days? left|due today|overdue)( - sell \d+ more .+ here)?$/;

function strip(result) {
  return String(result?.state?.contract_strip ?? '');
}

function board(result) {
  const rows = result?.state?.contract_board;
  return Array.isArray(rows) ? rows : [];
}

function activeRows(result) {
  const rows = result?.state?.contracts_active;
  return Array.isArray(rows) ? rows : [];
}

function slug(name) {
  return name.trim().toLowerCase().replace(/'/g, '').replace(/[^a-z0-9]+/g, '_');
}

// Board detail reads "Porcelain x5 to Al-Manar   400 silver + 70 bonus".
function cardTerms(card) {
  const found = String(card?.detail ?? '').match(/^(.+?) x(\d+) to (.+?)\s{2,}(\d+) silver/);
  if (!found) return null;
  const quantity = Number(found[2]);
  return {
    good: slug(found[1]),
    quantity,
    destination: found[3].trim(),
    port: slug(found[3]),
    perUnit: Number(found[4]) / Math.max(quantity, 1),
  };
}

// The first `N days left` (or due today / overdue) in the strip.
function stripTiming(text) {
  const found = text.match(/(\d+ days? left|due today|overdue)/);
  return found ? found[1] : '';
}

async function closeContracts(socket, result) {
  return String(result?.state?.screen) === 'contracts'
    ? choose(socket, 'chart.contracts.close')
    : result;
}

async function voyage(socket, chart) {
  const notes = {};
  assert(strip(chart) === '', 'a fresh game showed the contract strip', chart);
  assert(ids(chart).includes('chart.contracts.open'), 'docked chart did not offer contracts', chart);
  const opened = await choose(socket, 'chart.contracts.open');
  assert(String(opened?.state?.screen) === 'contracts', 'contracts did not open', opened);
  assert(ids(opened).includes('chart.contracts.close'), 'contracts did not offer close', opened);
  const closed = await choose(socket, 'chart.contracts.close');
  assert(String(closed?.state?.screen) === 'chart', 'contracts did not return to the chart', closed);
  assert(!openDesks(closed).includes('contracts'), 'contracts stayed open', closed);
  assert(board(closed).length === 0, 'board cards were reported with the desk closed', closed);

  // What the market sells here decides which card can be delivered.
  const market = await choose(socket, 'chart.market');
  const buyable = new Set(ids(market)
    .filter((id) => id.startsWith('chart.buy.'))
    .map((id) => id.slice('chart.buy.'.length)));
  await choose(socket, 'chart.market');

  // Read the cards. Take a run this market stocks to a port the chart can
  // sail to, cheapest goods first (lowest reward per unit), so the hold can
  // be paid for. Seed 1 Merchant picks grain to Corsair's Rest.
  const again = await choose(socket, 'chart.contracts.open');
  const cards = board(again);
  assert(cards.length > 0, 'contracts showed no board cards', again);
  assert(/^Board:$/m.test(String(again?.text ?? '')), 'board cards were not in the text', again);
  for (const card of cards) {
    assert(card.title && card.detail, `board card ${card.id} had no title or detail`, again);
  }
  const pick = cards
    .map((card) => ({ card, terms: cardTerms(card) }))
    .filter(({ card, terms }) => terms
      && buyable.has(terms.good)
      && ids(again).includes(`chart.sail.${terms.port}`)
      && card.actions.includes(`contracts.accept.${card.id}`))
    .sort((a, b) => a.terms.perUnit - b.terms.perUnit)[0];
  assert(pick, `no card for goods sold here (${[...buyable].join(', ')})`, again);
  const { card, terms } = pick;
  const acceptId = `contracts.accept.${card.id}`;
  const accepted = await choose(socket, acceptId);
  assert(!ids(accepted).includes(acceptId), `${acceptId} was still offered after accept`, accepted);
  assert(!board(accepted).some((row) => row.id === card.id), 'accepted card stayed on the board', accepted);
  const row = activeRows(accepted).find((active) => active.id === card.id);
  assert(row, 'accepted contract was not an active row', accepted);
  assert(row.detail.includes(`0/${terms.quantity}`), `active row did not show 0/${terms.quantity}`, accepted);
  assert(strip(accepted).includes(card.title), 'strip did not name the accepted contract', accepted);
  const desk = await closeContracts(socket, accepted);
  assert(String(desk?.state?.screen) === 'chart', 'chart was not back after accept', desk);
  assert(/^Contract strip: /m.test(String(desk?.text ?? '')), 'strip was not in the chart text', desk);
  const stripAtAccept = strip(desk);

  // Rough seas take 1-3 units per event, so carry a margin over the order.
  const silverBefore = Number(desk?.state?.silver);
  const shop = await choose(socket, 'chart.market');
  const buyId = `chart.buy.${terms.good}`;
  assert(ids(shop).includes(buyId), `market did not offer ${terms.good}`, shop);
  let bought = shop;
  const want = terms.quantity + 3;
  for (let unit = 0; unit < want; unit += 1) {
    const before = Number(bought?.state?.silver);
    bought = await choose(socket, buyId);
    assert(Number(bought?.state?.silver) < before, `buying ${terms.good} did not spend silver`, bought);
  }
  assert(Number(bought?.state?.silver) < silverBefore, 'buying did not spend silver', bought);

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

  const sailed = await choose(socket, `chart.sail.${terms.port}`);
  assert(
    String(sailed?.text ?? '').includes(`Departed for ${terms.destination}`),
    'depart log missing',
    sailed,
  );
  assert(String(sailed?.state?.docked ?? '') === '', 'sail left the ship docked', sailed);
  assert(strip(sailed) !== '', 'strip hid at sea', sailed);

  const startDay = Number(chart?.state?.day);
  let here = sailed;
  let arrival = null;
  for (let step = 0; step < 40; step += 1) {
    // At sea a day report may still be escaped; it is not the Arrival card.
    here = await clearOverlays(socket, here, notes);
    assert(String(here?.state?.screen) === 'chart', 'expected the chart while under way', here);
    assert(ids(here).includes('chart.next_day'), 'next day was not offered', here);
    here = await choose(socket, 'chart.next_day');
    if (/Rough seas damaged/.test(String(here?.text ?? ''))) notes.roughSeas = true;
    if (String(here?.state?.docked) === terms.port) {
      // The docking reply. Do not press Escape before the card is read.
      here = await clearEncounterOnly(socket, here, notes);
      arrival = here;
      break;
    }
  }
  assert(arrival, `did not dock at ${terms.destination}`, here);

  // Arrival card (#57): the deliverable contract makes the docking day notable.
  assert(isDayReport(arrival), 'arrival card did not open on the docking advance', arrival);
  assert(ids(arrival).includes('chart.day_report.close'), 'arrival card offered no Close', arrival);
  const report = arrival?.state?.day_report;
  assert(report && typeof report === 'object', 'observation had no state.day_report', arrival);
  assert(
    report.title === `Arrived - ${terms.destination}`,
    `arrival title was ${JSON.stringify(report.title)}`,
    arrival,
  );
  assert(report.eyebrow === "Day's report", `arrival eyebrow was ${JSON.stringify(report.eyebrow)}`, arrival);
  assert(
    String(arrival?.text ?? '').includes(`Day's report: Arrived - ${terms.destination}`),
    'arrival card was not in the text',
    arrival,
  );
  const sections = Array.isArray(report.sections) ? report.sections : [];
  assert(!sections.some((section) => section.id === 'prices'), 'Prices section shown on arrival day', arrival);
  const arrivalSection = sections.find((section) => section.id === 'arrival');
  assert(arrivalSection && arrivalSection.lines.length > 0, 'no Arrival section on arrival', arrival);
  const lines = arrivalSection.lines.map(String);
  const contractRows = lines.filter((line) => ARRIVAL_CONTRACT_RE.test(line));
  const movers = lines.filter((line) => MOVER_RE.test(line));
  const more = lines.filter((line) => /^\+\d+ more$/.test(line));
  assert(contractRows.some((line) => line.startsWith(card.title)), 'accepted contract not in the Arrival section', arrival);
  assert(contractRows.length <= 3, 'arrival contract rows over the cap', arrival);
  assert(movers.length <= 3, `more than 3 movers (${movers.length})`, arrival);
  for (const mover of movers) {
    const [, , oldPrice, newPrice, pct] = mover.match(MOVER_RE);
    const expect = Math.round(((Number(newPrice) - Number(oldPrice)) / Math.max(Number(oldPrice), 1)) * 100);
    assert(Number(pct) === expect && Math.abs(expect) >= 10, `mover math or threshold off: ${mover}`, arrival);
  }
  assert(
    contractRows.length + movers.length + more.length === lines.length,
    'unknown line in the Arrival section',
    arrival,
  );
  assert(!lines.some((line) => /due soon|\u2014|Complete|Deliver/.test(line)), 'forbidden arrival copy', arrival);
  notes.arrivalMovers = movers.length;

  here = await call(socket, 'act', { kind: 'key', key: 'escape' });
  assert(!isDayReport(here), 'Escape left the arrival card open', here);
  assert(!ids(here).includes('chart.day_report.close'), 'day-report Close still offered after dismiss', here);
  assert(here?.state?.day_report == null, 'state.day_report still reported after dismiss', here);
  here = await clearOverlays(socket, here, notes);
  assert(String(here?.state?.docked) === terms.port, `dismiss left ${terms.destination}`, here);
  assert(Number(here?.state?.day) > startDay, 'the day counter did not advance', here);
  assert(String(here?.text ?? '').includes(`Docked at ${terms.destination}`), 'arrival text missing', here);
  const stripAtArrival = strip(here);
  assert(stripAtArrival.includes(card.title), 'strip lost the contract on the way', here);
  assert(
    stripTiming(stripAtArrival) !== stripTiming(stripAtAccept),
    `strip countdown did not move (${stripTiming(stripAtAccept)} -> ${stripTiming(stripAtArrival)})`,
    here,
  );

  // Delivery is the sale. One press sells one unit; the sale that fills the
  // order settles it, so the strip clears without a Complete press.
  if (!ids(here).includes(`chart.sell.${terms.good}`)) {
    here = await choose(socket, 'chart.market');
  }
  const sellId = `chart.sell.${terms.good}`;
  assert(ids(here).includes(sellId), `${terms.destination} market did not offer ${terms.good}`, here);
  const silverAtPort = Number(here?.state?.silver);
  let sold = here;
  for (let unit = 0; unit < want && strip(sold) !== ''; unit += 1) {
    sold = await choose(socket, sellId);
    assert(
      !/Only have 0 units/.test(String(sold?.text ?? '').split('\n').slice(-1)[0] ?? ''),
      `hold ran out before ${terms.quantity} ${terms.good} were delivered; encounter ${notes.encounter ?? 'none'}`,
      sold,
    );
  }
  assert(strip(sold) === '', 'strip did not clear after delivering the order', sold);
  assert(!/^Contract strip: /m.test(String(sold?.text ?? '')), 'strip line stayed in the text', sold);
  assert(Number(sold?.state?.silver) > silverAtPort, `silver did not go up (${silverAtPort} -> ${sold?.state?.silver})`, sold);

  // R10: the sale settled it, so Complete is not offered. The press is the
  // offered-check.
  const desk2 = await choose(socket, 'chart.contracts.open');
  assert(!activeRows(desk2).some((active) => active.id === card.id), 'delivered contract stayed active', desk2);
  const completeId = `contracts.complete.${card.id}`;
  assert(!ids(desk2).includes(completeId), `${completeId} was offered after the sale settled it`, desk2);
  let missed = false;
  try {
    await choose(socket, completeId);
  } catch (error) {
    missed = /not on screen/.test(error instanceof Error ? error.message : '');
  }
  assert(missed, `${completeId} press was not an offered-check miss`, desk2);
  const afterComplete = await closeContracts(socket, desk2);
  assert(String(afterComplete?.state?.screen) === 'chart', 'chart was not back after contracts', afterComplete);

  const journal = await choose(socket, 'chart.journal.open');
  assert(String(journal?.state?.screen) === 'journal', 'journal did not open', journal);
  const back = await choose(socket, 'chart.journal.close');
  assert(String(back?.state?.screen) === 'chart', 'journal did not close', back);

  const parts = [
    `board ${cards.length} cards`,
    `accept ${terms.good} x${terms.quantity}`,
    'hire',
    `sail ${terms.destination}`,
    `arrival card read (${notes.arrivalMovers} movers) then escaped`,
    `strip ${stripTiming(stripAtAccept)} -> ${stripTiming(stripAtArrival)}`,
    `sell delivers (silver ${silverAtPort} -> ${sold?.state?.silver}, strip clear, not active)`,
    'complete not offered',
    'journal',
  ];
  if (notes.roughSeas) parts.push('rough seas');
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
