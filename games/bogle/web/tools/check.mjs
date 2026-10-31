// Headless smoke check of the Bogle page (games/bogle/web/build.sh output, served over
// HTTP): loads it in headless Chrome, reports console errors, frame rate and frame cost,
// the sound path, and takes screenshots of the title, the menu (after Enter) and play.
//
//   node games/bogle/web/tools/check.mjs <url> <shot.png> [seconds before the first shot]
//
// Use <url>?autostart (the page's click-to-start otherwise waits). Environment:
//   CHROME       the Chrome binary (default: the macOS app path, else google-chrome)
//   CHROME_ARGS  extra Chrome switches, space separated
//   NO_JSPI=1    remove WebAssembly JSPI before the page runs (a browser without it)
//   QUIT=1       instead: open the menu, choose Quit, and report what the page shows
// Exits 1 when the page logged an error.
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
const [url, shot, secs = '8'] = process.argv.slice(2);
if (!url || !shot) {
  console.error('usage: node check.mjs <url> <shot.png> [seconds]');
  process.exit(2);
}
const mac = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
const chromePath = process.env.CHROME || (existsSync(mac) ? mac : 'google-chrome');
const prof = mkdtempSync(join(tmpdir(), 'bogle-check-'));
const chrome = spawn(chromePath, [
  '--headless=new', '--remote-debugging-port=9333', `--user-data-dir=${prof}`, '--no-first-run',
  '--window-size=960,600', '--autoplay-policy=no-user-gesture-required',
  ...(process.env.CHROME_ARGS ? process.env.CHROME_ARGS.split(' ') : []), 'about:blank'], { stdio: 'ignore' });
