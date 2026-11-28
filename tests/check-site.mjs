import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, posix } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = fileURLToPath(new URL('../site/', import.meta.url));
const canonical = 'https://cleancodeshit.app/';
const contents = (file) => readFile(join(site, file), 'utf8');

function localAsset(value, page, allowData = false) {
  if (allowData && /^data:image\//i.test(value)) return;
  assert(value && !/^(?:[\w+.-]+:|\/\/|\\)/i.test(value), 'External asset in ' + page + ': ' + value);
  if (value.startsWith('#')) return;
  const pathname = new URL(value, new URL(page, canonical)).pathname;
  assert(existsSync(join(site, pathname)), 'Missing asset in ' + page + ': ' + value);
}

function attributes(tag) {
  return Object.fromEntries([...tag.matchAll(/([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))/g)]
    .map(([, name, a, b, c]) => [name.toLowerCase(), a ?? b ?? c]));
}

async function checkSources() {
  const [html, missing, css, js, manifest, cli] = await Promise.all([
    contents('index.html'), contents('404.html'), contents('assets/app.css'), contents('assets/app.js'),
    contents('site.webmanifest').then(JSON.parse), readFile(new URL('../cleancodeshit', import.meta.url), 'utf8'),
  ]);
  const version = cli.match(/^VERSION="([^"]+)"/m)?.[1];
  assert(version && html.includes('data-version="' + version + '"'), 'The site version must match the CLI VERSION');
  const withoutIcons = html.replace(/<svg\b[\s\S]*?<\/svg>/gi, '');
  const versions = [...withoutIcons.matchAll(/\bv?(\d+\.\d+\.\d+)\b/g)].map(([, value]) => value);
  assert(versions.length >= 5 && versions.every((v) => v === version), 'The page contains a stale release version');
  assert(html.includes('/releases/download/v' + version) && html.includes('/releases/tag/v' + version));
  assert(html.includes('<link rel="canonical" href="' + canonical + '">'));
  for (const attr of ['property="og:image"', 'name="twitter:image"']) {
    assert(html.includes(attr + ' content="' + canonical + 'og.png"'));
  }
  const og = await readFile(join(site, 'og.png'));
  assert(og.subarray(0, 8).equals(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10])));
  assert(og.readUInt32BE(16) === 1200 && og.readUInt32BE(20) === 630, 'OG image must be 1200x630');

  const items = [...cli.matchAll(/^[ \t]*add (\w+) +"([^"]+)" +(safe|caution|danger) +"([^"]+)"/gm)]
    .map((match) => match.slice(1));
  const registered = js.match(/^  var CLI_ITEMS = (\[[\s\S]*?\n  \]);$/m);
  assert(registered, 'Site CLI_ITEMS is missing');
  assert(items.length > 0 && new Set(items.map(([id]) => id)).size === items.length, 'Unexpected CLI registrations');
  assert.deepEqual(JSON.parse(registered[1]), items, 'Site item IDs, groups, risks, and labels must match the CLI');
  const itemIds = new Set(items.map(([id]) => id));
  for (const [, id] of js.matchAll(/cliId: '([^']+)'/g)) assert(itemIds.has(id), 'Unknown terminal item ' + id);

  for (const [page, source] of [['index.html', html], ['404.html', missing]]) {
    assert(!/<style\b/i.test(source), 'Inline style element in ' + page);
    for (const [, tag, text] of source.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)) {
      assert(attributes(tag).src && !text.trim(), 'Inline script in ' + page);
    }
    for (const [tag, name] of source.matchAll(/<([a-z][\w:-]*)\b[^>]*>/gi)) {
      const attrs = attributes(tag);
      assert(!Object.keys(attrs).some((key) => key === 'style' || /^on[a-z]+$/.test(key)), 'Inline handler or style in ' + page);
      for (const key of ['src', 'poster']) {
        if (key in attrs) localAsset(attrs[key], page, /^(img|source)$/i.test(name));
      }
      if ('href' in attrs && !/^(a|area)$/i.test(name) && attrs.rel !== 'canonical') {
        localAsset(attrs.href, page);
      }
      if (attrs['xlink:href']) localAsset(attrs['xlink:href'], page);
      if ('srcset' in attrs) {
        for (const part of attrs.srcset.split(',')) localAsset(part.trim().split(/\s+/)[0], page);
      }
    }
  }
  for (const [, a, b, c] of css.matchAll(/url\(\s*(?:"([^"]*)"|'([^']*)'|([^) \t]+))\s*\)/gi)) {
    localAsset(a ?? b ?? c, 'assets/app.css', true);
  }
  for (const [, a, b] of css.matchAll(/@import\s+(?:"([^"]+)"|'([^']+)')/gi)) {
    localAsset(a ?? b, 'assets/app.css');
  }
  for (const icon of manifest.icons) localAsset(icon.src, 'site.webmanifest');
  return items;
}

