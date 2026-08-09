import { mkdirSync, readFileSync, writeFileSync, renameSync } from 'node:fs'
import { join } from 'node:path'

export interface KeyCipher {
  isAvailable(): boolean
  encrypt(plain: string): Buffer
  decrypt(buffer: Buffer): string
}

const KEY_FILE = 'api-keys.json'

function writeJsonAtomic(file: string, value: unknown): void {
  const tmp = `${file}.tmp`
  writeFileSync(tmp, JSON.stringify(value, null, 2), 'utf-8')
  renameSync(tmp, file)
}

export class ApiKeyStore {
  private readonly file: string

  constructor(baseDir: string, private readonly cipher: KeyCipher) {
    mkdirSync(baseDir, { recursive: true })
    this.file = join(baseDir, KEY_FILE)
  }

  private load(): Record<string, string> {
    try {
      return JSON.parse(readFileSync(this.file, 'utf-8')) as Record<string, string>
    } catch {
      return {}
    }
  }

  has(id: string): boolean {
    return id in this.load()
  }

  set(id: string, plain: string): void {
    if (!this.cipher.isAvailable()) {
      throw new Error('KEY_STORE_UNAVAILABLE')
    }
    const map = this.load()
    map[id] = this.cipher.encrypt(plain).toString('base64')
    writeJsonAtomic(this.file, map)
  }

  get(id: string): string | null {
    const map = this.load()
    const raw = map[id]
    if (!raw || !this.cipher.isAvailable()) return null
    try {
      return this.cipher.decrypt(Buffer.from(raw, 'base64'))
    } catch {
      return null
    }
  }

  delete(id: string): void {
    const map = this.load()
    delete map[id]
    writeJsonAtomic(this.file, map)
  }
}
