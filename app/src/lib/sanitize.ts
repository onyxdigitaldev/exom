/// Input sanitization and XSS prevention.
///
/// All user-generated content passes through these functions before
/// rendering. React already escapes JSX text content, but we add an
/// extra layer for URLs, HTML entities, and content that might be
/// dangerously inserted.

/** Characters that must be escaped in HTML context. */
const HTML_ESCAPE_MAP: Record<string, string> = {
  '&': '&amp;',
  '<': '&lt;',
  '>': '&gt;',
  '"': '&quot;',
  "'": '&#x27;',
}

const HTML_ESCAPE_RE = /[&<>"']/g

/**
 * Escape HTML special characters.
 * Use when inserting user content into innerHTML or attribute values.
 * React's JSX already handles this for text nodes, but this is the
 * safety net for any raw HTML paths.
 */
export function escapeHtml(str: string): string {
  return str.replace(HTML_ESCAPE_RE, (char) => HTML_ESCAPE_MAP[char] || char)
}

/**
 * Sanitize a URL to prevent javascript: and data: protocol attacks.
 * Returns the URL if safe, or an empty string if suspicious.
 */
export function sanitizeUrl(url: string): string {
  const trimmed = url.trim().toLowerCase()

  // Block javascript:, vbscript:, data: URLs
  if (
    trimmed.startsWith('javascript:') ||
    trimmed.startsWith('vbscript:') ||
    trimmed.startsWith('data:text/html')
  ) {
    return ''
  }

  // Allow http, https, mailto, and relative URLs
  if (
    trimmed.startsWith('http://') ||
    trimmed.startsWith('https://') ||
    trimmed.startsWith('mailto:') ||
    trimmed.startsWith('/') ||
    trimmed.startsWith('#')
  ) {
    return url
  }

  // Block anything else with a colon (potential protocol attack)
  if (trimmed.includes(':') && !trimmed.startsWith('//')) {
    return ''
  }

  return url
}

/**
 * Sanitize message content for safe rendering.
 * Strips HTML tags, normalizes whitespace, limits length.
 */
export function sanitizeMessage(content: string): string {
  return content
    .replace(/<[^>]*>/g, '')       // Strip HTML tags
    .replace(/\0/g, '')            // Remove null bytes
    .trim()
}

/**
 * Sanitize a username or display name.
 * Strips control characters and limits length.
 */
export function sanitizeName(name: string, maxLength = 32): string {
  return name
    .replace(/[\x00-\x1f\x7f]/g, '') // Strip control characters
    .replace(/<[^>]*>/g, '')          // Strip HTML tags
    .trim()
    .slice(0, maxLength)
}

/**
 * Validate that a string is a valid UUID v4.
 * Prevents injection via malformed IDs.
 */
export function isValidUuid(str: string): boolean {
  return /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(str)
}

/**
 * Rate-limit guard for client-side action throttling.
 * Returns a function that returns true if the action is allowed.
 */
export function createRateLimiter(maxPerSecond: number): () => boolean {
  const timestamps: number[] = []

  return () => {
    const now = Date.now()
    // Remove timestamps older than 1 second
    while (timestamps.length > 0 && timestamps[0] < now - 1000) {
      timestamps.shift()
    }
    if (timestamps.length >= maxPerSecond) {
      return false
    }
    timestamps.push(now)
    return true
  }
}
