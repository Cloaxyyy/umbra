import { getVersion } from '@tauri-apps/api/app'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { reactive } from 'vue'
import { Toast } from './store'

export const updates = reactive({
    current: '',
    checking: false,
    available: null as Update | null,
    installing: false,
    done: 0,
    total: 0,
    error: '',
    lastChecked: null as number | null,
})

getVersion()
    .then((v) => (updates.current = v))
    .catch(() => {})

export async function CheckForUpdate(manual: boolean) {
    if (updates.checking || updates.installing) return
    updates.checking = true
    updates.error = ''
    try {
        const u = await check()
        updates.available = u
        updates.lastChecked = Date.now()
        if (u) Toast('good', `Umbra ${u.version} is available.`)
        else if (manual) Toast('good', "You're on the latest version.")
    } catch (e) {
        updates.error = String(e)
        if (manual) Toast('bad', `Couldn't check for updates: ${e}`)
    }
    updates.checking = false
}

export async function InstallUpdate() {
    const u = updates.available
    if (!u || updates.installing) return
    updates.installing = true
    updates.done = 0
    updates.total = 0
    updates.error = ''
    try {
        await u.downloadAndInstall((ev) => {
            if (ev.event === 'Started') updates.total = ev.data.contentLength ?? 0
            else if (ev.event === 'Progress') updates.done += ev.data.chunkLength
        })
        await relaunch()
    } catch (e) {
        updates.error = String(e)
        Toast('bad', `Update failed: ${e}`)
    }
    updates.installing = false
}
