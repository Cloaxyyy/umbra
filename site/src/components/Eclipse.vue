<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'

const props = defineProps<{ fallback: string }>()
const canvas = ref<HTMLCanvasElement | null>(null)
const failed = ref(false)

const VERT = `
attribute vec2 aPos;
void main() { gl_Position = vec4(aPos, 0.0, 1.0); }
`

const FRAG = `
precision highp float;
uniform vec2 uRes;
uniform float uTime;
uniform float uAngle;
uniform float uIntro;

float Hash(vec3 p) {
    p = fract(p * 0.3183099 + 0.1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

float Noise(vec3 x) {
    vec3 i = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(mix(Hash(i), Hash(i + vec3(1, 0, 0)), f.x), mix(Hash(i + vec3(0, 1, 0)), Hash(i + vec3(1, 1, 0)), f.x), f.y),
        mix(mix(Hash(i + vec3(0, 0, 1)), Hash(i + vec3(1, 0, 1)), f.x), mix(Hash(i + vec3(0, 1, 1)), Hash(i + vec3(1, 1, 1)), f.x), f.y),
        f.z
    );
}

float Fbm(vec3 p) {
    float v = 0.0;
    float a = 0.5;
    for (int i = 0; i < 5; i++) {
        v += a * Noise(p);
        p = p * 2.03 + vec3(1.7, 9.2, 3.1);
        a *= 0.5;
    }
    return v;
}

vec3 Tint(float t) {
    vec3 violet = vec3(0.62, 0.42, 1.0);
    vec3 blue = vec3(0.34, 0.62, 1.0);
    return mix(violet, blue, clamp(t, 0.0, 1.0));
}

float AngleGap(float a, float b) {
    float d = mod(a - b + 3.14159265, 6.2831853) - 3.14159265;
    return abs(d);
}

void main() {
    vec2 p = (gl_FragCoord.xy - 0.5 * uRes) / min(uRes.x, uRes.y);
    vec3 ro = vec3(0.0, 0.0, 3.4);
    vec3 rd = normalize(vec3(p, -1.55));
    float R = 0.62;

    vec2 L2 = vec2(cos(uAngle), sin(uAngle));
    vec3 L = normalize(vec3(L2 * 0.92, -0.62));

    float b = dot(ro, rd);
    float c = dot(ro, ro) - R * R;
    float disc = b * b - c;

    vec3 closest = ro - rd * b;
    float dist = length(closest);
    float h = max(dist - R, 0.0);
    float ang = atan(closest.y, closest.x);
    float gap = AngleGap(ang, uAngle);
    float side = 0.5 + 0.5 * cos(gap);

    float px = 2.2 / min(uRes.x, uRes.y) * 1.25;
    float cover = 1.0 - smoothstep(R - px, R + px, dist);

    float t = -b - sqrt(max(disc, 0.0));
    vec3 n = normalize(ro + rd * t);
    float fres = pow(1.0 - max(dot(n, -rd), 0.0), 3.2);
    float facing = max(dot(normalize(n.xy + 1e-4), L2), 0.0);
    vec3 rim = Tint(side) * fres * (0.18 + 2.2 * pow(facing, 3.0)) * 0.55;
    vec3 refl = reflect(rd, n);
    float spec = pow(max(dot(refl, L), 0.0), 60.0) * 0.9;
    float sheen = pow(max(dot(refl, normalize(vec3(-0.4, 0.7, 0.6))), 0.0), 5.0) * 0.022;
    vec3 body = vec3(0.004, 0.004, 0.008) + rim + vec3(spec) * Tint(side) + vec3(sheen);
    body *= mix(0.25, 1.0, uIntro);

    float flow = uTime * 0.035;
    vec2 dir = vec2(cos(ang), sin(ang));
    float streaks = Fbm(vec3(dir * 1.6, h * 3.0 - flow));
    float rays = Fbm(vec3(dir * 5.5, h * 1.4 - flow * 0.7 + 4.0));
    rays = rays * rays * 2.2;
    float inner = exp(-h * 55.0);
    float halo = exp(-h * 9.0) * (0.6 + 1.3 * streaks);
    float wide = exp(-h * 2.8) * 0.3 * (0.35 + rays);
    float lit = 0.4 + 1.5 * pow(side, 2.0);
    vec3 tint = Tint(side * 0.85 + 0.1);
    vec3 glow = tint * (inner * 2.0 + halo + wide) * lit;
    float spot = exp(-pow(gap / 0.24, 2.0)) * exp(-h * 12.0) * 2.8;
    float bead = exp(-pow(gap / 0.07, 2.0)) * exp(-h * 60.0) * 3.2;
    glow += mix(tint, vec3(1.0), 0.65) * spot + vec3(0.95, 0.97, 1.0) * bead;
    vec2 sp = vec2(cos(uAngle), sin(uAngle)) * (R * 1.55 / 3.4 * 1.02);
    vec2 dv = p - sp;
    float flare = exp(-length(dv) * 34.0) * 1.3;
    float cross = (exp(-abs(dv.y) * 240.0) * exp(-abs(dv.x) * 11.0) + exp(-abs(dv.x) * 240.0) * exp(-abs(dv.y) * 11.0)) * 0.4;
    glow += vec3(0.85, 0.9, 1.0) * (flare + cross);
    glow *= (1.0 - smoothstep(0.3, 0.5, length(p))) * uIntro;

    vec3 col = mix(glow, body, cover);
    col = 1.0 - exp(-col * 1.35);
    float grain = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233))) * 43758.5453) - 0.5;
    col = max(col + grain / 255.0, 0.0);
    float alpha = mix(clamp(max(max(col.r, col.g), col.b), 0.0, 1.0), 1.0, cover);
    gl_FragColor = vec4(min(col, vec3(alpha)), alpha);
}
`

