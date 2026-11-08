export function Compact(n: number): string {
    if (n < 1000) return String(n)
    const [d, s] = n >= 1e9 ? [1e9, 'B'] : n >= 1e6 ? [1e6, 'M'] : [1e3, 'K']
    const v = n / d
    return (v >= 100 ? v.toFixed(0) : v.toFixed(1).replace(/\.0$/, '')) + s
}

export function LikePct(up: number, down: number): number | null {
    const t = up + down
    return t ? Math.round((up / t) * 100) : null
}

export function Sleep(ms: number) {
    return new Promise((r) => setTimeout(r, ms))
}

export function Dur(secs: number): string {
    if (secs < 60) return `${secs}s`
    const m = Math.floor(secs / 60)
    if (m < 60) return `${m}m`
    return `${Math.floor(m / 60)}h ${m % 60}m`
}

export function Ago(unix: number): string {
    const s = Math.max(0, Math.floor(Date.now() / 1000) - unix)
    if (s < 60) return 'just now'
    if (s < 3600) return `${Math.floor(s / 60)}m ago`
    if (s < 86400) return `${Math.floor(s / 3600)}h ago`
    return `${Math.floor(s / 86400)}d ago`
}

export function Mb(bytes: number): string {
    if (bytes < 1_048_576) return Math.max(1, Math.round(bytes / 1024)) + ' KB'
    const m = bytes / 1_048_576
    return (m < 10 ? m.toFixed(1) : Math.round(m)) + ' MB'
}
