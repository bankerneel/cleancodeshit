import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { bumpSiteVersions, checkSiteVersions, cliVersion, siteSources, stableVersion } from './site-version.mjs';
import { renderReadmeShot } from '../tools/readme-shot/render.mjs';

const [operation, argument, notesFile] = process.argv.slice(2);
const read = (file) => readFileSync(file, 'utf8');

function sections(changelog) {
  const headings = [...changelog.matchAll(/^## \[([^\]\n]+)\][^\n]*$/gm)];
  return headings.map((heading, index) => ({
    name: heading[1], heading: heading[0], start: heading.index,
    notes: changelog.slice(heading.index + heading[0].length, headings[index + 1]?.index).trim(),
  }));
}

function hasNotes(notes) {
  return notes.replace(/<!--[\s\S]*?-->/g, '').split('\n').some((line) => line.trim() && !/^\s*#+\s/.test(line));
}

function isNewer(requested, current) {
  const newParts = requested.split('.').map(BigInt);
  const oldParts = current.split('.').map(BigInt);
  const index = newParts.findIndex((part, i) => part !== oldParts[i]);
  return index !== -1 && newParts[index] > oldParts[index];
}

function checksum(source) {
  return `${createHash('sha256').update(source).digest('hex')}  cleancodeshit\n`;
}

function verify(version, cli, changelog, sources) {
  assert.equal(argument, `v${version}`, `Tag must equal v${version}`);
  const latest = sections(changelog).find(({ name }) => name !== 'Unreleased');
  assert(latest?.name === version && /^## \[\d+\.\d+\.\d+\] - \d{4}-\d{2}-\d{2}$/.test(latest.heading)
    && hasNotes(latest.notes), `CHANGELOG.md must have a dated ${version} section with release notes`);
  assert.equal(read('cleancodeshit.sha256'), checksum(cli), 'The CLI checksum is stale');
  checkSiteVersions(sources, version);
  writeFileSync(notesFile, latest.notes + '\n');
  console.log(`Verified ${argument}, its release notes, CLI checksum, and site version.`);
}

async function bump(current, cli, changelog, sources) {
  assert(stableVersion.test(argument), 'Supply a stable SemVer in X.Y.Z form.');
  assert(isNewer(argument, current), `Version must be greater than ${current}.`);
  const [unreleased, ...otherSections] = sections(changelog);
  assert(unreleased?.heading === '## [Unreleased]' && hasNotes(unreleased.notes)
    && !otherSections.some(({ name }) => name === 'Unreleased'), 'CHANGELOG.md needs a non-empty ## [Unreleased] section above the releases.');
  assert(!otherSections.some(({ name }) => name === argument), `${argument} is already in CHANGELOG.md.`);
  checkSiteVersions(sources, current);

  const version = argument;
  const today = new Date();
  const date = [today.getFullYear(), today.getMonth() + 1, today.getDate()]
    .map((part) => String(part).padStart(2, '0')).join('-');
  const updatedCli = cli.replace(/^VERSION="[^"]+"$/m, `VERSION="${version}"`);
  const changed = new Map([
    ['cleancodeshit', updatedCli],
    ['cleancodeshit.sha256', checksum(updatedCli)],
    ['CHANGELOG.md', changelog.slice(0, unreleased.start) + `## [${version}] - ${date}`
      + changelog.slice(unreleased.start + unreleased.heading.length)],
  ]);
  const currentToken = new RegExp(`(?<![\\w.])(v?)${current.replaceAll('.', '\\.')}\\b(?!\\.\\d)`, 'g');
  for (const file of ['README.md', 'docs/items.md', 'packaging/homebrew/cleancodeshit.rb.in']) {
    const before = read(file);
    const after = before.replace(currentToken, (_, prefix) => prefix + version);
    assert.notEqual(after, before, `${file} is missing ${current}`);
    changed.set(file, after);
  }
  const security = read('SECURITY.md');
  const supported = `| ${current} | Supported, latest release |`;
  assert(security.includes(supported), `SECURITY.md is missing the supported ${current} row`);
  changed.set('SECURITY.md', security.replace(supported, `| ${version} | Supported, latest release |\n`
    + `| ${current} | Not supported, superseded by ${version} |`));

  for (const [file, before] of sources) {
    const after = bumpSiteVersions(before, version);
    sources.set(file, after);
    if (after !== before) changed.set(join('site', file), after);
  }
  checkSiteVersions(sources, version);

  const screenshot = await renderReadmeShot(updatedCli);
  if (screenshot) changed.set('docs/sample-terminal.png', screenshot);
  for (const [file, content] of changed) writeFileSync(file, content);

  console.log(`Prepared ${current} -> ${version}:\n${[...changed.keys()].map((path) => `  ${path}`).join('\n')}`);
  console.log(`\nReview the diff, then run:\n  git add -u\n  git commit -m "Release ${version}"\n  git push origin main\n  git tag v${version}\n  git push origin v${version}`);
}

try {
  const cli = read('cleancodeshit');
  const version = cliVersion(cli);
  const changelog = read('CHANGELOG.md');
  const sources = siteSources('site');
  if (operation === 'verify' && notesFile) verify(version, cli, changelog, sources);
  else if (operation === 'bump') await bump(version, cli, changelog, sources);
  else throw new Error('Expected bump X.Y.Z or verify vX.Y.Z NOTES_FILE.');
} catch (error) {
  console.error(`Release: ${error.message}`);
  process.exitCode = 1;
}
