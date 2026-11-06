package com.octaviantocan.mymind.ui.sky

import android.graphics.RuntimeShader
import android.os.Build
import androidx.annotation.ChecksSdkIntAtLeast
import androidx.annotation.RequiresApi

/**
 * The prototype's GLSL, as AGSL for Android 13 and later. Each works in linear light as the prototype does and gives
 * sRGB, so what it adds glows as it does there (see [Srgb]). Older phones draw the sky without them.
 */
internal object SkyShaders {
    /** True when this phone runs AGSL. */
    @ChecksSdkIntAtLeast(api = Build.VERSION_CODES.TIRAMISU)
    val supported: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU

    /** The sky's dome: a dark sky, a band of nebula with dust lanes and a warm core, and dusk low down while looking up. */
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun dome(): RuntimeShader = RuntimeShader(DOME)

    /** One line of a constellation: a lit gold line with travelling pulses, or a dotted dim one. */
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun line(): RuntimeShader = RuntimeShader(LINE)

    /** A find as smoke in the sky behind its star, the Condense view's ghost. */
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun smoke(): RuntimeShader = RuntimeShader(SMOKE)

    /** The ring a shooting star throws out as it lands. */
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun shock(): RuntimeShader = RuntimeShader(SHOCK)

    /** A Back today find shown as a flickering blue hologram, restored from where it is tapped. */
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    fun hologram(): RuntimeShader = RuntimeShader(HOLOGRAM)

    private const val ENCODE = """
        float encode1(float x) { x = clamp(x, 0.0, 1.0); return x <= 0.0031308 ? x * 12.92 : 1.055 * pow(x, 1.0 / 2.4) - 0.055; }
        vec3 encode(vec3 c) { return vec3(encode1(c.r), encode1(c.g), encode1(c.b)); }
        float decode1(float x) { return x <= 0.04045 ? x / 12.92 : pow((x + 0.055) / 1.055, 2.4); }
        vec3 decode(vec3 c) { return vec3(decode1(c.r), decode1(c.g), decode1(c.b)); }
    """

    private const val DOME = """
        uniform float2 uSize;
        uniform float uTan;
        uniform float uPitch;
        uniform float uSpin;
        uniform float uTw;
        $ENCODE
        float h(vec3 p) { p = fract(p * 0.1031); p += dot(p, p.zyx + 31.32); return fract((p.x + p.y) * p.z); }
        float n3(vec3 p) {
            vec3 i = floor(p); vec3 f = fract(p); f = f * f * (3.0 - 2.0 * f);
            return mix(mix(mix(h(i), h(i + vec3(1, 0, 0)), f.x), mix(h(i + vec3(0, 1, 0)), h(i + vec3(1, 1, 0)), f.x), f.y),
                       mix(mix(h(i + vec3(0, 0, 1)), h(i + vec3(1, 0, 1)), f.x), mix(h(i + vec3(0, 1, 1)), h(i + vec3(1, 1, 1)), f.x), f.y), f.z);
        }
        float fbm(vec3 p) { float s = 0.0; float a = 0.5; for (int i = 0; i < 5; i++) { s += a * n3(p); p = p * 2.02 + vec3(3.1, 1.7, 4.3); a *= 0.5; } return s; }
        half4 main(float2 at) {
            vec2 n = vec2(at.x / uSize.x * 2.0 - 1.0, 1.0 - at.y / uSize.y * 2.0);
            vec3 v = normalize(vec3(n.x * uTan * uSize.x / uSize.y, n.y * uTan, -1.0));
            float c = cos(uPitch); float s = sin(uPitch);
            vec3 w = vec3(v.x, v.y * c - v.z * s, v.y * s + v.z * c);
            vec3 ax = vec3(0.0, c, s);
            float ca = cos(uSpin); float sa = -sin(uSpin);
            vec3 d = normalize(w * ca + cross(ax, w) * sa + ax * dot(ax, w) * (1.0 - ca));
            vec3 N = normalize(vec3(0.707, -0.198, -0.679));
            vec3 P = d * 9.0;
            float b = dot(d, N) + (fbm(P * 0.5) - 0.5) * 0.09;
            float band = exp(-b * b / 0.006); float wide = exp(-b * b / 0.04);
            vec3 q = vec3(fbm(P), fbm(P + 5.2), fbm(P + 9.7));
            vec3 r = vec3(fbm(P * 1.5 + q * 2.4), fbm(P * 1.5 + q * 2.4 + 3.3), 0.0);
            float cl = fbm(P * 2.0 + vec3(r.xy * 2.6, 0.0));
            float fine = fbm(P * 7.0 + r * 3.0);
            vec3 sky = mix(vec3(0.0025, 0.005, 0.012), vec3(0.004, 0.009, 0.024), smoothstep(-0.2, 0.9, d.y));
            vec3 neb = vec3(0.012, 0.04, 0.09) * pow(cl, 2.2) * wide * 2.0;
            neb += vec3(0.03, 0.08, 0.12) * pow(cl, 3.0) * band * (0.5 + fine) * 1.6;
            neb += vec3(0.13, 0.03, 0.09) * pow(fbm(P * 2.6 + r * 1.7 + 11.0), 3.5) * band * 1.8;
            neb += vec3(0.34, 0.22, 0.11) * pow(cl, 4.0) * band * (0.4 + 1.1 * fine) * 2.2;
            vec3 core = normalize(vec3(0.3, 0.93, -0.2));
            float cg = pow(max(dot(d, core), 0.0), 60.0);
            neb += vec3(0.5, 0.33, 0.17) * cg * band * pow(cl, 1.2) * 1.6;
            float dust = smoothstep(0.44, 0.7, fbm(P * 3.4 + r * 2.5 + 2.0)) * smoothstep(0.0, 0.5, band);
            vec3 col = sky + neb * (1.0 - dust * 0.92);
            float y = d.y;
            vec3 dusk = mix(vec3(1.0, 0.64, 0.40), vec3(0.74, 0.40, 0.40), smoothstep(0.15, 0.38, y));
            dusk = mix(dusk, vec3(0.26, 0.16, 0.34), smoothstep(0.34, 0.6, y));
            dusk = mix(dusk, vec3(0.03, 0.045, 0.11), smoothstep(0.55, 0.92, y));
            col = mix(col, dusk + col * 0.4, uTw);
            return half4(encode(col), 1.0);
        }
    """

