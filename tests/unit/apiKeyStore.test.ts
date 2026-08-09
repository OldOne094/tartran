import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { ApiKeyStore, type KeyCipher } from '../../src/main/storage/apiKeyStore'

class FakeCipher implements KeyCipher {
  available = true
  isAvailable(): boolean {
    return this.available
  }
  encrypt(plain: string): Buffer {
    return Buffer.from(`enc:${plain}`)
  }
  decrypt(buffer: Buffer): string {
    return buffer.toString('utf-8').slice(4)
  }
}

let base: string
let cipher: FakeCipher
let store: ApiKeyStore

beforeEach(() => {
  base = mkdtempSync(join(tmpdir(), 'tartran-keys-'))
  cipher = new FakeCipher()
  store = new ApiKeyStore(base, cipher)
})

afterEach(() => {
  rmSync(base, { recursive: true, force: true })
})

describe('ApiKeyStore', () => {
  it('stores and retrieves an encrypted key', () => {
    store.set('default', 'AIzaSySecretKey123')
    expect(store.has('default')).toBe(true)
    expect(store.get('default')).toBe('AIzaSySecretKey123')
  })

  it('deletes a key', () => {
    store.set('default', 'AIzaSySecretKey123')
    store.delete('default')
    expect(store.has('default')).toBe(false)
    expect(store.get('default')).toBeNull()
  })

  it('refuses to store when encryption is unavailable', () => {
    cipher.available = false
    expect(() => store.set('default', 'AIzaSySecretKey123')).toThrow('KEY_STORE_UNAVAILABLE')
  })

  it('persists across instances', () => {
    store.set('default', 'AIzaSySecretKey123')
    const reopened = new ApiKeyStore(base, new FakeCipher())
    expect(reopened.get('default')).toBe('AIzaSySecretKey123')
  })
})
