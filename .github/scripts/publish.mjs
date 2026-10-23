// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
// SPDX-License-Identifier: Apache-2.0

import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

export function run(command, args, options = {}) {
  return spawnSync(command, args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024, ...options });
}

function requireSuccess(result, description) {
  if (result.error || result.status !== 0) throw new Error(description);
  return result.stdout;
}

export function artifacts(plan, suffix) {
  const selected = (plan.releases ?? []).flatMap(release =>
    (release.artifacts ?? []).filter(name => name.endsWith(suffix))
      .map(name => ({ name, version: release.app_version })));
  if (!selected.length) throw new Error(`No ${suffix} artifacts in release plan`);
  for (const { name, version } of selected) {
    if (!/^[\w.+-]+$/.test(name) || !/^\d+\.\d+\.\d+(?:-[\w.-]+)?(?:\+[\w.-]+)?$/.test(version)) {
      throw new Error('Invalid release artifact name or version');
    }
  }
  return selected;
}

function publishedDist(name, version, execute) {
  const result = execute('npm', ['view', `${name}@${version}`, 'dist', '--json', '--registry=https://registry.npmjs.org']);
  if (result.status === 0 && !result.error) {
    const dist = JSON.parse(result.stdout);
    if (!dist?.integrity) throw new Error('Published npm integrity is missing');
    return dist;
  }
  // A connection/authentication failure must not be treated as an unpublished version.
  let code;
  try { code = JSON.parse(result.stdout).error?.code; } catch { /* no structured npm error */ }
  if (!result.error && code === 'E404') return null;
  throw new Error('Failed to check published npm package');
}

export function publishNpm(plan, directory, execute = run, log = console.log) {
  for (const { name: artifact, version } of artifacts(plan, '-npm-package.tar.gz')) {
    const archive = resolve(directory, artifact);
    const packed = JSON.parse(requireSuccess(
      execute('npm', ['pack', '--dry-run', '--ignore-scripts', '--json', archive]),
      'Failed to inspect npm archive'));
    const local = packed[0];
    if (packed.length !== 1 || local?.name !== 'maonline-cli' || local.version !== version || !local.integrity) {
      throw new Error('npm archive metadata does not match the release plan');
    }
    const published = publishedDist(local.name, version, execute);
    if (published) {
      if (published.integrity !== local.integrity) {
        throw new Error(`Published ${local.name}@${version} differs from the release artifact`);
      }
      log(`${local.name}@${version} already published with matching integrity; skipping`);
      continue;
    }
    requireSuccess(execute('npm', ['publish', '--access', 'public', '--registry=https://registry.npmjs.org', archive],
      { stdio: 'inherit' }), 'npm publish failed');
  }
}

function formulaIdentity(source) {
  const version = source.match(/^\s*version\s+["']([^"']+)["']/m)?.[1];
  const archives = [...source.matchAll(/\burl\s+["']([^"']+)["']\s+sha256\s+["']([a-f0-9]{64})["']/g)]
    .map(match => [match[1], match[2]]).sort((a, b) => a[0].localeCompare(b[0]));
  if (!version || archives.length === 0) throw new Error('Formula version or archive checksums are missing');
  return { version, archives };
}

export function publishHomebrew(plan, directory, tap, execute = run, log = console.log) {
  const formulae = artifacts(plan, '.rb');
  // Validate all same-version artifacts before modifying or pushing the tap.
  const updates = [];
  for (const { name, version } of formulae) {
    const source = resolve(directory, name);
    const target = resolve(tap, 'Formula', name);
    const incoming = formulaIdentity(readFileSync(source, 'utf8'));
    if (incoming.version !== version) throw new Error('Formula version does not match the release plan');
    if (existsSync(target)) {
      const previous = formulaIdentity(readFileSync(target, 'utf8'));
      if (previous.version === version && JSON.stringify(previous.archives) !== JSON.stringify(incoming.archives)) {
        throw new Error(`Published ${name}@${version} differs from the release artifact`);
      }
    }
    updates.push({ source, target, name, version });
  }
  const git = args => execute('git', args, { cwd: tap });
  requireSuccess(git(['config', 'user.name', 'github-actions[bot]']), 'Failed to configure git name');
  requireSuccess(git(['config', 'user.email', '41898282+github-actions[bot]@users.noreply.github.com']), 'Failed to configure git email');
  let changed = false;
  mkdirSync(resolve(tap, 'Formula'), { recursive: true });
  for (const { source, target, name, version } of updates) {
    copyFileSync(source, target);
    // Match cargo-dist formatting, retaining its tolerance of unrelated audit warnings.
    const style = execute('brew', ['style', '--except-cops', 'FormulaAudit/Homepage,FormulaAudit/Desc,FormulaAuditStrict', '--fix', target],
      { cwd: tap, stdio: 'inherit' });
    if (style.error) throw new Error('Failed to run brew style');
    requireSuccess(git(['add', '--', `Formula/${name}`]), 'Failed to stage formula');
    const diff = git(['diff', '--cached', '--quiet', '--', `Formula/${name}`]);
    if (!diff.error && diff.status === 0) {
      log(`${name}@${version} has no changes; skipping commit`);
      continue;
    }
    if (diff.error || diff.status !== 1) throw new Error('Failed to inspect staged formula');
    requireSuccess(git(['commit', '-m', `${name.slice(0, -3)} ${version}`, '--', `Formula/${name}`]), 'Failed to commit formula');
    changed = true;
  }
  if (changed) requireSuccess(git(['push']), 'Failed to push formula');
  else log('Homebrew tap is unchanged; skipping push');
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    const plan = JSON.parse(process.env.PLAN);
    if (process.argv[2] === 'npm') publishNpm(plan, process.argv[3]);
    else if (process.argv[2] === 'homebrew') publishHomebrew(plan, process.argv[3], process.argv[4]);
    else throw new Error('Usage: publish.mjs npm DIRECTORY | homebrew DIRECTORY TAP');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
