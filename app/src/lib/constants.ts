/// Role configuration — colors and display labels for the 5-tier hierarchy.
/// Extracted from mock data as the canonical source of role metadata.

export type RoleKey = 'builder' | 'prefect' | 'moderator' | 'agent' | 'fellow'

export const roleConfig: Record<RoleKey, { label: string; color: string; icon: string }> = {
  builder: { label: 'Hall Builder', color: '#F59E0B', icon: 'crown' },
  prefect: { label: 'Hall Prefect', color: '#EF4444', icon: 'shield' },
  moderator: { label: 'Hall Moderator', color: '#3B82F6', icon: 'wrench' },
  agent: { label: 'Hall Agent', color: '#22C55E', icon: 'user' },
  fellow: { label: 'Hall Fellow', color: '#9CA3AF', icon: 'eye' },
}

/// Map a backend role short name to its config key.
export function roleKeyFromName(name: string): RoleKey {
  const lower = name.toLowerCase()
  if (lower in roleConfig) return lower as RoleKey
  return 'agent'
}
