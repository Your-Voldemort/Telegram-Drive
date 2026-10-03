// Opt-in measurement through the native child-process/storage boundary.
// Writes exactly 2 GB of synthetic data; no sparse file and no OS application launch.
const assert = require('node:assert/strict');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const { tmpdir } = require('node:os');
const { resolve, join } = require('node:path');
const { createInterface } = require('node:readline');
const { performance } = require('node:perf_hooks');
const root = fs.mkdtempSync(join(tmpdir(), 'telegram-drive-opening-measure-'));
fs.chmodSync(root, 0o700);
fs.writeFileSync(join(root, '.native-e2e-fixture'), 'telegram-drive-synthetic-e2e\n');
let driver;
(async () => {
  try {
    const size = 2_000_000_000;
    const block = Buffer.alloc(4 * 1024 * 1024);
    for (let i = 0; i < block.length; i++) block[i] = i % 251;
    const fd = fs.openSync(join(root, 'synthetic.bin'), 'wx', 0o600);
    try {
      for (let done = 0; done < size;) {
        const bytes = Math.min(block.length, size - done);
        done += fs.writeSync(fd, block, 0, bytes);
      }
      fs.fsyncSync(fd);
    } finally { fs.closeSync(fd); }
    assert.equal(fs.statSync(join(root, 'synthetic.bin')).size, size);
    driver = spawn(resolve(__dirname, '../app/src-tauri/target/debug/native-e2e-driver'), [root], { stdio: ['pipe', 'pipe', 'inherit'] });
    const replies = createInterface({ input: driver.stdout })[Symbol.asyncIterator]();
    assert.deepEqual(JSON.parse((await replies.next()).value), { ready: true });
    async function request(command) {
      driver.stdin.write(`${JSON.stringify(command)}\n`);
      const reply = JSON.parse((await replies.next()).value);
      assert.equal(reply.ok, true, JSON.stringify(reply));
      return reply.value;
    }
    await request({ command: 'seed_account', owner: 101 });
    const file = await request({ command: 'legacy_external_seed', owner: '101', source: 'synthetic.bin', category: 'offline' });
    const measurements = [];
    for (const label of ['first legacy open', 'subsequent open']) {
      const before = await request({ command: 'external_file_status' });
      const start = performance.now();
      await request({ command: 'external_file_open', path: file.path });
      const seconds = (performance.now() - start) / 1000;
      const after = await request({ command: 'external_file_status' });
      const hashedBytes = after[1] - before[1];
      assert.equal(hashedBytes, size * (label === 'first legacy open' ? 2 : 1));
      measurements.push({ label, seconds, hashedBytes });
    }
    console.log(JSON.stringify({ bytes: size, platform: process.platform, cache: 'warm local synthetic file', driver: 'debug native E2E; release performance unmeasured', measurements }, null, 2));
    driver.stdin.end(`${JSON.stringify({ command: 'shutdown' })}\n`);
    await new Promise(done => driver.exitCode !== null ? done() : driver.once('exit', done));
  } finally {
    if (driver && driver.exitCode === null) driver.kill();
    fs.rmSync(root, { recursive: true, force: true });
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