    // Local units: x along the line from -uLen/2 to uLen/2, y across it.
    private const val LINE = """
        uniform float uLen;
        uniform float uLit;
        uniform float uDraw;
        uniform float uTime;
        uniform float uOp;
        uniform float uSurge;
        uniform float uBloom;
        uniform float uHalf;
        uniform float uK;
        uniform float uT;
        $ENCODE
        half4 main(float2 p) {
            float ac = p.y; float al = p.x / uLen + 0.5;
            float trim = smoothstep(0.0, 18.0 / uLen, al) * smoothstep(1.0, 1.0 - 18.0 / uLen, al);
            float core = exp(-ac * ac / 0.9); float glow = exp(-abs(ac) / 3.5) * 0.4;
            float drawn = step(al, uDraw);
            float head = exp(-pow((al - uDraw) * uLen / 22.0, 2.0)) * step(0.001, uDraw) * step(uDraw, 0.999);
            float pulse = exp(-pow((fract(al * 0.6 - uTime * 0.18) - 0.5) * uLen / 30.0, 2.0)) * 0.5;
            float surge = exp(-pow((al - uSurge) * uLen / 30.0, 2.0)) * step(-0.5, uSurge);
            vec3 gold = vec3(1.0, 0.86, 0.56);
            // The prototype's bloom pass, its three narrower levels (1.4, 5.1 and 14.5 CSS pixels): only what is brighter
            // than half once averaged over a CSS pixel blooms, from opacity uT on (higher for a thin line), times the
            // pass's gain uBloom; uK is CSS pixels per unit here. The two wider levels are added for the whole sky.
            float o = uOp * (1.0 + pulse * uLit + head * 2.0 + surge * 3.0);
            float oo = o >= 1.0 ? o : 0.42 + (o - uT) * 0.58 / max(1.0 - uT, 0.1);
            float g = oo >= 1.0 ? 2.69 + 3.94 * (oo - 1.0) : (oo <= 0.42 ? 0.0 : 2.69 * pow((oo - 0.42) / 0.58, 0.8));
            float e = g * uK * drawn;
            float a = ac * uK;
            float edge = 1.0 - smoothstep(uHalf * 0.55, uHalf, abs(ac));
            float bloom = e * (0.0912 * exp(-a * a / 3.92) + 0.03598 * exp(-a * a / 52.02) + 0.01651 * exp(-a * a / 420.5)) * uBloom * edge;
            vec3 lit = gold * (core * 0.95 + glow) * (1.0 + pulse * uLit) * drawn + vec3(1.0, 0.95, 0.85) * (head * 2.2 + surge * 3.0) * (core * 2.0 + glow);
            float dots = smoothstep(0.32, 0.0, abs(fract(al * uLen / 11.0) - 0.5)) * exp(-ac * ac / 1.4);
            vec3 dim = vec3(0.55, 0.64, 0.76) * dots * 0.45;
            vec3 col = mix(dim, lit, uLit) * trim * uOp + gold * bloom * trim;
            return half4(encode(col), 1.0);
        }
    """