const REST_ANGLE = -0.72
const SPIN = (Math.PI * 2) / 22
let raf = 0
let gl: WebGLRenderingContext | null = null
let cleanup: (() => void) | null = null

function Compile(g: WebGLRenderingContext, type: number, src: string) {
    const s = g.createShader(type)!
    g.shaderSource(s, src)
    g.compileShader(s)
    if (!g.getShaderParameter(s, g.COMPILE_STATUS)) throw new Error(g.getShaderInfoLog(s) ?? 'shader')
    return s
}

onMounted(() => {
    const el = canvas.value
    if (!el) return
    try {
        gl = el.getContext('webgl', { premultipliedAlpha: true, antialias: true, alpha: true })
        if (!gl) throw new Error('webgl')
        const g = gl
        const prog = g.createProgram()!
        g.attachShader(prog, Compile(g, g.VERTEX_SHADER, VERT))
        g.attachShader(prog, Compile(g, g.FRAGMENT_SHADER, FRAG))
        g.linkProgram(prog)
        if (!g.getProgramParameter(prog, g.LINK_STATUS)) throw new Error('link')
        g.useProgram(prog)
        const buf = g.createBuffer()
        g.bindBuffer(g.ARRAY_BUFFER, buf)
        g.bufferData(g.ARRAY_BUFFER, new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]), g.STATIC_DRAW)
        const loc = g.getAttribLocation(prog, 'aPos')
        g.enableVertexAttribArray(loc)
        g.vertexAttribPointer(loc, 2, g.FLOAT, false, 0, 0)
        const U = {
            res: g.getUniformLocation(prog, 'uRes'),
            time: g.getUniformLocation(prog, 'uTime'),
            angle: g.getUniformLocation(prog, 'uAngle'),
            intro: g.getUniformLocation(prog, 'uIntro'),
        }

        const still = matchMedia('(prefers-reduced-motion: reduce)').matches
        let visible = true
        const start = performance.now()

        function Resize() {
            const dpr = Math.min(devicePixelRatio || 1, el!.clientWidth < 480 ? 1.5 : 2)
            const w = Math.max(1, Math.round(el!.clientWidth * dpr))
            const h = Math.max(1, Math.round(el!.clientHeight * dpr))
            if (el!.width !== w || el!.height !== h) {
                el!.width = w
                el!.height = h
            }
            g.viewport(0, 0, w, h)
        }

        function Frame(now: number) {
            const t = (now - start) / 1000
            const angle = still ? REST_ANGLE : REST_ANGLE - t * SPIN
            Resize()
            g.uniform2f(U.res, el!.width, el!.height)
            g.uniform1f(U.time, t)
            g.uniform1f(U.angle, angle)
            g.uniform1f(U.intro, still ? 1 : Math.min(1, t / 1.8) ** 1.5)
            g.drawArrays(g.TRIANGLE_STRIP, 0, 4)
        }

        function Loop(now: number) {
            Frame(now)
            if (visible && !document.hidden) raf = requestAnimationFrame(Loop)
            else raf = 0
        }
        function Wake() {
            if (!still && !raf && visible && !document.hidden) raf = requestAnimationFrame(Loop)
        }

        const io = new IntersectionObserver(([entry]) => {
            visible = !!entry?.isIntersecting
            Wake()
        })
        io.observe(el)
        const ro = new ResizeObserver(() => requestAnimationFrame(Frame))
        ro.observe(el)
        document.addEventListener('visibilitychange', Wake)
        el.addEventListener('webglcontextlost', () => (failed.value = true))

        if (still) requestAnimationFrame(Frame)
        else Wake()

        cleanup = () => {
            io.disconnect()
            ro.disconnect()
            document.removeEventListener('visibilitychange', Wake)
        }
    } catch {
        failed.value = true
    }
})

onUnmounted(() => {
    cancelAnimationFrame(raf)
    cleanup?.()
})
</script>

<template>
    <img v-if="failed" class="eclipse" :src="props.fallback" alt="" width="1400" height="1400" />
    <canvas v-else ref="canvas" class="eclipse" aria-hidden="true"></canvas>
</template>

<style scoped>
.eclipse {
    display: block;
    aspect-ratio: 1;
}
</style>
