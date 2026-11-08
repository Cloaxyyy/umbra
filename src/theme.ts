export const ACCENTS = [
    { name: 'Eclipse', hex: '#9580ff' },
    { name: 'Corona', hex: '#62d4ff' },
    { name: 'Aurora', hex: '#3fe0a0' },
    { name: 'Ember', hex: '#ff8a5c' },
    { name: 'Rose', hex: '#ff6f9f' },
    { name: 'Sun', hex: '#ffc94d' },
]

export function ApplyAccent(hex: string) {
    const value = /^#[0-9a-f]{6}$/i.test(hex) ? hex : ACCENTS[0].hex
    const n = parseInt(value.slice(1), 16)
    const rgb = `${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}`
    const root = document.documentElement.style
    root.setProperty('--accent', value)
    root.setProperty('--accent-soft', `rgba(${rgb}, 0.14)`)
    root.setProperty('--accent-line', `rgba(${rgb}, 0.4)`)
}
