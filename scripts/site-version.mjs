import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';

export const stableVersion = /^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/;
const versionToken = /(?<![\w.])v?(\d+\.\d+\.\d+(?:-[a-z\d-]+(?:\.[a-z\d-]+)*)?(?:\+[a-z\d-]+(?:\.[a-z\d-]+)*)?)(?!\w|\.\d)/gi;
// SVG path coordinates may put consecutive decimals together without separators.
const svgPath = /((?:<|%3c)path\b(?:(?!%3e)[^>])*?\sd\s*=\s*)("[^"]*"|'[^']*')/gi;

export function cliVersion(cli) {
  const version = cli.match(/^VERSION="([^"]+)"$/m)?.[1];
  assert(version && stableVersion.test(version), 'Expected a stable CLI VERSION');
  return version;
}

export function siteSources(directory) {
  const sources = new Map();
  for (const entry of readdirSync(directory, { recursive: true, withFileTypes: true })) {
    if (!entry.isFile()) continue;
    const file = join(entry.parentPath, entry.name);
    const bytes = readFileSync(file);
    if (bytes.includes(0)) continue;
    try {
      sources.set(relative(directory, file), new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(bytes));
    } catch {
      // Binary assets do not contain text version references.
    }
  }
  return sources;
}

function versionText(source, transform) {
  const rewrite = (text) => text.replace(versionToken, transform);
  let last = 0;
  let result = '';
  for (const attribute of source.matchAll(svgPath)) {
    const coordinates = attribute[2];
    const start = attribute.index + attribute[0].length - coordinates.length;
    result += rewrite(source.slice(last, start)) + coordinates;
    last = start + coordinates.length;
  }
  return result + rewrite(source.slice(last));
}

export function checkSiteVersions(sources, version) {
  const index = sources.get('index.html') ?? '';
  for (const required of [`data-version="${version}"`, `/releases/download/v${version}`, `/releases/tag/v${version}`]) {
    assert(index.includes(required), `site/index.html is missing ${required}`);
  }
  for (const [file, source] of sources) {
    versionText(source, (token, value) => {
      assert.equal(value, version, `site/${file} contains ${token}, expected ${version}`);
      return token;
    });
  }
}

export function bumpSiteVersions(source, version) {
  return versionText(source, (token, value) => token.slice(0, token.length - value.length) + version);
}