    // Local units: the square from -uSide/2 to uSide/2, y down.
    private const val SMOKE = """
        uniform shader uMap;
        uniform float2 uMapSize;
        uniform float uSide;
        uniform float uAsp;
        uniform float uOp;
        uniform float uIn;
        uniform float uTime;
        uniform float uWorn;
        $ENCODE
        float h(vec2 p) { vec3 p3 = fract(vec3(p.xyx) * 0.1031); p3 += dot(p3, p3.yzx + 33.33); return fract((p3.x + p3.y) * p3.z); }
        float n2(vec2 p) { vec2 i = floor(p); vec2 f = fract(p); f = f * f * (3.0 - 2.0 * f); return mix(mix(h(i), h(i + vec2(1, 0)), f.x), mix(h(i + vec2(0, 1)), h(i + vec2(1, 1)), f.x), f.y); }
        float fbm(vec2 p) { float s = 0.0; float a = 0.5; for (int i = 0; i < 5; i++) { s += a * n2(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; } return s / 0.97; }
        float L(vec3 c) { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
        vec2 fit(vec2 p, float z) { return 0.5 + p * z * (uAsp > 1.0 ? vec2(1.0 / uAsp, 1.0) : vec2(1.0, uAsp)); }
        vec3 pic(vec2 uv) { uv = clamp(uv, 0.002, 0.998); return decode(uMap.eval(vec2(uv.x, 1.0 - uv.y) * uMapSize).rgb); }
        half4 main(float2 at) {
            vec2 vUv = vec2(at.x / uSide + 0.5, 0.5 - at.y / uSide);
            vec2 p = vUv - 0.5; float t = uTime; float rev = clamp(uIn, 0.0, 1.0);
            float drift = 1.0 - 0.04 * sin(t * 0.07);
            vec2 uv = fit(p, drift) + vec2(sin(t * 0.05), cos(t * 0.04)) * 0.012;
            vec3 c = pic(uv); float l = L(c);
            float wisp = fbm(p * 4.0 + vec2(t * 0.02, 0.0)); float rr = length(p * vec2(1.0, 1.05));
            float m = 1.0 - smoothstep(0.18, 0.5, rr + (wisp - 0.5) * 0.32);
            float grain = 0.85 + 0.3 * h(floor(vUv * uSide * 0.9));
            vec3 smoke = mix(vec3(0.06, 0.16, 0.36), vec3(1.0, 0.86, 0.6), smoothstep(0.3, 0.95, l)) * (0.25 + l * 1.3) + c * 0.18;
            vec3 col = smoke * m * grain * 0.42 * smoothstep(0.5, 1.0, rev);
            float lw = L(col); col = mix(col, vec3(lw) * vec3(0.62, 0.72, 0.95), uWorn * 0.55) * (1.0 - uWorn * 0.3);
            return half4(encode(col * uOp), 1.0);
        }
    """

    private const val SHOCK = """
        uniform float2 uCenter;
        uniform float uRadius;
        uniform float uP;
        uniform float uOp;
        $ENCODE
        half4 main(float2 at) {
            float r = length(at - uCenter) / uRadius;
            float ring = exp(-pow((r - uP) / (0.008 + 0.02 * uP), 2.0));
            float inner = (1.0 - smoothstep(0.0, max(uP, 0.001), r)) * 0.08 * (1.0 - uP);
            return half4(encode(vec3(1.0, 0.88, 0.62) * (ring + inner) * uOp), 1.0);
        }
    """