function headerRules(source) {
  const rules = [];
  for (const line of source.split(/\r?\n/)) {
    const value = line.trim();
    if (!value || value.startsWith('#')) continue;
    if (line.startsWith('/')) {
      const route = value.split('*').map((part) => part.replace(/[.*+?^$(){}|[\]\\]/g, '\\$&')).join('.*');
      rules.push({ route: new RegExp('^' + route + '$'), fields: [] });
    } else {
      const i = value.indexOf(':');
      rules.at(-1).fields.push(value.startsWith('!')
        ? [value.slice(1).trim().toLowerCase(), null]
        : [value.slice(0, i).toLowerCase(), value.slice(i + 1).trim()]);
    }
  }
  return rules;
}

function headersFor(rules, pathname) {
  const result = {};
  for (const { route, fields } of rules) {
    if (!route.test(pathname)) continue;
    for (const [key, value] of fields) {
      if (value === null) delete result[key];
      else result[key] = result[key] ? result[key] + ', ' + value : value;
    }
  }
  return result;
}

const types = {
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.js': 'text/javascript; charset=utf-8',
  '.woff2': 'font/woff2', '.svg': 'image/svg+xml', '.ico': 'image/x-icon', '.png': 'image/png',
  '.webmanifest': 'application/manifest+json', '.txt': 'text/plain; charset=utf-8', '.xml': 'application/xml',
};

async function serve(port = 0) {
  const rules = headerRules(await contents('_headers'));
  const server = createServer(async (request, response) => {
    let pathname;
    try {
      pathname = decodeURIComponent(new URL(request.url, canonical).pathname);
    } catch {
      response.writeHead(400).end();
      return;
    }
    const policy = headersFor(rules, pathname);
    let page = '404.html';
    let status = 404;
    if (pathname.startsWith('/') && !pathname.includes('\\') && !pathname.split('/').some((p) => /^[._]/.test(p))) {
      const requested = pathname.endsWith('/') ? posix.join(pathname, 'index.html')
        : extname(pathname) ? pathname : pathname + '.html';
      const resource = requested.slice(1);
      if (types[extname(resource)] && existsSync(join(site, resource))) {
        page = resource;
        status = 200;
      }
    }
    try {
      const file = await readFile(join(site, page));
      response.writeHead(status, {
        ...policy, 'content-type': types[extname(page)], 'content-length': file.length,
      });
      response.end(request.method === 'HEAD' ? undefined : file);
    } catch {
      response.writeHead(500, policy).end();
    }
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(port, '127.0.0.1', resolve);
  });
  return { server, origin: 'http://127.0.0.1:' + server.address().port };
}

async function checkHeaders(origin) {
  const root = await fetch(origin);
  assert.equal(root.status, 200);
  const policy = root.headers.get('content-security-policy');
  for (const part of ["default-src 'self'", "script-src 'self'", "style-src 'self'",
    "img-src 'self' data:", "frame-ancestors 'none'", "base-uri 'self'", "form-action 'none'"]) {
    assert(policy.split('; ').includes(part), 'Missing CSP directive ' + part);
  }
  assert(!policy.includes("'unsafe-inline'"));
  assert.equal(root.headers.get('x-content-type-options'), 'nosniff');
  assert.equal(root.headers.get('referrer-policy'), 'strict-origin-when-cross-origin');
  assert.equal(root.headers.get('permissions-policy'), 'camera=(), microphone=(), geolocation=()');
  assert.equal(root.headers.get('x-frame-options'), 'DENY');
  for (const [file, expectedStatus, cache] of [
    ['/', 200, 'public, max-age=0, must-revalidate'],
    ['/assets/app.css', 200, 'public, max-age=0, must-revalidate'],
    ['/assets/app.js', 200, 'public, max-age=0, must-revalidate'],
    ['/assets/brand/logo-dark.svg', 200, 'public, max-age=31536000, immutable'],
    ['/assets/fonts/archivo-black-latin-400-normal.woff2', 200, 'public, max-age=31536000, immutable'],
    ['/missing/nested/path', 404, 'public, max-age=0, must-revalidate'],
  ]) {
    const response = await fetch(origin + file);
    assert.equal(response.status, expectedStatus, file);
    assert.equal(response.headers.get('cache-control'), cache, file);
    assert.equal(response.headers.get('content-security-policy'), policy, file);
    if (expectedStatus === 404) assert((await response.text()).includes('href="/">Back to home</a>'));
    else await response.body.cancel();
  }
  await root.body.cancel();
}

function chromePath() {
  if (process.env.CHROME_BIN) return process.env.CHROME_BIN;
  if (process.platform === 'win32') return 'C:/Program Files/Google/Chrome/Application/chrome.exe';
  if (process.platform === 'darwin') return '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
  return 'google-chrome';
}

