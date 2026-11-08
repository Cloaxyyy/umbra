<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { api, type Settings, type Status } from '../api'
import { Mb } from '../format'
import { StartFriendAlerts, Toast } from '../store'
import { ACCENTS, ApplyAccent } from '../theme'
import { CheckForUpdate, InstallUpdate, updates } from '../updater'

const s = ref<Settings | null>(null)
const status = ref<Status | null>(null)
const isHandler = ref(false)
const autostart = ref(false)
const discord = ref<string | null>(null)
const msg = ref('')

async function Load() {
    s.value = await api.getSettings()
    autostart.value = await api.getAutostart()
    status.value = await api.status()
    isHandler.value = status.value.isProtocolHandler
}
async function Save() {
    if (!s.value) return
    try {
        await api.setSettings(s.value)
    } catch (e) {
        Toast('bad', String(e))
    }
}
const recording = ref(false)
function KeyName(e: KeyboardEvent) {
    const code = e.code
    if (/^Key[A-Z]$/.test(code)) return code.slice(3)
    if (/^Digit[0-9]$/.test(code)) return code.slice(5)
    if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) return code
    if (['Backquote', 'Space', 'Insert', 'Home', 'End', 'PageUp', 'PageDown', 'Pause'].includes(code)) return code
    return null
}
async function SaveHotkey(hotkey: string, enabled: boolean) {
    try {
        await api.setOverlayHotkey(hotkey, enabled)
        s.value!.overlayHotkey = hotkey
        s.value!.overlayEnabled = enabled
        Toast('good', enabled ? `Press ${hotkey} in game to open the overlay.` : 'Overlay hotkey turned off.')
    } catch (e) {
        Toast('bad', String(e))
    }
}
function OnRecord(e: KeyboardEvent) {
    e.preventDefault()
    if (e.key === 'Escape') {
        StopRecording()
        return
    }
    const key = KeyName(e)
    if (!key) return
    const mods = [e.ctrlKey && 'Ctrl', e.altKey && 'Alt', e.shiftKey && 'Shift'].filter(Boolean) as string[]
    if (!mods.length && !/^F\d+$/.test(key)) {
        Toast('bad', 'Add Ctrl, Alt or Shift so the hotkey does not fire while you type in game.')
        return
    }
    StopRecording()
    SaveHotkey([...mods, key].join('+'), true)
}
function StartRecording() {
    recording.value = true
    window.addEventListener('keydown', OnRecord, true)
}
function StopRecording() {
    recording.value = false
    window.removeEventListener('keydown', OnRecord, true)
}
onUnmounted(StopRecording)

function SetAccent(hex: string) {
    s.value!.accent = hex
    ApplyAccent(hex)
    Save()
}
function Flip(
    key: 'multiInstance' | 'serverLocation' | 'discordEnabled' | 'discordJoinButton' | 'closeToTray' | 'autoCheckUpdates' | 'friendAlerts',
) {
    s.value![key] = !s.value![key]
    Save()
    if (key === 'friendAlerts') StartFriendAlerts(s.value!.friendAlerts)
}
async function ToggleAutostart() {
    try {
        await api.setAutostart(!autostart.value)
        autostart.value = await api.getAutostart()
    } catch (e) {
        Toast('bad', String(e))
    }
}
async function Register() {
    msg.value = ''
    try {
        await api.registerProtocol()
        isHandler.value = true
        msg.value = 'Done. Play buttons on roblox.com now open Umbra.'
    } catch (e) {
        msg.value = String(e)
    }
}

let timer = 0
onMounted(() => {
    Load().catch((e) => Toast('bad', String(e)))
    const Poll = async () => {
        try {
            discord.value = await api.discordStatus()
        } catch {}
    }
    Poll()
    timer = window.setInterval(Poll, 2500)
})
onUnmounted(() => clearInterval(timer))
</script>

