import { useState } from "react"
import { cn } from "@/lib/utils"
import { 
  X, 
  User, 
  Shield, 
  Bell, 
  Palette, 
  Mic, 
  Keyboard, 
  LogOut,
  ChevronRight,
  Camera,
  Check
} from "lucide-react"
import { currentUser } from "@/lib/mock-data"
import { Switch } from "@/components/ui/switch"

interface SettingsModalProps {
  onClose: () => void
}

type Tab = "account" | "profile" | "privacy" | "notifications" | "appearance" | "voice" | "keybinds"

const tabs: { id: Tab; label: string; icon: React.ElementType }[] = [
  { id: "account", label: "My Account", icon: User },
  { id: "profile", label: "Profile", icon: User },
  { id: "privacy", label: "Privacy & Safety", icon: Shield },
  { id: "notifications", label: "Notifications", icon: Bell },
  { id: "appearance", label: "Appearance", icon: Palette },
  { id: "voice", label: "Voice & Video", icon: Mic },
  { id: "keybinds", label: "Keybinds", icon: Keyboard },
]

export function SettingsModal({ onClose }: SettingsModalProps) {
  const [activeTab, setActiveTab] = useState<Tab>("account")

  return (
    <div className="fixed inset-0 bg-background z-50 flex">
      {/* Sidebar */}
      <div className="w-56 bg-secondary/30 p-4 flex flex-col">
        <div className="flex-1">
          <h3 className="px-3 py-2 text-xs font-semibold text-muted-foreground uppercase tracking-wide">
            User Settings
          </h3>
          <div className="space-y-0.5 mt-2">
            {tabs.map(tab => {
              const Icon = tab.icon
              return (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={cn(
                    "w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-all",
                    activeTab === tab.id
                      ? "bg-primary/15 text-primary"
                      : "text-muted-foreground hover:text-foreground hover:bg-secondary/50"
                  )}
                >
                  <Icon className="w-4 h-4" />
                  {tab.label}
                </button>
              )
            })}
          </div>

          <div className="h-px bg-border/50 my-4" />

          <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-destructive hover:bg-destructive/10 transition-all">
            <LogOut className="w-4 h-4" />
            Log Out
          </button>
        </div>

        <div className="text-xs text-muted-foreground text-center py-4">
          Exom v1.0.0
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-2xl mx-auto p-8">
          {/* Close button */}
          <div className="fixed top-4 right-4 flex items-center gap-2">
            <button
              onClick={onClose}
              className="w-10 h-10 rounded-full bg-secondary/50 hover:bg-secondary flex items-center justify-center transition-colors"
            >
              <X className="w-5 h-5 text-muted-foreground" />
            </button>
            <span className="text-xs text-muted-foreground">ESC</span>
          </div>

          {activeTab === "account" && (
            <div>
              <h2 className="text-2xl font-bold text-foreground mb-6">My Account</h2>
              
              {/* Profile Card */}
              <div className="rounded-2xl bg-secondary/30 border border-border/30 overflow-hidden">
                {/* Banner */}
                <div className="h-24 bg-gradient-to-r from-primary/50 to-accent/50" />
                
                <div className="p-6 -mt-12">
                  <div className="flex items-end gap-4">
                    <div className="relative">
                      <div className="w-20 h-20 rounded-full bg-gradient-to-br from-primary to-accent flex items-center justify-center text-white text-2xl font-bold border-4 border-card">
                        {currentUser.username.charAt(0).toUpperCase()}
                      </div>
                      <button className="absolute bottom-0 right-0 w-7 h-7 rounded-full bg-secondary border border-border flex items-center justify-center hover:bg-secondary/80 transition-colors">
                        <Camera className="w-3.5 h-3.5 text-muted-foreground" />
                      </button>
                    </div>
                    <div className="flex-1 pb-1">
                      <h3 className="text-xl font-bold text-foreground">{currentUser.username}</h3>
                      <p className="text-sm text-muted-foreground">#{currentUser.discriminator}</p>
                    </div>
                    <button className="px-4 py-2 rounded-lg bg-primary/10 text-primary text-sm font-medium hover:bg-primary/20 transition-all">
                      Edit Profile
                    </button>
                  </div>

                  <div className="mt-6 space-y-4">
                    <div className="flex items-center justify-between py-3 border-b border-border/30">
                      <div>
                        <p className="text-xs text-muted-foreground uppercase tracking-wide">Username</p>
                        <p className="text-foreground mt-0.5">{currentUser.username}#{currentUser.discriminator}</p>
                      </div>
                      <button className="px-3 py-1.5 rounded-lg bg-secondary/50 text-sm text-foreground hover:bg-secondary transition-colors">
                        Edit
                      </button>
                    </div>
                    <div className="flex items-center justify-between py-3 border-b border-border/30">
                      <div>
                        <p className="text-xs text-muted-foreground uppercase tracking-wide">Email</p>
                        <p className="text-foreground mt-0.5">u***@example.com</p>
                      </div>
                      <button className="px-3 py-1.5 rounded-lg bg-secondary/50 text-sm text-foreground hover:bg-secondary transition-colors">
                        Edit
                      </button>
                    </div>
                    <div className="flex items-center justify-between py-3">
                      <div>
                        <p className="text-xs text-muted-foreground uppercase tracking-wide">Phone Number</p>
                        <p className="text-foreground mt-0.5">Not added</p>
                      </div>
                      <button className="px-3 py-1.5 rounded-lg bg-secondary/50 text-sm text-foreground hover:bg-secondary transition-colors">
                        Add
                      </button>
                    </div>
                  </div>
                </div>
              </div>

              {/* Password & Authentication */}
              <div className="mt-8">
                <h3 className="text-lg font-semibold text-foreground mb-4">Password and Authentication</h3>
                <button className="px-4 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-all">
                  Change Password
                </button>

                <div className="mt-6 p-4 rounded-xl bg-secondary/30 border border-border/30">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="font-medium text-foreground">Two-Factor Authentication</p>
                      <p className="text-sm text-muted-foreground mt-0.5">Protect your account with 2FA</p>
                    </div>
                    <button className="px-4 py-2 rounded-lg bg-online/20 text-online text-sm font-medium hover:bg-online/30 transition-all">
                      Enable
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {activeTab === "appearance" && (
            <div>
              <h2 className="text-2xl font-bold text-foreground mb-6">Appearance</h2>
              
              <div className="space-y-6">
                <div>
                  <h3 className="text-sm font-medium text-foreground mb-3">Theme</h3>
                  <div className="grid grid-cols-3 gap-3">
                    <button className="p-4 rounded-xl bg-zinc-900 border-2 border-primary flex flex-col items-center gap-2 transition-all">
                      <div className="w-full h-12 rounded-lg bg-zinc-800" />
                      <span className="text-sm text-white">Dark</span>
                      <Check className="w-4 h-4 text-primary" />
                    </button>
                    <button className="p-4 rounded-xl bg-white border border-border flex flex-col items-center gap-2 hover:border-primary/50 transition-all">
                      <div className="w-full h-12 rounded-lg bg-gray-100" />
                      <span className="text-sm text-zinc-900">Light</span>
                    </button>
                    <button className="p-4 rounded-xl bg-gradient-to-b from-zinc-900 to-white border border-border flex flex-col items-center gap-2 hover:border-primary/50 transition-all">
                      <div className="w-full h-12 rounded-lg bg-gradient-to-b from-zinc-800 to-gray-100" />
                      <span className="text-sm text-foreground">System</span>
                    </button>
                  </div>
                </div>

                <div className="h-px bg-border/50" />

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Message Display</p>
                    <p className="text-sm text-muted-foreground">Cozy shows avatars, Compact hides them</p>
                  </div>
                  <div className="flex items-center gap-2 bg-secondary/50 rounded-lg p-1">
                    <button className="px-3 py-1.5 rounded-md bg-primary text-primary-foreground text-sm font-medium">
                      Cozy
                    </button>
                    <button className="px-3 py-1.5 rounded-md text-sm text-muted-foreground hover:text-foreground">
                      Compact
                    </button>
                  </div>
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Show Embeds</p>
                    <p className="text-sm text-muted-foreground">Display link previews in chat</p>
                  </div>
                  <Switch defaultChecked />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Animate Emoji</p>
                    <p className="text-sm text-muted-foreground">Allow animated emoji to play</p>
                  </div>
                  <Switch defaultChecked />
                </div>
              </div>
            </div>
          )}

          {activeTab === "notifications" && (
            <div>
              <h2 className="text-2xl font-bold text-foreground mb-6">Notifications</h2>
              
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Enable Desktop Notifications</p>
                    <p className="text-sm text-muted-foreground">Show notifications on your desktop</p>
                  </div>
                  <Switch defaultChecked />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Enable Unread Message Badge</p>
                    <p className="text-sm text-muted-foreground">Show count on app icon</p>
                  </div>
                  <Switch defaultChecked />
                </div>

                <div className="flex items-center justify-between">
                  <div>
                    <p className="font-medium text-foreground">Enable Notification Sounds</p>
                    <p className="text-sm text-muted-foreground">Play sounds for incoming messages</p>
                  </div>
                  <Switch defaultChecked />
                </div>

                <div className="h-px bg-border/50" />

                <div>
                  <h3 className="text-lg font-semibold text-foreground mb-4">Hall Notification Settings</h3>
                  <p className="text-sm text-muted-foreground">Configure notifications for each Hall individually in the Hall settings.</p>
                </div>
              </div>
            </div>
          )}

          {(activeTab === "profile" || activeTab === "privacy" || activeTab === "voice" || activeTab === "keybinds") && (
            <div>
              <h2 className="text-2xl font-bold text-foreground mb-6">
                {tabs.find(t => t.id === activeTab)?.label}
              </h2>
              <div className="rounded-2xl bg-secondary/30 border border-border/30 p-8 text-center">
                <p className="text-muted-foreground">Settings for {tabs.find(t => t.id === activeTab)?.label} coming soon...</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
