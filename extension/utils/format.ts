export function Compact(n: number): string {
    if (!Number.isFinite(n)) return '0'
    const abs = Math.abs(n)
    if (abs >= 1e9) return `${(n / 1e9).toFixed(abs >= 1e10 ? 0 : 1)}B`
    if (abs >= 1e6) return `${(n / 1e6).toFixed(abs >= 1e7 ? 0 : 1)}M`
    if (abs >= 1e3) return `${(n / 1e3).toFixed(abs >= 1e4 ? 0 : 1)}K`
    return String(Math.round(n))
}

export function Since(iso: string | number): string {
    const t = typeof iso === 'number' ? iso * 1000 : Date.parse(iso)
    if (!Number.isFinite(t)) return 'Unknown'
    const days = Math.floor((Date.now() - t) / 86_400_000)
    if (days < 1) return 'Today'
    if (days < 2) return 'Yesterday'
    if (days < 30) return `${days} days ago`
    const months = Math.floor(days / 30.44)
    if (months < 12) return `${months} month${months === 1 ? '' : 's'} ago`
    const years = Math.floor(days / 365.25)
    return `${years} year${years === 1 ? '' : 's'} ago`
}

export function Age(iso: string): string {
    const t = Date.parse(iso)
    if (!Number.isFinite(t)) return 'Unknown'
    const days = Math.floor((Date.now() - t) / 86_400_000)
    const years = Math.floor(days / 365.25)
    const months = Math.floor((days - years * 365.25) / 30.44)
    if (years > 0) return `${years}y ${months}m`
    if (months > 0) return `${months} months`
    return `${days} days`
}

export function Duration(secs: number): string {
    if (secs < 60) return `${secs}s`
    const m = Math.floor(secs / 60)
    if (m < 60) return `${m}m`
    return `${Math.floor(m / 60)}h ${m % 60}m`
}

export function LikePct(up: number, down: number): number | null {
    const total = up + down
    return total ? Math.round((up / total) * 100) : null
}

export const DEMAND = ['Terrible', 'Low', 'Normal', 'High', 'Amazing']
export const TREND = ['Lowering', 'Unstable', 'Stable', 'Raising', 'Fluctuating']