<template>
    <div class="stack" style="gap: 22px">
        <header>
            <h1>Settings</h1>
        </header>

        <template v-if="s">
            <section class="card stack" style="gap: 18px">
                <h2 class="group">Appearance</h2>
                <div class="setting">
                    <div>
                        <h3>Accent color</h3>
                        <p class="muted">Used for links, focus rings, switches and progress. Everything else stays neutral.</p>
                    </div>
                    <div class="accents" role="radiogroup" aria-label="Accent color">
                        <button
                            v-for="a in ACCENTS"
                            :key="a.hex"
                            class="accent"
                            role="radio"
                            :aria-checked="s.accent === a.hex"
                            :aria-label="a.name"
                            :title="a.name"
                            :style="{ background: a.hex }"
                            @click="SetAccent(a.hex)"
                        ></button>
                    </div>
                </div>
            </section>

            <section class="card stack" style="gap: 18px">
                <h2 class="group">In-game overlay</h2>
                <div class="setting">
                    <div>
                        <h3>Overlay hotkey</h3>
                        <p class="muted">
                            Opens a panel on top of Roblox to rejoin, server hop, bring alts, change priority and tune graphics without
                            tabbing out. It is a separate Umbra window, so it never touches the game itself.
                        </p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.overlayEnabled"
                        aria-label="Overlay hotkey"
                        @click="SaveHotkey(s.overlayHotkey, !s.overlayEnabled)"
                    ></button>
                </div>
                <div v-if="s.overlayEnabled" class="setting">
                    <div>
                        <h3>Hotkey</h3>
                        <p class="muted">
                            Works while Roblox is in front. Roblox must be windowed or borderless fullscreen, which is its default.
                        </p>
                    </div>
                    <div class="row" style="gap: 8px">
                        <kbd class="hk">{{ recording ? 'Press keys…' : s.overlayHotkey }}</kbd>
                        <button class="btn btn-sm" @click="recording ? StopRecording() : StartRecording()">
                            {{ recording ? 'Cancel' : 'Change' }}
                        </button>
                    </div>
                </div>
            </section>

            <section class="card stack" style="gap: 18px">
                <h2 class="group">Updates</h2>
                <div class="setting">
                    <div>
                        <h3>Umbra {{ updates.current }}</h3>
                        <p v-if="updates.available" class="muted">
                            Version {{ updates.available.version }} is ready. Umbra restarts to finish.
                        </p>
                        <p v-else-if="updates.error" class="muted">Couldn't reach GitHub: {{ updates.error }}</p>
                        <p v-else class="muted">
                            New versions come from the Umbra releases on GitHub and are checked against Umbra's signing key before
                            installing.
                        </p>
                        <div
                            v-if="updates.installing && updates.total"
                            class="bar"
                            role="progressbar"
                            :aria-valuenow="Math.round((updates.done / updates.total) * 100)"
                            aria-valuemin="0"
                            aria-valuemax="100"
                        >
                            <div class="fill" :style="{ width: (updates.done / updates.total) * 100 + '%' }"></div>
                        </div>
                        <p v-if="updates.installing && updates.total" class="faint mono small">
                            {{ Mb(updates.done) }} / {{ Mb(updates.total) }}
                        </p>
                    </div>
                    <button v-if="updates.available" class="btn btn-primary" :disabled="updates.installing" @click="InstallUpdate">
                        {{ updates.installing ? 'Updating…' : 'Update and restart' }}
                    </button>
                    <button v-else class="btn" :disabled="updates.checking" @click="CheckForUpdate(true)">
                        {{ updates.checking ? 'Checking…' : 'Check now' }}
                    </button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Check on startup</h3>
                        <p class="muted">Look for a new version each time Umbra opens.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.autoCheckUpdates"
                        aria-label="Check for updates on startup"
                        @click="Flip('autoCheckUpdates')"
                    ></button>
                </div>
            </section>

            <section class="card stack" style="gap: 18px">
                <h2 class="group">Launching</h2>
                <div class="setting">
                    <div>
                        <h3>Multiple instances</h3>
                        <p class="muted">Let more than one Roblox client run at the same time.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.multiInstance"
                        aria-label="Multiple instances"
                        @click="Flip('multiInstance')"
                    ></button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Website Play button</h3>
                        <p class="muted">
                            Make Umbra the launcher roblox.com opens. This replaces Bloxstrap or the stock launcher until you switch back
                            there.
                        </p>
                        <p v-if="msg" class="muted" style="margin-top: 6px">{{ msg }}</p>
                    </div>
                    <button class="btn" :disabled="isHandler" @click="Register">{{ isHandler ? 'Using Umbra' : 'Use Umbra' }}</button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Start with Windows</h3>
                        <p class="muted">Open Umbra in the tray when you sign in to Windows.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="autostart"
                        aria-label="Start with Windows"
                        @click="ToggleAutostart"
                    ></button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Keep running in the tray</h3>
                        <p class="muted">The close button hides Umbra to the tray instead of quitting. Quit from the tray icon.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.closeToTray"
                        aria-label="Keep running in the tray"
                        @click="Flip('closeToTray')"
                    ></button>
                </div>
            </section>

            <section class="card stack" style="gap: 18px">
                <h2 class="group">While you play</h2>
                <div class="setting">
                    <div>
                        <h3>Friend alerts</h3>
                        <p class="muted">Pops up when a friend of your active account starts playing, with a button to join them.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.friendAlerts"
                        aria-label="Friend alerts"
                        @click="Flip('friendAlerts')"
                    ></button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Show server location</h3>
                        <p class="muted">
                            Shows which city a server is in. Umbra looks up the server's address with ipinfo.io, which also sees your IP
                            address. Off by default.
                        </p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.serverLocation"
                        aria-label="Show server location"
                        @click="Flip('serverLocation')"
                    ></button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Discord status</h3>
                        <p class="muted">Show the game you're in on your Discord profile while the Discord app is open.</p>
                    </div>
                    <button
                        class="switch"
                        role="switch"
                        :aria-checked="s.discordEnabled"
                        aria-label="Discord status"
                        @click="Flip('discordEnabled')"
                    ></button>
                </div>
                <div v-if="s.discordEnabled" class="discord">
                    <ol class="steps">
                        <li>
                            Open <b>discord.com/developers/applications</b> and choose <b>New Application</b>. Name it <b>Umbra</b>, since
                            Discord shows that name as "Playing Umbra".
                        </li>
                        <li>Copy the <b>Application ID</b> from its General Information page and paste it here.</li>
                    </ol>
                    <label class="field">
                        <span class="eyebrow">Discord application ID</span>
                        <input
                            id="discord-app-id"
                            v-model.trim="s.discordAppId"
                            class="input mono"
                            inputmode="numeric"
                            placeholder="123456789012345678"
                            style="max-width: 280px"
                            @change="Save"
                        />
                    </label>
                    <div class="setting">
                        <div>
                            <h3>Join button</h3>
                            <p class="muted">
                                Adds a "Join server" button to your status. Anyone who can see your profile could join your server.
                            </p>
                        </div>
                        <button
                            class="switch"
                            role="switch"
                            :aria-checked="s.discordJoinButton"
                            aria-label="Join button"
                            @click="Flip('discordJoinButton')"
                        ></button>
                    </div>
                    <p class="status" :class="{ ok: discord?.startsWith('Connected') }">
                        {{ discord ?? (s.discordAppId ? 'Starting…' : 'Paste an application ID to turn it on.') }}
                    </p>
                </div>
            </section>

            <section class="card stack" style="gap: 18px">
                <h2 class="group">Roblox</h2>
                <div class="setting">
                    <div>
                        <h3>Release channel</h3>
                        <p class="muted">Leave as <code>production</code> unless you know a channel name.</p>
                    </div>
                    <input id="channel" v-model.trim="s.channel" class="input mono" style="width: 180px" @change="Save" />
                </div>
                <div class="setting">
                    <div>
                        <h3>Data folder</h3>
                        <p class="muted">Roblox builds, instances, mods and settings live here.</p>
                    </div>
                    <button class="btn" @click="api.openDataFolder()">Open folder</button>
                </div>
                <div class="setting">
                    <div>
                        <h3>Roblox build</h3>
                        <p class="muted">
                            Installed <code>{{ status?.installed ?? 'none' }}</code> · Latest
                            <code>{{ status?.latest.clientVersionUpload ?? '…' }}</code> ({{ status?.latest.version ?? '…' }})
                        </p>
                    </div>
                </div>
            </section>
        </template>
    </div>
