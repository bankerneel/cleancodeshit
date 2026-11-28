import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { findChrome, openChrome } from '../chrome.mjs';

const palette = new Map([
  ['1', { 'font-weight': 700 }],
  ['38;2;243;245;248', { color: '#f3f5f8' }],
  ['38;2;255;200;61', { color: '#ffc83d' }],
  ['38;2;142;145;155', { color: '#8e919b' }],
  ['38;2;229;56;59', { color: '#e5383b' }],
  ['38;2;17;18;22;48;2;243;245;248', { color: '#111216', background: '#f3f5f8' }],
  ['38;2;17;18;22;48;2;255;200;61', { color: '#111216', background: '#ffc83d' }],
]);
const controls = /\x1b\[[0-9;?]*[A-Za-z]/g;
const escape = (text) => text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');

function terminalHtml(raw) {
  const lines = raw.replaceAll('\r\n', '\n').trim().split('\n');
  let hidden = false;
  return lines.map((line) => {
    const display = line.slice(line.lastIndexOf('\r') + 1);
    const plain = display.replace(controls, '');
    if (plain.includes('Nothing to clean (')) hidden = true;
    if (hidden && (!plain || plain.includes(' Plan'))) hidden = false;
    if (hidden) return null;
    const label = plain.includes('Dry run:') ? 'dry' : plain.trimStart().startsWith('›') ? 'group' : '';
    let active = {};
    const body = display.split(/\x1b\[([0-9;]*)m/).map((part, index) => {
      if (index % 2) {
        if (part === '0') active = {};
        else {
          assert(palette.has(part), `Unsupported ANSI style ${part}`);
          Object.assign(active, palette.get(part));
        }
        return '';
      }
      const text = escape(part.replace(controls, ''));
      const css = Object.entries(active).map(([name, value]) => `${name}:${value}`).join(';');
      return css && text ? `<span style="${css}">${text}</span>` : text;
    }).join('');
    return `<span class="${label}">${body}</span>`;
  }).filter((line) => line !== null).join('\n');
}

export async function renderReadmeShot(cli) {
  const executable = findChrome();
  if (!executable) {
    console.warn('Warning: Chrome is missing. Skipping docs/sample-terminal.png.');
    return null;
  }
  const temporary = await mkdtemp(join(tmpdir(), 'ccs-readme-shot-'));
  let chrome;
  try {
    await writeFile(join(temporary, 'cleancodeshit'), cli);
    const fixture = fileURLToPath(new URL('./fixture.sh', import.meta.url));
    const platform = process.platform === 'darwin' ? 'Darwin' : 'Linux';
    const { stdout, stderr, status, error } = spawnSync('/bin/bash', [fixture, temporary, platform], {
      encoding: 'utf8', timeout: 60000,
      env: { PATH: process.env.PATH, LC_ALL: platform === 'Darwin' ? 'en_US.UTF-8' : 'C.UTF-8', SHELL: '/bin/bash' },
    });
    if (error) throw error;
    assert.equal(status, 0, stderr || 'Could not generate the sample terminal fixture');
    const version = cli.match(/^VERSION="([^"]+)"$/m)?.[1];
    const plain = stdout.replace(controls, '');
    assert(plain.includes(`cleancodeshit ${version}`) && plain.includes('Dry run:'), 'Missing sample transcript or version');
    const template = await readFile(new URL('./terminal.html', import.meta.url), 'utf8');
    const page = join(temporary, 'terminal.html');
    await writeFile(page, template.replace('<!-- transcript -->', () => terminalHtml(stdout)));

    let pageLoaded;
    chrome = await openChrome({
      executable, args: ['--disable-lcd-text', '--hide-scrollbars'],
      onEvent(message) {
        if (message.method === 'Page.loadEventFired') pageLoaded?.();
      },
    });
    const { send } = chrome;
    await send('Page.enable');
    const width = 1152;
    await send('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: 2, mobile: false });
    const loaded = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('README terminal page did not load')), 10000);
      pageLoaded = () => { clearTimeout(timer); resolve(); };
    });
    await Promise.all([loaded, send('Page.navigate', { url: pathToFileURL(page).href })]);
    const { result, exceptionDetails } = await send('Runtime.evaluate', {
      expression: `document.fonts.ready.then(() => {
          const pre = document.querySelector('pre');
          const title = [...pre.children].filter((line) => [...line.children].some((part) =>
            part.style.color === 'rgb(243, 245, 248)' && /[\\u2500-\\u259f]/.test(part.textContent)));
          return {
            width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight,
            fits: pre.scrollWidth === pre.clientWidth, dpr: devicePixelRatio,
            oneLine: title.length === 6 && title.every((line) => {
              const white = [...line.children].find((part) => part.style.color === 'rgb(243, 245, 248)');
              const yellow = [...line.children].find((part) => part.style.color === 'rgb(255, 200, 61)');
              const a = white?.getBoundingClientRect();
              const b = yellow?.getBoundingClientRect();
              return a && b && Math.abs(a.top - b.top) < 1 && b.left > a.left;
            }),
          };
      })`,
      awaitPromise: true, returnByValue: true,
    });
    assert(!exceptionDetails, JSON.stringify(exceptionDetails));
    const size = result.value;
    assert(size.width === width && size.fits && size.dpr === 2 && size.oneLine, `Unexpected terminal layout: ${JSON.stringify(size)}`);
    const { data } = await send('Page.captureScreenshot', {
      format: 'png', captureBeyondViewport: true,
      clip: { x: 0, y: 0, width, height: size.height, scale: 1 },
    });
    return Buffer.from(data, 'base64');
  } finally {
    try { await chrome?.close(); }
    finally { await rm(temporary, { recursive: true, force: true }); }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const source = await readFile('cleancodeshit', 'utf8');
  const png = await renderReadmeShot(source);
  if (png) await writeFile('docs/sample-terminal.png', png);
}
