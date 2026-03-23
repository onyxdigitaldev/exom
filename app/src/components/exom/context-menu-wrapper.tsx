/// ContextMenuWrapper — reusable right-click context menu.
///
/// Wraps any element with a custom context menu that appears on right-click.
/// Positioned at the cursor. Closes on click outside or Escape.

import { useState, useCallback, useRef, useEffect, type ReactNode } from 'react'
import { cn } from '@/lib/utils'

export interface ContextMenuItem {
  label: string
  icon?: ReactNode
  onClick: () => void
  danger?: boolean
  separator?: boolean
  disabled?: boolean
}

interface ContextMenuWrapperProps {
  items: ContextMenuItem[]
  children: ReactNode
  className?: string
}

export function ContextMenuWrapper({ items, children, className }: ContextMenuWrapperProps) {
  const [open, setOpen] = useState(false)
  const [position, setPosition] = useState({ x: 0, y: 0 })
  const menuRef = useRef<HTMLDivElement>(null)

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()

    // Position the menu at the cursor, clamped to viewport
    const x = Math.min(e.clientX, window.innerWidth - 200)
    const y = Math.min(e.clientY, window.innerHeight - items.length * 36 - 16)

    setPosition({ x, y })
    setOpen(true)
  }, [items.length])

  // Close on click outside or Escape
  useEffect(() => {
    if (!open) return

    function handleClick(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }

    document.addEventListener('mousedown', handleClick)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handleClick)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [open])

  return (
    <>
      <div onContextMenu={handleContextMenu} className={className}>
        {children}
      </div>

      {open && (
        <div
          ref={menuRef}
          className="fixed z-[200] min-w-[180px] bg-popover border border-border rounded-xl shadow-2xl p-1 animate-in fade-in-0 zoom-in-95"
          style={{ left: position.x, top: position.y }}
        >
          {items.map((item, i) => {
            if (item.separator) {
              return <div key={i} className="h-px bg-border/50 my-1" />
            }

            return (
              <button
                key={i}
                disabled={item.disabled}
                onClick={() => {
                  setOpen(false)
                  item.onClick()
                }}
                className={cn(
                  'w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition-all',
                  item.danger
                    ? 'text-destructive hover:bg-destructive/10'
                    : 'text-foreground hover:bg-secondary/50',
                  item.disabled && 'opacity-40 cursor-not-allowed',
                )}
              >
                {item.icon && (
                  <span className="w-4 h-4 flex items-center justify-center flex-shrink-0">
                    {item.icon}
                  </span>
                )}
                <span>{item.label}</span>
              </button>
            )
          })}
        </div>
      )}
    </>
  )
}
