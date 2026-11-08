export interface Snippet {
    file: string
    label: string
    code: string
}

export const SNIPPETS: Snippet[] = [
    {
        file: 'src-tauri/src/roblox.rs',
        label: 'Private servers',
        code: `pub fn link_code(input: &str) -> Option<String> {
    let i = input.find("privateServerLinkCode=")? + "privateServerLinkCode=".len();
    let code: String = input[i..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    (!code.is_empty() && code.len() <= 64).then_some(code)
}`,
    },
    {
        file: 'src-tauri/src/theme.rs',
        label: 'Accent menus',
        code: `fn tint_pixel(px: &mut [u8], accent: [u8; 3]) {
    let (lo, hi) = (px[0].min(px[1]).min(px[2]), px[0].max(px[1]).max(px[2]));
    if px[3] == 0 || hi - lo > GRAY_SPREAD {
        return;
    }
    let l = (px[0] as f32 + px[1] as f32 + px[2] as f32) / (3.0 * 255.0);
    for i in 0..3 {
        let target = 255.0 * (1.0 - TINT_STRENGTH) + accent[i] as f32 * TINT_STRENGTH;
        px[i] = (l * target).round().clamp(0.0, 255.0) as u8;
    }
}`,
    },
]

export interface Token {
    t: string
    c: string
}

const TOKEN_RE =
    /("[^"]*"|'[^']*')|\b(pub|fn|let|as|if|else|return|mut|for|in)\b|\b([A-Z][A-Z_]+)\b|\b(\d+(?:\.\d+)?)\b|\b([a-z_]+)(?=\()|\b(u8|f32|str|String|Option)\b/g
const CLASSES = ['tk-s', 'tk-k', 'tk-c', 'tk-n', 'tk-f', 'tk-t']

export function Tokenize(code: string): Token[][] {
    return code.split('\n').map((line) => {
        const out: Token[] = []
        let last = 0
        for (const m of line.matchAll(TOKEN_RE)) {
            if (m.index > last) out.push({ t: line.slice(last, m.index), c: '' })
            const group = m.slice(1).findIndex((g) => g !== undefined)
            out.push({ t: m[0], c: CLASSES[group] ?? '' })
            last = m.index + m[0].length
        }
        if (last < line.length) out.push({ t: line.slice(last), c: '' })
        return out
    })
}
