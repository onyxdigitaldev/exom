/// useFileDrop — drag-and-drop file upload handler.
///
/// Attaches to a drop zone element. Shows a visual indicator when
/// files are dragged over. Calls onDrop with the dropped files.

import { useState, useCallback, useRef, type DragEvent } from 'react'

interface UseFileDropOptions {
  /** Called when files are dropped. */
  onDrop: (files: File[]) => void
  /** Maximum file size in bytes (default 25MB). */
  maxSize?: number
  /** Accepted MIME types (e.g. ['image/*', 'application/pdf']). Empty = all. */
  accept?: string[]
}

export function useFileDrop(options: UseFileDropOptions) {
  const { onDrop, maxSize = 25 * 1024 * 1024, accept = [] } = options
  const [isDragging, setIsDragging] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const dragCounter = useRef(0)

  const handleDragEnter = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    dragCounter.current++
    if (e.dataTransfer.items?.length > 0) {
      setIsDragging(true)
    }
  }, [])

  const handleDragLeave = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    dragCounter.current--
    if (dragCounter.current === 0) {
      setIsDragging(false)
    }
  }, [])

  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback((e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
    dragCounter.current = 0
    setError(null)

    const files = Array.from(e.dataTransfer.files)
    if (files.length === 0) return

    // Validate file sizes
    const oversized = files.find((f) => f.size > maxSize)
    if (oversized) {
      setError(`File "${oversized.name}" exceeds ${Math.round(maxSize / 1024 / 1024)}MB limit`)
      return
    }

    // Validate MIME types
    if (accept.length > 0) {
      const invalid = files.find((f) => !accept.some((a) => {
        if (a.endsWith('/*')) {
          return f.type.startsWith(a.replace('/*', '/'))
        }
        return f.type === a
      }))
      if (invalid) {
        setError(`File type "${invalid.type || 'unknown'}" is not accepted`)
        return
      }
    }

    onDrop(files)
  }, [onDrop, maxSize, accept])

  const clearError = useCallback(() => setError(null), [])

  return {
    /** Whether files are currently being dragged over the zone. */
    isDragging,
    /** Validation error from the last drop attempt. */
    error,
    clearError,
    /** Props to spread onto the drop zone element. */
    dropZoneProps: {
      onDragEnter: handleDragEnter,
      onDragLeave: handleDragLeave,
      onDragOver: handleDragOver,
      onDrop: handleDrop,
    },
  }
}
