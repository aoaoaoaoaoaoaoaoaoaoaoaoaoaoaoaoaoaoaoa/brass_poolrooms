(() => {
"use strict";

try {
  if (!self.WebGL2RenderingContext || !self.Promise || !self.PointerEvent || !self.matchMedia ||
      matchMedia("(prefers-reduced-motion: reduce)").matches ||
      (navigator.connection && navigator.connection.saveData)) return;
} catch {
  return;
}

const script = document.currentScript;
if (!script || !script.src) return;
const assetRoot = new URL(".", script.src);

const TILE_PITCH = 252;
const WAKE_SLOTS = 6;
const FIELD_LIMIT = 8 * 1024 * 1024;
const DELUGE = 2;

const VERTEX = `#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`;

const FRAGMENT = `#version 300 es
precision highp float;

const int WAKE_SLOTS = ${WAKE_SLOTS};
const float TILE_PITCH = ${TILE_PITCH}.0;
const float WAVE_V = 320.0;
const float WAVE_SIGMA = 14.0;
const float WAVE_DAMP = 4.8;
const float WAVE_SPREAD = 480.0;
const float REFRACT_PX = 2.0;
const float IOR_SPREAD = 0.68;

uniform sampler2D floor_tex;
uniform vec2 viewport;
uniform vec2 origin;
uniform float density;
uniform float tide;
uniform vec4 wakes[WAKE_SLOTS];
out vec4 color;

vec2 touch_flow(vec2 px, vec4 touch) {
  float age = tide - touch.z;
  if (touch.w <= 0.0 || age < 0.0) return vec2(0.0);

  vec2 ray = px - touch.xy;
  float travel = WAVE_V * age;
  float reach = 4.0 * WAVE_SIGMA + 0.05 * travel;
  float square = max(abs(ray.x), abs(ray.y));
  if (square > travel + reach) return vec2(0.0);
  if (travel > reach && square < (travel - reach) * 0.70710678) return vec2(0.0);

  float d = length(ray);
  if (abs(d - travel) > reach) return vec2(0.0);
  float a = touch.w * exp(-age / WAVE_DAMP) / sqrt(1.0 + d / WAVE_SPREAD);
  float s = (d - travel) / WAVE_SIGMA;
  return ray / max(d, 1e-3) * (a * s * exp(-0.5 * s * s));
}

vec3 submerged(vec2 px, vec2 flow) {
  vec2 g = flow * REFRACT_PX;
  vec2 r = (px + g * (1.0 - IOR_SPREAD)) / TILE_PITCH;
  vec2 m = (px + g) / TILE_PITCH;
  vec2 b = (px + g * (1.0 + IOR_SPREAD)) / TILE_PITCH;
  return vec3(texture(floor_tex, r).r, texture(floor_tex, m).g, texture(floor_tex, b).b);
}

void main() {
  vec2 screen_px = vec2(gl_FragCoord.x, viewport.y - gl_FragCoord.y) / density;
  vec2 document_px = screen_px + origin;
  vec2 flow = vec2(0.0);
  for (int i = 0; i < WAKE_SLOTS; ++i) flow += touch_flow(document_px, wakes[i]);
  color = vec4(submerged(document_px, flow), 1.0);
}`;

class Riptide {
  static forge() {
    const tileDpr = Math.max(1, Math.ceil(devicePixelRatio));
    if (!isFinite(tileDpr) || tileDpr > 3) return null;
    const canvas = document.createElement("canvas");
    canvas.className = "riptide";
    canvas.setAttribute("aria-hidden", "true");
    canvas.style.cssText = "position:absolute;z-index:0;opacity:0;pointer-events:none";
    canvas.width = canvas.height = 1;
    const hush = event => event.preventDefault();
    canvas.addEventListener("webglcontextcreationerror", hush);
    const gl = canvas.getContext("webgl2", {
      alpha: true,
      antialias: false,
      depth: false,
      desynchronized: true,
      failIfMajorPerformanceCaveat: true,
      powerPreference: "low-power",
      premultipliedAlpha: false,
      preserveDrawingBuffer: false,
      stencil: false,
    });
    canvas.removeEventListener("webglcontextcreationerror", hush);
    if (!gl) return null;

    const image = new Image();
    return new Promise((resolve, reject) => {
      image.onload = () => resolve(image);
      image.onerror = reject;
      image.src = new URL(
        tileDpr === 1 ? "brass-tiles.png" : `brass-tiles@${tileDpr}x.png`,
        assetRoot,
      ).href;
    }).then(tile => {
      const program = gl.createProgram();
      const sources = [[gl.VERTEX_SHADER, VERTEX], [gl.FRAGMENT_SHADER, FRAGMENT]];
      for (let i = 0; i < sources.length; ++i) {
        const shader = gl.createShader(sources[i][0]);
        gl.shaderSource(shader, sources[i][1]);
        gl.compileShader(shader);
        if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw 0;
        gl.attachShader(program, shader);
        gl.deleteShader(shader);
      }
      gl.linkProgram(program);
      if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw 0;
      gl.useProgram(program);
      gl.bindVertexArray(gl.createVertexArray());

      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, gl.createTexture());
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, tile);

      const uniform = name => gl.getUniformLocation(program, name);
      gl.uniform1i(uniform("floor_tex"), 0);
      gl.clearColor(0, 0, 0, 0);
      return new Riptide(canvas, gl, tileDpr, {
        density: uniform("density"),
        origin: uniform("origin"),
        tide: uniform("tide"),
        viewport: uniform("viewport"),
        wakes: uniform("wakes[0]"),
      });
    }).catch(() => {
      Riptide.quench(gl);
      return null;
    });
  }

  static quench(gl) {
    try {
      const extinction = gl.getExtension("WEBGL_lose_context");
      if (extinction) extinction.loseContext();
    } catch {}
  }

  constructor(canvas, gl, tileDpr, uniforms) {
    this._canvas = canvas;
    this._gl = gl;
    this._tileDpr = tileDpr;
    this._uniforms = uniforms;
    this._wakes = new Float32Array(WAKE_SLOTS * 4);
    this._victim = 0;
    this._frame = 0;
    this._motion = null;
    this._flood = [];
    this._field = null;
    this._dead = false;
    this._unbind = [];
  }

  bind() {
    const strike = (event, impulse) => this._strike(event.clientX + scrollX, event.clientY + scrollY, impulse * DELUGE);
    const wetted = node => {
      if (!(node instanceof Element)) return null;
      return node.closest(".poolrooms-frame,.plate") || node.closest("a,button");
    };
    this._listen(self, "pointerdown", event => strike(event, 1.6), { passive: true });
    this._listen(self, "pointermove", event => this._trail(event), { passive: true });
    this._listen(self, "pointerover", event => {
      const target = wetted(event.target);
      if (target && !target.contains(event.relatedTarget)) strike(event, 0.9);
    }, { passive: true });
    this._listen(self, "pointerout", event => {
      const target = wetted(event.target);
      if (target && !target.contains(event.relatedTarget)) strike(event, 0.42);
    }, { passive: true });
    this._listen(self, "focusin", event => {
      const target = wetted(event.target);
      if (!target) return;
      const rect = target.getBoundingClientRect();
      this._strike(rect.left + rect.width / 2 + scrollX, rect.top + rect.height / 2 + scrollY, 0.9 * DELUGE);
    });
    const retreat = () => {
      this._motion = null;
      if (this._canvas.parentNode || this._frame) this._still("ready");
    };
    this._listen(self, "scroll", retreat, { passive: true });
    this._listen(self, "resize", retreat, { passive: true });
    this._listen(document, "visibilitychange", () => {
      if (document.hidden) this._still("ready");
    });
    const motion = matchMedia("(prefers-reduced-motion: reduce)");
    const quell = event => { if (event.matches) this._abort(); };
    if (motion.addEventListener) this._listen(motion, "change", quell);
    else {
      motion.addListener(quell);
      this._unbind.push(() => motion.removeListener(quell));
    }
    const connection = navigator.connection;
    if (connection && connection.addEventListener) {
      this._listen(connection, "change", () => { if (connection.saveData) this._abort(); });
    }
    this._listen(self, "pagehide", event => { if (!event.persisted) this._abort(); });
    this._listen(this._canvas, "webglcontextlost", () => this._abort());
    document.documentElement.setAttribute("data-riptide", "ready");
  }

  _listen(target, type, effect, options) {
    const safe = event => {
      if (this._dead) return;
      try {
        effect(event);
      } catch {
        this._abort();
      }
    };
    target.addEventListener(type, safe, options);
    this._unbind.push(() => target.removeEventListener(type, safe, options));
  }

  _trail(event) {
    if (event.pointerType !== "mouse") return;
    const now = performance.now();
    const next = [event.clientX + scrollX, event.clientY + scrollY, now];
    if (!this._motion) {
      this._motion = next;
      return;
    }
    const x = this._motion[0];
    const y = this._motion[1];
    const then = this._motion[2];
    const distance = Math.hypot(next[0] - x, next[1] - y);
    if (distance < 54 || now - then < 72) return;
    this._motion = next;
    this._strike(next[0], next[1], Math.min(0.72, 0.2 + distance / Math.max(now - then, 1) * 0.15) * DELUGE);
  }

  _strike(x, y, impulse) {
    if (this._dead) return;
    if (!this._canvas.parentNode && !this._raise()) {
      this._abort();
      return;
    }
    const slot = this._victim++ % WAKE_SLOTS * 4;
    this._wakes.set([x, y, performance.now() / 1000, impulse], slot);
    this._summon();
  }

  _raise() {
    const width = innerWidth;
    const height = innerHeight;
    const density = devicePixelRatio;
    if (!isFinite(density) || width <= 0 || height <= 0 || density <= 0 ||
        density > this._tileDpr) return false;
    const pw = Math.ceil(width * density);
    const ph = Math.ceil(height * density);
    const viewport = this._gl.getParameter(this._gl.MAX_VIEWPORT_DIMS);
    const edge = this._gl.getParameter(this._gl.MAX_RENDERBUFFER_SIZE);
    if (pw * ph > FIELD_LIMIT || pw > edge || ph > edge ||
        pw > viewport[0] || ph > viewport[1]) return false;

    const ox = scrollX;
    const oy = scrollY;
    this._field = [density, ox, oy];
    this._canvas.style.left = `${ox}px`;
    this._canvas.style.top = `${oy}px`;
    this._canvas.style.width = `${width}px`;
    this._canvas.style.height = `${height}px`;
    this._canvas.width = pw;
    this._canvas.height = ph;
    if (this._gl.drawingBufferWidth !== pw || this._gl.drawingBufferHeight !== ph) return false;
    this._gl.viewport(0, 0, pw, ph);
    this._excavate(width, height, density);
    document.body.insertBefore(this._canvas, document.body.firstChild);
    return true;
  }

  _summon() {
    if (!this._dead && !document.hidden && !this._frame) {
      this._frame = requestAnimationFrame(now => {
        try {
          this._render(now);
        } catch {
          this._abort();
        }
      });
    }
  }

  _still(state) {
    if (this._frame) cancelAnimationFrame(this._frame);
    this._frame = 0;
    this._wakes.fill(0);
    this._canvas.style.opacity = 0;
    if (this._canvas.parentNode) this._canvas.parentNode.removeChild(this._canvas);
    this._canvas.width = this._canvas.height = 1;
    this._field = null;
    if (state) document.documentElement.setAttribute("data-riptide", state);
    else document.documentElement.removeAttribute("data-riptide");
  }

  _abort() {
    if (this._dead) return;
    this._dead = true;
    while (this._unbind.length) {
      try {
        this._unbind.pop()();
      } catch {}
    }
    Riptide.quench(this._gl);
    try {
      this._still(null);
    } catch {}
  }

  _excavate(width, height, density) {
    const plates = Array.prototype.map.call(
      document.querySelectorAll(".poolrooms-frame,.plate"),
      plate => {
        const rect = plate.getBoundingClientRect();
        return {
          left: Math.max(0, rect.left),
          top: Math.max(0, rect.top),
          right: Math.min(width, rect.right),
          bottom: Math.min(height, rect.bottom),
        };
      },
    ).filter(rect => rect.left < rect.right && rect.top < rect.bottom);
    const cuts = [0, height];
    for (let i = 0; i < plates.length; ++i) cuts.push(plates[i].top, plates[i].bottom);
    cuts.sort((a, b) => a - b);
    for (let cut = cuts.length - 1; cut > 0; --cut) {
      if (cuts[cut] === cuts[cut - 1]) cuts.splice(cut, 1);
    }
    const flood = [];
    for (let band = 1; band < cuts.length; ++band) {
      const top = cuts[band - 1];
      const bottom = cuts[band];
      const dams = plates.filter(rect => rect.top < bottom && rect.bottom > top)
        .map(rect => [rect.left, rect.right]).sort((a, b) => a[0] - b[0]);
      let left = 0;
      for (let dam = 0; dam < dams.length; ++dam) {
        if (dams[dam][0] > left) flood.push([left, top, dams[dam][0], bottom]);
        left = Math.max(left, dams[dam][1]);
      }
      if (left < width) flood.push([left, top, width, bottom]);
    }
    this._flood = flood.map(field => {
      const x0 = Math.floor(field[0] * density);
      const x1 = Math.ceil(field[2] * density);
      const y0 = Math.floor((height - field[3]) * density);
      const y1 = Math.ceil((height - field[1]) * density);
      return [x0, y0, x1 - x0, y1 - y0];
    });
  }

  _render(now) {
    this._frame = 0;
    const [density, ox, oy] = this._field;
    const width = innerWidth;
    const height = innerHeight;
    const pw = this._canvas.width;
    const ph = this._canvas.height;
    if (devicePixelRatio !== density || pw !== Math.ceil(width * density) ||
        ph !== Math.ceil(height * density)) {
      this._still("ready");
      return;
    }

    const tide = now / 1000;
    const life = (Math.hypot(width, height) + 4 * 14) / 320;
    let alive = false;
    for (let slot = 2; slot < this._wakes.length; slot += 4) {
      if (this._wakes[slot + 1] > 0 && tide - this._wakes[slot] < life) alive = true;
      else this._wakes[slot + 1] = 0;
    }
    this._gl.uniform1f(this._uniforms.density, density);
    this._gl.uniform2f(this._uniforms.origin, ox, oy);
    this._gl.uniform1f(this._uniforms.tide, tide);
    this._gl.uniform2f(this._uniforms.viewport, pw, ph);
    this._gl.uniform4fv(this._uniforms.wakes, this._wakes);

    this._gl.disable(this._gl.SCISSOR_TEST);
    this._gl.clear(this._gl.COLOR_BUFFER_BIT);
    this._gl.enable(this._gl.SCISSOR_TEST);
    for (let field = 0; field < this._flood.length; ++field) {
      this._gl.scissor(...this._flood[field]);
      this._gl.drawArrays(this._gl.TRIANGLES, 0, 3);
    }
    this._canvas.style.opacity = 1;
    document.documentElement.setAttribute("data-riptide", "live");

    if (alive) this._summon();
    else this._still("ready");
  }
}

Promise.resolve().then(() => Riptide.forge()).then(riptide => {
  if (!riptide) return;
  try {
    riptide.bind();
  } catch {
    riptide._abort();
  }
}, () => {});
})();
