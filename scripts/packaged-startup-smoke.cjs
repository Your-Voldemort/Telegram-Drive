#!/usr/bin/env node
// Run only in a disposable OS user/VM. Installer/keyring state is not redirected
// merely by HOME; the app also uses a unique smoke-only runtime identifier.
const fs = require('node:fs'), path = require('node:path'), os = require('node:os');
const { spawn, spawnSync } = require('node:child_process');
const { randomUUID } = require('node:crypto');
async function boundedWait(promise, milliseconds) {
  let timer;
  try { return await Promise.race([promise, new Promise(resolve => { timer = setTimeout(resolve, milliseconds); })]); }
  finally { clearTimeout(timer); }
}
function belongsToLaunch(pid, launched) {
  if (pid === launched) return true;
  if (process.platform === 'win32' || !Number.isSafeInteger(pid) || pid < 2) return false;
  for (let depth = 0; depth < 16; depth++) {
    const parent = spawnSync('ps', ['-o', 'ppid=', '-p', String(pid)], { encoding: 'utf8', timeout: 2000 });
    if (parent.status !== 0) return false;
    pid = Number(parent.stdout.trim()); if (pid === launched) return true; if (pid < 2) return false;
  }
  return false;
}
async function main() {
  const options = { timeout: 45_000, args: [] };
  for (let i = 2; i < process.argv.length; i++) {
    const flag = process.argv[i];
    if (flag === '--disposable-user') options.disposable = true;
    else if (flag === '--executable') options.executable = process.argv[++i];
    else if (flag === '--args-json') options.args = JSON.parse(process.argv[++i]);
    else if (flag === '--timeout-ms') options.timeout = Number(process.argv[++i]);
    else throw new Error(`Unknown flag ${flag}`);
  }
  if (!options.disposable && process.env.GITHUB_ACTIONS !== 'true') throw new Error('Packaged smoke requires a disposable OS user/VM: confirm with --disposable-user');
  if (!Array.isArray(options.args) || options.args.some(value => typeof value !== 'string') || !Number.isInteger(options.timeout) || options.timeout < 100 || options.timeout > 60_000) throw new Error('Invalid smoke arguments/deadline');
  const executable = fs.realpathSync(options.executable);
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'telegram-drive-packaged-smoke-'));
  fs.chmodSync(root, 0o700);
  const token = randomUUID().replaceAll('-', ''), identifier = `com.cameronamer.telegramdrive.smoke.${token}`;
  fs.writeFileSync(path.join(root, '.packaged-startup-smoke'), `${token}\n`, { mode: 0o600 });
  const environment = { ...process.env, TELEGRAM_DRIVE_STARTUP_SMOKE_ROOT: root, TELEGRAM_DRIVE_STARTUP_SMOKE_TOKEN: token, TELEGRAM_DRIVE_SMOKE_DISPOSABLE_USER: '1',
    HOME: path.join(root, 'home'), USERPROFILE: path.join(root, 'home'), APPDATA: path.join(root, 'roaming'), LOCALAPPDATA: path.join(root, 'local'),
    XDG_DATA_HOME: path.join(root, 'data'), XDG_CONFIG_HOME: path.join(root, 'config'), XDG_CACHE_HOME: path.join(root, 'cache'), TMPDIR: path.join(root, 'tmp'), TMP: path.join(root, 'tmp'), TEMP: path.join(root, 'tmp') };
  for (const directory of ['home', 'roaming', 'local', 'data', 'config', 'cache', 'tmp']) fs.mkdirSync(path.join(root, directory));
  const child = spawn(executable, options.args, { cwd: path.dirname(executable), env: environment, stdio: ['ignore', 'pipe', 'pipe'], detached: process.platform !== 'win32' });
  let output = '', exited = false, ready, failure;
  child.stdout.on('data', data => { output = (output + data).slice(-16_384); }); child.stderr.on('data', data => { output = (output + data).slice(-16_384); });
  const completion = new Promise(resolve => { child.once('error', error => { failure = error; exited = true; resolve(); }); child.once('close', () => { exited = true; resolve(); }); });
  try {
    const deadline = Date.now() + options.timeout;
    while (Date.now() < deadline && !exited) {
      const marker = path.join(root, 'startup-ready.json');
      if (fs.existsSync(marker)) {
        const value = JSON.parse(fs.readFileSync(marker, 'utf8'));
        if (!belongsToLaunch(value.process_id, child.pid) || value.run_token !== token || value.profile_identifier !== identifier || !value.database_ready || !value.app_data_ready || !value.streaming_runtime_ready) throw new Error('Invalid or stale application readiness marker');
        for (const name of ['app_data_dir', 'app_cache_dir', 'app_config_dir', 'app_local_data_dir']) {
          if (typeof value[name] !== 'string' || path.basename(value[name]) !== identifier) throw new Error(`Readiness ${name} is not the isolated Tauri profile`);
        }
        ready = value; break;
      }
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    if (!ready || exited) throw failure || new Error(`Application readiness not reached${exited ? ' before early exit' : ' before deadline'}\n${output}`);
    console.log(`[packaged-smoke] Startup readiness passed for ${path.basename(executable)} (${ready.version}), isolated profile ${identifier}.`);
  } finally {
    // Kill the complete process tree and wait for the launch process to be reaped.
    if (child.pid) {
      if (process.platform === 'win32') spawnSync('taskkill', ['/PID', String(child.pid), '/T', '/F'], { encoding: 'utf8', timeout: 10_000 });
      else { try { process.kill(-child.pid, 'SIGTERM'); } catch {} }
    }
    await boundedWait(completion, 2_000);
    if (process.platform !== 'win32' && child.pid) { try { process.kill(-child.pid, 'SIGKILL'); } catch {} }
    await boundedWait(completion, 5_000);
    if (!exited) throw new Error('Smoke process could not be reaped');
    // Windows/macOS known-folder APIs may choose locations outside the temporary
    // HOME. Remove only the exact unique profile paths returned by readiness.
    if (ready) for (const name of ['app_data_dir', 'app_cache_dir', 'app_config_dir', 'app_local_data_dir']) {
      const directory = ready[name]; if (path.basename(directory) === identifier) fs.rmSync(directory, { recursive: true, force: true });
    }
    fs.rmSync(root, { recursive: true, force: true });
  }
}
main().catch(error => { console.error(`[packaged-smoke] ${error.message}`); process.exitCode = 1; });
