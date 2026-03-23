/// EmojiPicker — compact emoji selector with categories and search.
///
/// Uses native Unicode emoji. Grouped by category with quick-access
/// tabs. Recently used section at the top.

import { useState, useMemo, useRef, useEffect } from 'react'
import { cn } from '@/lib/utils'
import { Search, X, Clock, Smile, Heart, Coffee, Palmtree, Gamepad2, Flag, Hash } from 'lucide-react'

interface EmojiPickerProps {
  onSelect: (emoji: string) => void
  onClose: () => void
}

interface EmojiCategory {
  id: string
  name: string
  icon: React.ReactNode
  emojis: string[]
}

const CATEGORIES: EmojiCategory[] = [
  {
    id: 'smileys', name: 'Smileys', icon: <Smile className="w-4 h-4" />,
    emojis: ['😀','😃','😄','😁','😆','😅','🤣','😂','🙂','😉','😊','😇','🥰','😍','🤩','😘','😗','😚','😋','😛','😜','🤪','😝','🤑','🤗','🤭','🤫','🤔','🤐','🤨','😐','😑','😶','😏','😒','🙄','😬','🤥','😌','😔','😪','🤤','😴','😷','🤒','🤕','🤢','🤮','🥵','🥶','🥴','😵','🤯','🤠','🥳','🥸','😎','🤓','🧐','😕','😟','🙁','😮','😯','😲','😳','🥺','😦','😧','😨','😰','😥','😢','😭','😱','😖','😣','😞','😓','😩','😫','🥱','😤','😡','😠','🤬','😈','👿','💀','☠️','💩','🤡','👹','👺','👻','👽','👾','🤖'],
  },
  {
    id: 'gestures', name: 'Gestures', icon: <Hash className="w-4 h-4" />,
    emojis: ['👋','🤚','🖐️','✋','🖖','👌','🤌','🤏','✌️','🤞','🤟','🤘','🤙','👈','👉','👆','🖕','👇','☝️','👍','👎','✊','👊','🤛','🤜','👏','🙌','👐','🤲','🤝','🙏','✍️','💪','🦾','🦿','🦵','🦶','👂','🦻','👃','🫀','🫁','🧠','🦷','🦴','👀','👁️','👅','👄'],
  },
  {
    id: 'hearts', name: 'Hearts', icon: <Heart className="w-4 h-4" />,
    emojis: ['❤️','🧡','💛','💚','💙','💜','🖤','🤍','🤎','💔','❤️‍🔥','❤️‍🩹','❣️','💕','💞','💓','💗','💖','💘','💝','💟','♥️','🫶'],
  },
  {
    id: 'nature', name: 'Nature', icon: <Palmtree className="w-4 h-4" />,
    emojis: ['🌸','💐','🌷','🌹','🥀','🌺','🌻','🌼','🌱','🌲','🌳','🌴','🌵','🌾','🌿','☘️','🍀','🍁','🍂','🍃','🍄','🐶','🐱','🐭','🐹','🐰','🦊','🐻','🐼','🐨','🐯','🦁','🐮','🐷','🐸','🐵','🐔','🐧','🐦','🐤','🦅','🦆','🦉','🐺','🐗','🐴','🦄','🐝','🐛','🦋','🐌','🐞'],
  },
  {
    id: 'food', name: 'Food', icon: <Coffee className="w-4 h-4" />,
    emojis: ['🍎','🍐','🍊','🍋','🍌','🍉','🍇','🍓','🫐','🍈','🍒','🍑','🥭','🍍','🥥','🥝','🍅','🍆','🥑','🥦','🥬','🥒','🌶️','🫑','🌽','🥕','🫒','🧄','🧅','🥔','🍠','🥐','🥖','🍞','🥨','🥯','🧀','🥚','🍳','🧈','🥞','🧇','🥓','🥩','🍗','🍖','🌭','🍔','🍟','🍕','🫓','🥪','🌮','🌯','🫔','🥙','🧆','🥚','🍝','🍜','🍲','🍛','🍣','🍱','🥟','🦪','🍤','🍙','🍚','🍘','🍥','🥠','🥮','🍢','🍡','🍧','🍨','🍦','🥧','🧁','🍰','🎂','🍮','🍭','🍬','🍫','🍿','🍩','🍪','🌰','🥜','🍯','🥛','🍼','☕','🫖','🍵','🧃','🥤','🧋','🍶','🍺','🍻','🥂','🍷','🥃','🍸','🍹','🧉'],
  },
  {
    id: 'activities', name: 'Activities', icon: <Gamepad2 className="w-4 h-4" />,
    emojis: ['⚽','🏀','🏈','⚾','🥎','🎾','🏐','🏉','🥏','🎱','🪀','🏓','🏸','🏒','🏑','🥍','🏏','🪃','🥅','⛳','🪁','🏹','🎣','🤿','🥊','🥋','🎽','🛹','🛼','🛷','⛸️','🥌','🎿','⛷️','🏂','🪂','🏋️','🤸','🤼','🤽','🤾','🤺','⛹️','🧘','🏄','🏇','🚴','🚵','🎮','🕹️','🎯','🎳','🎲','🧩','♟️','🎰','🎪'],
  },
  {
    id: 'objects', name: 'Objects', icon: <Flag className="w-4 h-4" />,
    emojis: ['💡','🔦','🕯️','💰','💵','💴','💶','💷','💳','💎','⚖️','🔧','🔨','⚒️','🛠️','⛏️','🔩','⚙️','🧱','⛓️','🧲','🔫','💣','🧨','🪓','🔪','🗡️','⚔️','🛡️','🚬','⚰️','⚱️','🏺','🔮','📿','🧿','💈','⚗️','🔭','🔬','🕳️','🩹','🩺','💊','💉','🩸','🧬','🦠','🧫','🧪','🌡️','🧹','🧺','🧻','🚰','🚿','🛁','🛀','🧼','🪥','🪒','🧽','🪣','🧴','🔑','🗝️','🚪','🪑','🛋️','🛏️','🛌','🧸','🖼️','🪞','🪟','🛒','🎁','🎈','🎏','🎀','🪄','🎊','🎉','🎎','🏮','🎐','🧧','✉️','📩','📨','📧','💌','📥','📤','📦','🏷️','📪','📫','📬','📭','📮','📯','📜','📃','📄','📑','🧾','📊','📈','📉','🗒️','🗓️','📆','📅','🗑️','📇','🗃️','🗳️','🗄️','📋','📁','📂','🗂️','🗞️','📰','📓','📔','📒','📕','📗','📘','📙','📚','📖','🔖','🧷','🔗','📎','🖇️','📐','📏','🧮','📌','📍','✂️','🖊️','🖋️','✒️','🖌️','🖍️','📝','✏️','🔍','🔎','🔏','🔐','🔒','🔓'],
  },
]

