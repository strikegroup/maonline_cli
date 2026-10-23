// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
// SPDX-License-Identifier: Apache-2.0

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { artifacts, publishNpm, publishHomebrew, run } from './publish.mjs';

const version = '0.0.2';
const plan = name => ({ releases: [{ app_version: version, artifacts: [name] }] });
const ok = stdout => ({ status: 0, stdout: stdout ?? '' });
const npmPlan = plan('maonline_cli-npm-package.tar.gz');
const local = { name: 'maonline-cli', version, integrity: 'sha512-test' };
const silent = () => {};

function fakeNpm(view, publish = ok()) {
  const calls = [];
  return { calls, execute(command, args) {
    calls.push([command, ...args]);
    assert.equal(command, 'npm');
    if (args[0] === 'pack') return ok(JSON.stringify([local]));
    if (args[0] === 'view') return view;
    if (args[0] === 'publish') return publish;
    throw new Error('Unexpected npm operation');
  } };
}

test('npm matching published integrity skips publication', () => {
  const fake = fakeNpm(ok(JSON.stringify({ integrity: local.integrity })));
  publishNpm(npmPlan, '.', fake.execute, silent);
  assert.deepEqual(fake.calls.map(call => call[1]), ['pack', 'view']);
});

test('npm differing published integrity fails without publication', () => {
  const fake = fakeNpm(ok(JSON.stringify({ integrity: 'sha512-different' })));
  assert.throws(() => publishNpm(npmPlan, '.', fake.execute, silent), /differs/);
  assert.equal(fake.calls.length, 2);
});

test('npm an unpublished version is published once and failures propagate', () => {
  const missing = { status: 1, stdout: JSON.stringify({ error: { code: 'E404' } }) };
  const fake = fakeNpm(missing);
  publishNpm(npmPlan, '.', fake.execute, silent);
  assert.deepEqual(fake.calls.map(call => call[1]), ['pack', 'view', 'publish']);
  const failing = fakeNpm(missing, { status: 1 });
  assert.throws(() => publishNpm(npmPlan, '.', failing.execute, silent), /npm publish failed/);
});

test('npm authentication, network and malformed registry responses never trigger publish', () => {
  for (const view of [
    { status: 1, stdout: JSON.stringify({ error: { code: 'E401' } }) },
    { status: 1, stdout: JSON.stringify({ error: { code: 'ETIMEDOUT' } }) },
    { status: null, error: new Error('failed to start'), stdout: '' },
    ok('{}'), ok('invalid json'),
  ]) {
    const fake = fakeNpm(view);
    assert.throws(() => publishNpm(npmPlan, '.', fake.execute, silent));
    assert.equal(fake.calls.length, 2);
  }
});

test('npm rejects archive metadata that does not match the plan', () => {
  assert.throws(() => publishNpm(npmPlan, '.', () => ok(JSON.stringify([{ ...local, version: '9.0.0' }])), silent), /metadata/);
});

test('missing artifacts, unsafe paths and invalid versions fail', () => {
  assert.throws(() => artifacts({ releases: [] }, '.rb'), /No .rb/);
  assert.throws(() => artifacts(plan('../maonline.rb'), '.rb'), /Invalid/);
  assert.throws(() => artifacts({ releases: [{ app_version: 'unsafe', artifacts: ['maonline.rb'] }] }, '.rb'), /Invalid/);
});

function formula(release, hash = 'a'.repeat(64)) {
  return `class Maonline < Formula\n  version "${release}"\n  url "https://example.com/${release}.tar.xz"\n  sha256 "${hash}"\nend\n`;
}

function tapFixture(context, source, previous) {
  const directory = mkdtempSync(join(tmpdir(), 'maonline-publish-'));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const tap = join(directory, 'tap');
  const incoming = join(directory, 'incoming');
  mkdirSync(join(tap, 'Formula'), { recursive: true });
  mkdirSync(incoming);
  writeFileSync(join(incoming, 'maonline.rb'), source);
  const git = args => {
    const result = run('git', args, { cwd: tap });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  };
  git(['init', '--quiet']);
  git(['config', 'user.name', 'test']);
  git(['config', 'user.email', 'test@example.com']);
  // Ignore machine-wide signing and hooks in this isolated test repository.
  git(['config', 'commit.gpgsign', 'false']);
  git(['config', 'core.hooksPath', '/dev/null']);
  writeFileSync(join(tap, 'Formula', 'maonline.rb'), previous);
  git(['add', '.']);
  git(['commit', '--quiet', '-m', 'initial']);
  let pushes = 0;
  const execute = (command, args, options) => {
    if (command === 'brew') return ok();
    if (command === 'git' && args[0] === 'push') { pushes++; return ok(); }
    return run(command, args, options);
  };
  return { tap, incoming, execute, git, pushes: () => pushes };
}

test('Homebrew unchanged formula skips commit and push using a real git repository', context => {
  const fixture = tapFixture(context, formula(version), formula(version));
  publishHomebrew(plan('maonline.rb'), fixture.incoming, fixture.tap, fixture.execute, silent);
  assert.equal(fixture.git(['rev-list', '--count', 'HEAD']).trim(), '1');
  assert.equal(fixture.pushes(), 0);
});

test('Homebrew updates once and rerunning the same release skips commit and push', context => {
  const fixture = tapFixture(context, formula(version), formula('0.0.1'));
  for (let i = 0; i < 2; i++) {
    publishHomebrew(plan('maonline.rb'), fixture.incoming, fixture.tap, fixture.execute, silent);
  }
  assert.equal(fixture.git(['rev-list', '--count', 'HEAD']).trim(), '2');
  assert.equal(fixture.pushes(), 1);
  assert.equal(fixture.git(['status', '--porcelain']).trim(), '');
});

test('Homebrew rejects different archive hashes for the same published version', context => {
  const fixture = tapFixture(context, formula(version, 'b'.repeat(64)), formula(version));
  assert.throws(() => publishHomebrew(plan('maonline.rb'), fixture.incoming, fixture.tap, fixture.execute, silent), /differs/);
  assert.equal(fixture.pushes(), 0);
  assert.equal(fixture.git(['status', '--porcelain']).trim(), '');
});