    // The prototype's look 25 on a card's face, in its units (one per dp), y up; uOrigin is where the tap was.
    private const val HOLOGRAM = """
        uniform shader content;
        uniform float2 uPx;
        uniform float2 uSize;
        uniform float2 uOrigin;
        uniform float uTime;
        uniform float uStage;
        uniform float uRestore;
        uniform float uShine;
        $ENCODE
        float h(vec2 p) { vec3 p3 = fract(vec3(p.xyx) * 0.1031); p3 += dot(p3, p3.yzx + 33.33); return fract((p3.x + p3.y) * p3.z); }
        float n2(vec2 p) { vec2 i = floor(p); vec2 f = fract(p); f = f * f * (3.0 - 2.0 * f); return mix(mix(h(i), h(i + vec2(1, 0)), f.x), mix(h(i + vec2(0, 1)), h(i + vec2(1, 1)), f.x), f.y); }
        float fbm(vec2 p) { float s = 0.0; float a = 0.5; for (int i = 0; i < 4; i++) { s += a * n2(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; } return s / 0.9375; }
        float lum(vec3 c) { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
        vec4 raw(vec2 uv) { return content.eval(vec2(uv.x, 1.0 - uv.y) * uPx); }
        vec3 tg(vec2 uv) { vec4 c = raw(uv); return c.a > 0.0 ? c.rgb / c.a : vec3(0.0); }
        float edgeL(vec2 uv, vec2 t) {
            float a = lum(tg(uv + vec2(t.x, 0.0))); float b = lum(tg(uv - vec2(t.x, 0.0)));
            float c = lum(tg(uv + vec2(0.0, t.y))); float d = lum(tg(uv - vec2(0.0, t.y)));
            return length(vec2(a - b, c - d));
        }
        half4 main(float2 at) {
            vec2 vUv = vec2(at.x / uPx.x, 1.0 - at.y / uPx.y);
            vec2 px = (vUv - 0.5) * uSize;
            vec4 tx = raw(vUv);
            float alpha = tx.a;
            vec3 cg = tx.a > 0.0 ? tx.rgb / tx.a : vec3(0.0);
            vec3 clean = decode(cg);
            vec3 col = clean;
            float md = max(uSize.x, uSize.y);
            float nz = fbm(vUv * uSize / 26.0);
            float dist = length((vUv - uOrigin) * uSize) / md + (nz - 0.5) * 0.22;
            float front = uRestore * 1.7 - 0.12;
            float revealed = 1.0 - smoothstep(front - 0.05, front + 0.01, dist);
            float live = step(0.001, uRestore) * step(uRestore, 0.999);
            float band = exp(-pow((dist - front) / 0.03, 2.0)) * live;
            float s = uStage; float s3 = clamp(s - 2.0, 0.0, 1.0);
            float l = lum(cg);
            float K = min(clamp(s / 3.0, 0.0, 1.0), 1.0);
            vec2 t1 = 1.0 / uSize;
            float bgL = lum(tg(vec2(0.04, 0.5)));
            float jx = sin(px.y * 0.4 + uTime * 20.0) * step(0.92, h(vec2(floor(px.y / 5.0), floor(uTime * 6.0)))) * 4.0;
            float ed = edgeL(vUv + vec2(jx * t1.x, 0.0), t1);
            float inten = bgL > 0.75 ? 1.0 - l : l;
            vec3 holo = vec3(0.15, 0.62, 1.0) * (inten * 0.85 + 0.12) + vec3(0.6, 0.95, 1.0) * smoothstep(0.08, 0.35, ed) * 0.7;
            holo *= (0.78 + 0.22 * sin(px.y * 2.1 - uTime * 5.0)) * (0.92 + 0.08 * step(0.7, h(vec2(floor(uTime * 14.0), 1.0))));
            holo *= 1.0 - 0.25 * s3 * h(vec2(floor(px.y / 2.0), floor(uTime * 3.0)));
            vec3 worn = mix(cg, vec3(0.03, 0.07, 0.13) + holo, clamp(K * 1.6, 0.0, 1.0));
            float wa = mix(1.0, 0.85, K);
            worn = pow(max(worn, 0.0), vec3(2.2));
            col = mix(clean, worn, 1.0 - revealed) + vec3(0.6, 0.95, 1.0) * band * 0.3;
            alpha *= mix(1.0, wa, 1.0 - revealed);
            float sw = uShine * 1.7 - 0.35; float ddv = vUv.x * 0.55 + (1.0 - vUv.y) * 0.45;
            col += vec3(1.0, 0.97, 0.9) * exp(-pow((ddv - sw) / 0.07, 2.0)) * step(0.001, uShine) * step(uShine, 0.999) * 0.16;
            return half4(encode(col) * alpha, alpha);
        }
    """
}
