import { spawnSync } from 'node:child_process'
import { copyFileSync, existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = dirname(fileURLToPath(import.meta.url))
const rootDir = resolve(scriptDir, '..')
const sourceIcon = resolve(rootDir, process.argv[2] ?? 'UMBRA1.png')
const publicDir = resolve(rootDir, 'public')
const tauriIconsDir = resolve(rootDir, 'src-tauri', 'icons')
const webTempDir = mkdtempSync(join(tmpdir(), 'umbra-web-icons-'))
const tauriBinary = resolve(
  rootDir,
  'node_modules',
  '.bin',
  process.platform === 'win32' ? 'tauri.cmd' : 'tauri'
)

function run(command, args, cwd = rootDir) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit' })
  if (result.error) {
    console.error(result.error.message)
    process.exit(1)
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1)
  }
}

function runTauriIcon(args) {
  if (process.platform === 'win32') {
    const quote = (value) => `'${String(value).replace(/'/g, "''")}'`
    const commandLine = ['&', quote(tauriBinary), 'icon', quote(sourceIcon), ...args.map(quote)].join(' ')
    run('powershell.exe', ['-NoProfile', '-Command', commandLine])
    return
  }

  run(tauriBinary, ['icon', sourceIcon, ...args])
}

if (!existsSync(sourceIcon)) {
  console.error(`missing icon source: ${sourceIcon}`)
  process.exit(1)
}

try {
  runTauriIcon(['-o', tauriIconsDir])
  runTauriIcon(['-o', webTempDir, '--png', '32', '--png', '48', '--png', '180'])

  copyFileSync(join(webTempDir, '32x32.png'), join(publicDir, 'favicon-32x32.png'))
  copyFileSync(join(webTempDir, '48x48.png'), join(publicDir, 'favicon-48x48.png'))
  copyFileSync(join(webTempDir, '180x180.png'), join(publicDir, 'apple-touch-icon.png'))
  copyFileSync(join(tauriIconsDir, 'icon.ico'), join(publicDir, 'favicon.ico'))

  console.log(`synced web + tauri icons from ${sourceIcon}`)
} finally {
  rmSync(webTempDir, { recursive: true, force: true })
}
