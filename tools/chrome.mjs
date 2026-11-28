import { spawn } from 'node:child_process';
import { accessSync, constants } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { delimiter, join, resolve } from 'node:path';

export function findChrome() {
  const path = process.env.PATH?.split(delimiter) ?? [];
  const candidates = process.env.CHROME_BIN ? [process.env.CHROME_BIN]
    : process.platform === 'win32' ? ['C:/Program Files/Google/Chrome/Application/chrome.exe']
    : process.platform === 'darwin' ? ['/Applications/Google Chrome.app/Contents/MacOS/Google Chrome']
    : ['google-chrome', 'google-chrome-stable', 'chromium', 'chromium-browser'];
  for (const name of candidates) {
    for (const file of /[/\\]/.test(name) ? [resolve(name)] : path.map((dir) => join(dir, name))) {
      try {
        accessSync(file, constants.X_OK);
        return file;
      } catch {
        // Try the next installed executable.
      }
    }
  }
  return null;
}

async function connect(executable, args, onEvent) {
  const profile = await mkdtemp(join(tmpdir(), 'ccs-chrome-'));
  const chrome = spawn(executable, [
    '--headless=new', '--remote-debugging-port=0', '--user-data-dir=' + profile, '--no-first-run',
    '--disable-background-networking', ...args, 'about:blank',
  ], { stdio: ['ignore', 'ignore', 'pipe'], windowsHide: true });
  const exited = new Promise((resolveExit) => chrome.once('close', resolveExit));
  const pending = new Map();
  let ws;
  let id = 0;

  function rejectPending() {
    for (const { reject, timer } of pending.values()) {
      clearTimeout(timer);
      reject(new Error('Chrome closed the debugging connection'));
    }
    pending.clear();
  }

  function send(method, params = {}, timeout = 7000) {
    const next = ++id;
    return new Promise((resolveCall, reject) => {
      if (ws?.readyState !== WebSocket.OPEN) return reject(new Error('Chrome debugging connection is closed'));
      const timer = setTimeout(() => {
        pending.delete(next);
        reject(new Error(`${method} timed out`));
      }, timeout);
      pending.set(next, { resolveCall, reject, timer });
      ws.send(JSON.stringify({ id: next, method, params }));
    });
  }

  async function close() {
    if (ws?.readyState === WebSocket.OPEN) await send('Browser.close', {}, 1000).catch(() => {});
    ws?.close();
    if (chrome.exitCode === null && chrome.pid) chrome.kill();
    await exited;
    await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  }

  try {
    const endpoint = await new Promise((resolveEndpoint, reject) => {
      let stderr = '';
      const timer = setTimeout(() => finish(new Error('Chrome did not advertise a debugging endpoint within 45 seconds')), 45000);
      function finish(error, url) {
        clearTimeout(timer);
        chrome.stderr.off('data', onData);
        if (error) reject(error); else resolveEndpoint(url);
      }
      function onData(chunk) {
        stderr = (stderr + chunk).slice(-4096);
        const endpoint = /DevTools listening on ws:\/\/([^/\s]+)\//.exec(stderr)?.[1];
        if (endpoint) finish(null, 'http://' + endpoint + '/json/list');
      }
      chrome.once('error', finish);
      chrome.once('exit', (code) => finish(new Error('Chrome exited ' + code)));
      chrome.stderr.on('data', onData);
      chrome.stderr.resume();
    });
    const targets = await (await fetch(endpoint, { signal: AbortSignal.timeout(7000) })).json();
    const target = targets.find(({ type }) => type === 'page');
    if (!target) throw new Error('Chrome did not expose a page to debug');
    ws = new WebSocket(target.webSocketDebuggerUrl);
    await new Promise((resolveSocket, reject) => {
      const timer = setTimeout(() => reject(new Error('Chrome debugging connection timed out')), 7000);
      ws.addEventListener('open', () => { clearTimeout(timer); resolveSocket(); }, { once: true });
      ws.addEventListener('error', () => { clearTimeout(timer); reject(new Error('Chrome debugging connection failed')); }, { once: true });
    });
    ws.addEventListener('close', rejectPending);
    ws.addEventListener('message', ({ data }) => {
      const message = JSON.parse(data);
      const response = pending.get(message.id);
      if (!response) {
        if (message.method) onEvent(message);
        return;
      }
      pending.delete(message.id);
      clearTimeout(response.timer);
      if (message.error) response.reject(new Error(message.error.message));
      else response.resolveCall(message.result);
    });
    return { send, close };
  } catch (error) {
    await close();
    throw error;
  }
}

export async function openChrome({ executable = findChrome(), args = [], onEvent = () => {} } = {}) {
  if (!executable) throw new Error('Chrome is missing. Install Chrome or set CHROME_BIN.');
  try {
    return await connect(executable, args, onEvent);
  } catch (error) {
    console.warn(`Chrome startup failed (${error.message}). Retrying with a fresh profile.`);
    return connect(executable, args, onEvent);
  }
}