</template>

<style scoped>
.hk {
    font-family: var(--mono);
    font-size: 12px;
    padding: 4px 10px;
    border-radius: 8px;
    border: 1px solid var(--line-strong);
    background: var(--bg-2);
    min-width: 120px;
    text-align: center;
}
.accents {
    display: flex;
    gap: 8px;
}
.accent {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid transparent;
}
.accent[aria-checked='true'] {
    border-color: var(--fg);
    box-shadow: 0 0 0 2px var(--bg) inset;
}
.group {
    font-size: 12px;
    color: var(--fg-3);
}
.setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    padding-top: 16px;
    border-top: 1px solid var(--line);
}
.group + .setting {
    border-top: 0;
    padding-top: 0;
}
.setting h3 {
    font-size: 15px;
    font-weight: 700;
    margin-bottom: 2px;
}
.setting p {
    max-width: 70ch;
}
.discord {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
}
.discord .setting {
    border-top: 1px solid var(--line);
}
.steps {
    margin: 0;
    padding-left: 20px;
    color: var(--fg-2);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.steps b {
    color: var(--fg);
    user-select: text;
}
.field {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.bar {
    height: 6px;
    border-radius: 999px;
    background: var(--bg-2);
    overflow: hidden;
    margin-top: 10px;
    max-width: 360px;
}
.fill {
    height: 100%;
    background: var(--accent);
}
.small {
    font-size: 12px;
}
.status {
    font-size: 13px;
    font-weight: 700;
    color: var(--warn);
}
.status.ok {
    color: var(--good);
}
</style>