const finish = (code) => {
  ws.close(); chrome.kill();
  setTimeout(() => { rmSync(prof, { recursive: true, force: true }); process.exit(code); }, 300);
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try { target = (await (await fetch('http://127.0.0.1:9333/json')).json()).find((t) => t.type === 'page'); } catch {}
}
const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener('open', r));
let id = 0; const waiting = new Map();
const send = (method, params = {}) => new Promise((r) => { waiting.set(++id, r); ws.send(JSON.stringify({ id, method, params })); });
let errors = 0;
ws.addEventListener('message', (m) => {
  const msg = JSON.parse(m.data);
  if (msg.id && waiting.has(msg.id)) { waiting.get(msg.id)(msg.result); waiting.delete(msg.id); }
  if (msg.method === 'Runtime.consoleAPICalled') {
    const p = msg.params; if (p.type === 'error') errors++;
    console.log(`console.${p.type}: ${p.args.map((a) => a.value ?? a.description).join(' ')}`);
  }
  if (msg.method === 'Runtime.exceptionThrown') { errors++; console.log('EXCEPTION:', msg.params.exceptionDetails.exception?.description ?? msg.params.exceptionDetails.text); }
  if (msg.method === 'Log.entryAdded' && msg.params.entry.level === 'error') { errors++; console.log('LOG:', msg.params.entry.text, msg.params.entry.url ?? ''); }
});
await send('Runtime.enable'); await send('Log.enable'); await send('Page.enable');
// Headless windows never hold focus; the game pauses input without it.
await send('Emulation.setFocusEmulationEnabled', { enabled: true });
// NO_JSPI=1: a browser without JSPI. Chrome ships JSPI unflagged (no switch turns it
// off), so remove its API before any page script runs.
if (process.env.NO_JSPI) {
  await send('Page.addScriptToEvaluateOnNewDocument', {
    source: 'delete WebAssembly.Suspending; delete WebAssembly.promising;' });
}
const t0 = Date.now();
await send('Page.navigate', { url });
await sleep(Number(secs) * 1000);
const probe = await send('Runtime.evaluate', { expression: 'JSON.stringify({presented: !!(window.Module && Module.mdhScreen), overlay: document.getElementById("overlay").className, title: document.title, focus: document.hasFocus(), jspi: typeof WebAssembly.Suspending})', returnByValue: true });
console.log('state:', probe.result.value, `after ${Date.now() - t0} ms`);
const img = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(shot, Buffer.from(img.data, 'base64'));
// Frame cost: from each display frame's resume to the frame's present, over 3 s.
const measure = () => send('Runtime.evaluate', { awaitPromise: true, returnByValue: true, expression: `new Promise((done) => {
  const s = Module.mdhScreen, put = s.ctx.putImageData.bind(s.ctx), raf = window.requestAnimationFrame;
  let resumed = 0, work = [], frames = 0;
  window.requestAnimationFrame = (cb) => raf((t) => { resumed = performance.now(); cb(t); });
  s.ctx.putImageData = (...a) => { if (resumed) work.push(performance.now() - resumed); frames++; return put(...a); };
  const t0 = performance.now();
  setTimeout(() => { work.sort((a, b) => a - b); done(JSON.stringify({fps: frames / ((performance.now() - t0) / 1000), median_ms: work[work.length >> 1], p95_ms: work[Math.floor(work.length * 0.95)], n: work.length})); }, 3000);
})` }).then((r) => r.result.value);
const press = async (code, key, vk, hold = 200) => {
  await send('Input.dispatchKeyEvent', { type: 'keyDown', code, key, windowsVirtualKeyCode: vk });
  await sleep(hold);
  await send('Input.dispatchKeyEvent', { type: 'keyUp', code, key, windowsVirtualKeyCode: vk });
};
// QUIT=1: from the menu, Down three times to Quit and Enter; the page should say so.
if (process.env.QUIT) {
  await press('Enter', 'Enter', 13); await sleep(800);
  for (let i = 0; i < 3; i++) { await press('ArrowDown', 'ArrowDown', 40); await sleep(300); }
  await press('Enter', 'Enter', 13); await sleep(1500);
  const after = await send('Runtime.evaluate', { returnByValue: true, expression:
    'document.getElementById("overlay").innerText + " | running=" + !!(window.Module && Module.mdhScreen) + " | sound=" + !!(window.Module && Module.mdhSound)' });
  console.log('after quit:', after.result.value);
  const img = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(shot.replace(/\.png$/, '_quit.png'), Buffer.from(img.data, 'base64'));
  console.log(`errors: ${errors}`);
  finish(errors ? 1 : 0);
}
console.log('timing (title):', await measure());
// Keyboard: a key on the title moves on.
for (const type of ['keyDown', 'keyUp']) {
  await send('Input.dispatchKeyEvent', { type, code: 'Enter', key: 'Enter', windowsVirtualKeyCode: 13 });
  await sleep(200); // held across frames: the game reads held keys
}
await sleep(1500);
const img2 = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(shot.replace(/\.png$/, '_key.png'), Buffer.from(img2.data, 'base64'));
// New game, through any briefing, into play; then frame cost while walking.
for (let i = 0; i < 4; i++) { await press('Enter', 'Enter', 13); await sleep(1200); }
await send('Input.dispatchKeyEvent', { type: 'keyDown', code: 'KeyW', key: 'w', windowsVirtualKeyCode: 87 });
console.log('timing (play, w held):', await measure());
await send('Input.dispatchKeyEvent', { type: 'keyUp', code: 'KeyW', key: 'w', windowsVirtualKeyCode: 87 });
const img3 = await send('Page.captureScreenshot', { format: 'png' });
writeFileSync(shot.replace(/\.png$/, '_play.png'), Buffer.from(img3.data, 'base64'));
const sound = await send('Runtime.evaluate', { awaitPromise: true, returnByValue: true, expression: `new Promise((done) => {
  const s = Module.mdhSound; if (!s) return done('no sound');
  const a = {played: s.played, written: s.written, t: performance.now()};
  setTimeout(() => done(JSON.stringify({state: s.ctx.state, rate: s.ctx.sampleRate, node: s.node && s.node.constructor.name,
    played_per_s: (s.played - a.played) / ((performance.now() - a.t) / 1000), queued: s.written - s.played})), 2000);
})` });
console.log('sound:', sound.result.value);
console.log(`errors: ${errors}`);
finish(errors ? 1 : 0);
