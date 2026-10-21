#!/usr/bin/env node
import assert from 'node:assert/strict';
import { appendFileSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const root = process.env.STATE_FIXTURE;
const fixture = JSON.parse(readFileSync(path.join(root, 'github.json'), 'utf8'));
const args = process.argv.slice(2);
assert.equal(args[0], 'api');
const endpoint = args.find((arg) => arg.startsWith('repos/'));
appendFileSync(path.join(root, 'calls'), `${args.includes('POST') ? 'POST' : 'GET'} ${endpoint}\n`);
if (args.includes('POST') && endpoint === 'repos/synthetic/private/deployments') {
  const request = JSON.parse(readFileSync(0, 'utf8'));
  writeFileSync(path.join(root, 'marker.json'), JSON.stringify(request));
  console.log(JSON.stringify({ id: 12, ...request }));
} else if (endpoint === 'repos/synthetic/private') console.log(JSON.stringify({ private: true }));
else if (endpoint.includes('/actions/artifacts?')) console.log(JSON.stringify({ artifacts: fixture.artifacts }));
else if (endpoint.includes('/deployments?')) console.log(JSON.stringify(fixture.deployments));
else if (endpoint === 'repos/synthetic/private/actions/runs/42') console.log(JSON.stringify({ path: fixture.workflow }));
else if (endpoint === 'repos/synthetic/private/actions/artifacts/1/zip') process.stdout.write(readFileSync(path.join(root, 'archive.zip')));
else throw new Error('Unexpected synthetic GitHub request');