/** Persistent recently used emojis (localStorage). */
function getRecent(): string[] {
  try {
    return JSON.parse(localStorage.getItem('exom:recent-emoji') || '[]')
  } catch {
    return []
  }
}

function addRecent(emoji: string): void {
  const recent = getRecent().filter((e) => e !== emoji)
  recent.unshift(emoji)
  localStorage.setItem('exom:recent-emoji', JSON.stringify(recent.slice(0, 24)))
}

export function EmojiPicker({ onSelect, onClose }: EmojiPickerProps) {
  const [search, setSearch] = useState('')
  const [activeCategory, setActiveCategory] = useState('smileys')
  const [recent] = useState(getRecent)
  const inputRef = useRef<HTMLInputElement>(null)
  const scrollRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const filteredCategories = useMemo(() => {
    if (!search.trim()) return CATEGORIES
    const q = search.toLowerCase()
    return CATEGORIES.map((cat) => ({
      ...cat,
      emojis: cat.emojis.filter((e) => e.includes(q)),
    })).filter((cat) => cat.emojis.length > 0)
  }, [search])

  const handleSelect = (emoji: string) => {
    addRecent(emoji)
    onSelect(emoji)
  }

  return (
    <div className="w-80 bg-popover border border-border rounded-2xl shadow-2xl overflow-hidden">
      {/* Search */}
      <div className="p-2 border-b border-border/30">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            ref={inputRef}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search emoji..."
            className="w-full h-9 pl-9 pr-3 rounded-lg bg-secondary/50 text-sm placeholder:text-muted-foreground focus:outline-none"
          />
        </div>
      </div>

      {/* Category tabs */}
      <div className="flex items-center gap-0.5 px-2 py-1 border-b border-border/30">
        {recent.length > 0 && (
          <button
            onClick={() => setActiveCategory('recent')}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              activeCategory === 'recent' ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground',
            )}
          >
            <Clock className="w-4 h-4" />
          </button>
        )}
        {CATEGORIES.map((cat) => (
          <button
            key={cat.id}
            onClick={() => setActiveCategory(cat.id)}
            className={cn(
              'p-1.5 rounded-lg transition-colors',
              activeCategory === cat.id ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground',
            )}
          >
            {cat.icon}
          </button>
        ))}
      </div>

      {/* Emoji grid */}
      <div ref={scrollRef} className="h-64 overflow-y-auto p-2">
        {/* Recent */}
        {activeCategory === 'recent' && recent.length > 0 && (
          <div className="mb-3">
            <p className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">
              Recently Used
            </p>
            <div className="grid grid-cols-8 gap-0.5">
              {recent.map((emoji, i) => (
                <button
                  key={`r-${i}`}
                  onClick={() => handleSelect(emoji)}
                  className="w-9 h-9 rounded-lg hover:bg-secondary/50 flex items-center justify-center text-xl transition-all hover:scale-110"
                >
                  {emoji}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Categories */}
        {filteredCategories.map((cat) => {
          if (search.trim() || activeCategory === cat.id || activeCategory === 'recent') {
            return (
              <div key={cat.id} className="mb-3">
                <p className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">
                  {cat.name}
                </p>
                <div className="grid grid-cols-8 gap-0.5">
                  {cat.emojis.map((emoji, i) => (
                    <button
                      key={`${cat.id}-${i}`}
                      onClick={() => handleSelect(emoji)}
                      className="w-9 h-9 rounded-lg hover:bg-secondary/50 flex items-center justify-center text-xl transition-all hover:scale-110"
                    >
                      {emoji}
                    </button>
                  ))}
                </div>
              </div>
            )
          }
          return null
        })}
      </div>
    </div>
  )
}
