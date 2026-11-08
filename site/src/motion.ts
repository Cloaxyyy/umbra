import { animate, createTimeline, stagger } from 'animejs'
import gsap from 'gsap'
import { ScrollTrigger } from 'gsap/ScrollTrigger'
import Lenis from 'lenis'

const REVEAL_GROUPS = ['.langs button', '.cards .card', '.list li', '.trio li', '.faq li']

export const ReducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches

function Intro() {
    const tl = createTimeline({ defaults: { ease: 'outExpo', duration: 1400 } })
    tl.add('.hero .pill', { opacity: [0, 1], y: [14, 0] }, 0)
        .add('.hero h1', { opacity: [0, 1], y: [34, 0], filter: ['blur(12px)', 'blur(0px)'] }, 120)
        .add('.hero .lead', { opacity: [0, 1], y: [20, 0] }, 300)
        .add('.hero .cta > *', { opacity: [0, 1], y: [14, 0], delay: stagger(90) }, 420)
        .add('.hero-art .eclipse', { opacity: [0, 1], scale: [0.86, 1], duration: 2200 }, 0)
}

function Reveals() {
    for (const section of gsap.utils.toArray<HTMLElement>('.section, .final')) {
        const heads = section.querySelectorAll<HTMLElement>(
            ':scope > .tile, :scope > h2, :scope > .sub, :scope > div > h2, :scope > div > .sub, .final-inner > *',
        )
        if (!heads.length) continue
        gsap.from(heads, {
            opacity: 0,
            y: 36,
            duration: 1.1,
            ease: 'power3.out',
            stagger: 0.09,
            scrollTrigger: { trigger: section, start: 'top 82%', once: true },
        })
    }

    const small = matchMedia('(max-width: 640px)').matches
    for (const win of gsap.utils.toArray<HTMLElement>('.window')) {
        gsap.fromTo(
            win,
            {
                rotateX: small ? 10 : 22,
                y: small ? 36 : 80,
                scale: small ? 0.97 : 0.94,
                opacity: 0.35,
                transformPerspective: 1400,
                transformOrigin: '50% 0%',
            },
            {
                rotateX: 0,
                y: 0,
                scale: 1,
                opacity: 1,
                ease: 'none',
                scrollTrigger: { trigger: win, start: 'top 98%', end: 'top 42%', scrub: 0.6 },
            },
        )
    }

    for (const sel of REVEAL_GROUPS) {
        const items = gsap.utils.toArray<HTMLElement>(sel)
        if (!items.length) continue
        gsap.set(items, { opacity: 0 })
        ScrollTrigger.create({
            trigger: items[0],
            start: 'top 88%',
            once: true,
            onEnter: () => animate(items, { opacity: [0, 1], y: [26, 0], delay: stagger(70), duration: 1000, ease: 'outExpo' }),
        })
    }

    gsap.to('.hero-art', {
        yPercent: 22,
        scale: 0.88,
        opacity: 0.5,
        ease: 'none',
        scrollTrigger: { trigger: '.hero', start: 'top top', end: 'bottom top', scrub: true },
    })
    gsap.to('.hero-text', {
        yPercent: -12,
        opacity: 0.2,
        ease: 'none',
        scrollTrigger: { trigger: '.hero', start: 'top top', end: 'bottom top', scrub: true },
    })

    gsap.to('.beams span', {
        xPercent: 5,
        duration: 10,
        ease: 'sine.inOut',
        yoyo: true,
        repeat: -1,
        stagger: { each: 1.7, from: 'random' },
    })
}

export function SwapShot(img: Element | null) {
    if (!img || ReducedMotion()) return
    animate(img, { opacity: [0, 1], scale: [1.015, 1], duration: 700, ease: 'outQuart' })
}

export function StartMotion(): () => void {
    if (ReducedMotion()) return () => {}
    gsap.registerPlugin(ScrollTrigger)
    const lenis = new Lenis({ lerp: 0.09, anchors: { offset: -72 } })
    lenis.on('scroll', ScrollTrigger.update)
    const tick = (time: number) => lenis.raf(time * 1000)
    gsap.ticker.add(tick)
    gsap.ticker.lagSmoothing(0)
    Intro()
    Reveals()
    const refresh = () => ScrollTrigger.refresh()
    if (document.readyState === 'complete') requestAnimationFrame(refresh)
    else window.addEventListener('load', refresh, { once: true })
    return () => {
        window.removeEventListener('load', refresh)
        gsap.ticker.remove(tick)
        ScrollTrigger.getAll().forEach((t) => t.kill())
        lenis.destroy()
    }
}
