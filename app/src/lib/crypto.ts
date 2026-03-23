/// End-to-end encryption for message payloads.
///
/// Uses the Web Crypto API (SubtleCrypto) for all operations.
/// The relay never sees plaintext — it only forwards encrypted blobs.
///
/// Key exchange: X25519 ECDH (via P-256 on Web Crypto as X25519 isn't
/// universally available). Each user generates a key pair. Shared secrets
/// are derived per-conversation via ECDH + HKDF.
///
/// Message encryption: AES-256-GCM with a random 12-byte IV per message.
/// The IV is prepended to the ciphertext.
///
/// Key storage: Key pairs are stored in IndexedDB (via localForage or
/// direct IDB). Never exported, never leave the device.

const ALGO = 'AES-GCM'
const KEY_LENGTH = 256
const IV_LENGTH = 12

/** Generate a new AES-256-GCM encryption key. */
export async function generateKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey(
    { name: ALGO, length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt'],
  )
}

/** Export a CryptoKey to a base64 string for storage/transmission. */
export async function exportKey(key: CryptoKey): Promise<string> {
  const raw = await crypto.subtle.exportKey('raw', key)
  return arrayBufferToBase64(raw)
}

/** Import a base64-encoded key string back to a CryptoKey. */
export async function importKey(base64: string): Promise<CryptoKey> {
  const raw = base64ToArrayBuffer(base64)
  return crypto.subtle.importKey(
    'raw',
    raw,
    { name: ALGO, length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt'],
  )
}

/**
 * Encrypt a plaintext string with AES-256-GCM.
 * Returns a base64 string containing IV + ciphertext.
 */
export async function encrypt(plaintext: string, key: CryptoKey): Promise<string> {
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH))
  const encoded = new TextEncoder().encode(plaintext)

  const ciphertext = await crypto.subtle.encrypt(
    { name: ALGO, iv },
    key,
    encoded,
  )

  // Prepend IV to ciphertext
  const combined = new Uint8Array(IV_LENGTH + ciphertext.byteLength)
  combined.set(iv, 0)
  combined.set(new Uint8Array(ciphertext), IV_LENGTH)

  return arrayBufferToBase64(combined.buffer)
}

/**
 * Decrypt a base64 string (IV + ciphertext) with AES-256-GCM.
 * Returns the plaintext string.
 */
export async function decrypt(encrypted: string, key: CryptoKey): Promise<string> {
  const combined = new Uint8Array(base64ToArrayBuffer(encrypted))

  const iv = combined.slice(0, IV_LENGTH)
  const ciphertext = combined.slice(IV_LENGTH)

  const decrypted = await crypto.subtle.decrypt(
    { name: ALGO, iv },
    key,
    ciphertext,
  )

  return new TextDecoder().decode(decrypted)
}

/**
 * Derive a shared encryption key from an ECDH key exchange.
 * Uses P-256 + HKDF to produce an AES-256-GCM key.
 */
export async function deriveSharedKey(
  privateKey: CryptoKey,
  publicKey: CryptoKey,
  salt: Uint8Array = new Uint8Array(32),
): Promise<CryptoKey> {
  // ECDH shared secret
  const sharedSecret = await crypto.subtle.deriveBits(
    { name: 'ECDH', public: publicKey },
    privateKey,
    256,
  )

  // Import as HKDF key material
  const hkdfKey = await crypto.subtle.importKey(
    'raw',
    sharedSecret,
    'HKDF',
    false,
    ['deriveKey'],
  )

  // Derive AES-256-GCM key via HKDF
  return crypto.subtle.deriveKey(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt,
      info: new TextEncoder().encode('exom-e2e-v1'),
    },
    hkdfKey,
    { name: ALGO, length: KEY_LENGTH },
    true,
    ['encrypt', 'decrypt'],
  )
}

/** Generate an ECDH P-256 key pair for key exchange. */
export async function generateKeyPair(): Promise<CryptoKeyPair> {
  return crypto.subtle.generateKey(
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    ['deriveBits'],
  )
}

/** Export an ECDH public key to base64 for transmission. */
export async function exportPublicKey(key: CryptoKey): Promise<string> {
  const raw = await crypto.subtle.exportKey('raw', key)
  return arrayBufferToBase64(raw)
}

/** Import an ECDH public key from base64. */
export async function importPublicKey(base64: string): Promise<CryptoKey> {
  const raw = base64ToArrayBuffer(base64)
  return crypto.subtle.importKey(
    'raw',
    raw,
    { name: 'ECDH', namedCurve: 'P-256' },
    true,
    [],
  )
}

// ── Base64 utilities ────────────────────────────

function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i])
  }
  return btoa(binary)
}

function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes.buffer
}
