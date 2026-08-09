import { safeStorage } from 'electron'
import type { KeyCipher } from './storage/apiKeyStore'

export const safeStorageCipher: KeyCipher = {
  isAvailable: () => safeStorage.isEncryptionAvailable(),
  encrypt: (plain: string) => safeStorage.encryptString(plain),
  decrypt: (buffer: Buffer) => safeStorage.decryptString(buffer)
}
