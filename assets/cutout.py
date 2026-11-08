"""Cut the eclipse out of its dark rounded-square background: solid black disc, glowing ring as soft alpha."""
import sys
import numpy as np
from PIL import Image

src, dst = sys.argv[1], sys.argv[2]
img = np.asarray(Image.open(src).convert("RGB")).astype(np.float32)
h, w, _ = img.shape
m = img.max(axis=2)

ys, xs = np.nonzero(m > 90)
A = np.c_[xs, ys, np.ones_like(xs)]
bx, by, c = np.linalg.lstsq(A, xs**2 + ys**2, rcond=None)[0]
cx, cy = bx / 2, by / 2
R = np.sqrt(c + cx**2 + cy**2)
N = 720
r_in = np.full(N, np.nan)
for i, t in enumerate(np.linspace(0, 2 * np.pi, N, endpoint=False)):
    rr = np.arange(R * 0.8, R * 1.15, 0.5)
    x = (cx + rr * np.cos(t)).astype(int); y = (cy + rr * np.sin(t)).astype(int)
    ok = (x >= 0) & (x < w) & (y >= 0) & (y < h)
    if ok.any():
        r_in[i] = rr[ok][np.argmax(m[y[ok], x[ok]])]
r_in = np.where(np.isnan(r_in), np.nanmedian(r_in), r_in)
pad = np.r_[r_in[-8:], r_in, r_in[:8]]
r_in = np.array([np.median(pad[i:i + 17]) for i in range(N)])
r_disc = r_in.min()

yy, xx = np.mgrid[0:h, 0:w]
dist = np.hypot(yy - cy, xx - cx)

T, k = 22.0, 2.6
alpha = np.clip((m - T) * k, 0, 255)
alpha *= np.clip((R * 1.55 - dist) / (R * 0.25), 0, 1)
ang = (np.arctan2(yy - cy, xx - cx) % (2 * np.pi)) / (2 * np.pi) * N
inside = dist < r_in[ang.astype(int) % N]
alpha[inside] = 255

a = np.maximum(alpha, 1)[..., None] / 255.0
rgb = np.where(inside[..., None], img, np.clip(img / a, 0, 255))
out = np.dstack([rgb, alpha]).astype(np.uint8)

vis = np.nonzero(alpha > 6)
y0, y1, x0, x1 = vis[0].min(), vis[0].max(), vis[1].min(), vis[1].max()
side = int(max(y1 - y0, x1 - x0) * 1.06)
cyc, cxc = (y0 + y1) // 2, (x0 + x1) // 2
canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
crop = Image.fromarray(out).crop((cxc - side // 2, cyc - side // 2, cxc - side // 2 + side, cyc - side // 2 + side))
canvas.alpha_composite(crop)
final = canvas.resize((1024, 1024), Image.LANCZOS)
final.save(dst)

fa = np.asarray(final)[..., 3]
assert fa[0, 0] == 0 and fa[-1, -1] == 0 and fa[0, -1] == 0 and fa[-1, 0] == 0, "corners not transparent"
assert fa[512, 512] == 255, "disc not solid"
print(f"centre=({cx:.0f},{cy:.0f}) R={R:.0f} r_disc={r_disc:.0f} visible_box={x1-x0}x{y1-y0} -> {dst}")