async function smoke(origin, items) {
  const profile = await mkdtemp(join(tmpdir(), 'ccs-site-smoke-'));
  const chrome = spawn(chromePath(), [
    '--headless=new', '--remote-debugging-port=0', '--user-data-dir=' + profile, '--no-first-run',
    '--disable-background-networking', 'about:blank',
  ], { stdio: ['ignore', 'ignore', 'pipe'], windowsHide: true });
  const exited = new Promise((resolve) => chrome.once('close', resolve));
  let ws;
  let send;
  try {
    const endpoint = await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('Chrome did not advertise a debugging endpoint')), 10000);
      chrome.once('error', (error) => { clearTimeout(timer); reject(error); });
      chrome.once('exit', (code) => { clearTimeout(timer); reject(new Error('Chrome exited ' + code)); });
      chrome.stderr.on('data', (chunk) => {
        const url = /DevTools listening on ws:\/\/([^/]+)/.exec(String(chunk));
        if (url) { clearTimeout(timer); resolve('http://' + url[1] + '/json/list'); }
      });
    });
    const targets = await (await fetch(endpoint)).json();
    ws = new WebSocket(targets.find((target) => target.type === 'page').webSocketDebuggerUrl);
    await new Promise((resolve, reject) => {
      ws.addEventListener('open', resolve, { once: true });
      ws.addEventListener('error', reject, { once: true });
    });
    let id = 0;
    let pageLoaded;
    const pending = new Map();
    const problems = [];
    ws.addEventListener('message', ({ data }) => {
      const message = JSON.parse(data);
      if (message.id && pending.has(message.id)) {
        const { resolve, reject, timer } = pending.get(message.id);
        pending.delete(message.id);
        clearTimeout(timer);
        if (message.error) reject(new Error(message.error.message));
        else resolve(message.result);
      }
      if (message.method === 'Page.loadEventFired') pageLoaded?.();
      if (message.method === 'Runtime.exceptionThrown') {
        problems.push(message.params.exceptionDetails.exception?.description ?? message.params.exceptionDetails.text);
      }
      if (message.method === 'Runtime.consoleAPICalled' && message.params.type === 'error') {
        problems.push(message.params.args.map((arg) => arg.value ?? arg.description).join(' '));
      }
      if (message.method === 'Log.entryAdded' && message.params.entry.level === 'error') problems.push(message.params.entry.text);
      if (message.method === 'Network.requestWillBeSent') {
        const url = message.params.request.url;
        if (/^https?:/.test(url) && new URL(url).origin !== origin) problems.push('External request: ' + url);
      }
    });
    ws.addEventListener('close', () => {
      for (const { reject, timer } of pending.values()) {
        clearTimeout(timer);
        reject(new Error('Chrome closed the debugging connection'));
      }
      pending.clear();
    });
    send = (method, params = {}) => new Promise((resolve, reject) => {
      const next = ++id;
      const timer = setTimeout(() => {
        pending.delete(next);
        reject(new Error(method + ' timed out'));
      }, 7000);
      pending.set(next, { resolve, reject, timer });
      ws.send(JSON.stringify({ id: next, method, params }));
    });
    for (const domain of ['Page', 'Runtime', 'Log', 'Network']) await send(domain + '.enable');
    await send('Page.addScriptToEvaluateOnNewDocument', {
      source: 'window.siteViolations = []; addEventListener("securitypolicyviolation", event => siteViolations.push(event.violatedDirective));',
    });
    for (const file of ['/index.html', '/404.html']) {
      const ready = new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('Page load timed out for ' + file)), 6000);
        pageLoaded = () => { clearTimeout(timeout); resolve(); };
      });
      await Promise.all([ready, send('Page.navigate', { url: origin + file })]);
      const { result, exceptionDetails } = await send('Runtime.evaluate', {
        expression: 'document.fonts.ready.then(() => ({ heading: document.querySelector("h1")?.textContent.trim(), home: document.querySelector("main a")?.getAttribute("href"), items: Array.from(document.querySelectorAll("#bins li[data-item-id]"), el => [el.dataset.itemId, el.children[0].textContent, el.children[1].textContent]), violations: siteViolations }))',
        awaitPromise: true, returnByValue: true,
      });
      assert(!exceptionDetails, JSON.stringify(exceptionDetails));
      assert.deepEqual(result.value.violations, [], 'CSP violation on ' + file);
      if (file === '/index.html') {
        assert(result.value.heading && result.value.items.length === items.length);
        const compare = (list) => list.sort((a, b) => a[0].localeCompare(b[0]));
        assert.deepEqual(compare(result.value.items), compare(items.map(([id, , risk, label]) => [id, label, risk])));
      } else {
        assert.equal(result.value.heading, 'Page not found.');
        assert.equal(result.value.home, '/');
      }
      assert.deepEqual(problems, [], file);
    }
  } finally {
    if (ws?.readyState === WebSocket.OPEN && send) {
      await Promise.race([send('Browser.close').catch(() => {}), exited]);
    }
    ws?.close();
    if (chrome.exitCode === null && chrome.pid) chrome.kill();
    await exited;
    await rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
  }
}

if (process.argv[2] === '--serve') {
  const { server, origin } = await serve(Number(process.argv[3] || 0));
  console.log(origin);
  for (const name of ['SIGINT', 'SIGTERM']) {
    process.once(name, () => {
      server.close();
      server.closeAllConnections();
    });
  }
} else {
  const items = await checkSources();
  const { server, origin } = await serve();
  try {
    await checkHeaders(origin);
    await smoke(origin, items);
    console.log('Site checks passed. ' + items.length + ' CLI registrations. Index and 404 loaded without console errors or CSP violations.');
  } finally {
    server.close();
    server.closeAllConnections();
  }
}
