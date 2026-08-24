"use strict";

const TOPOLOGY = Object.freeze({
  PRODUCTION: 0,
  KEYLINE: 1,
  FLAT_STAMP: 2,
  SHADOW_STAMP: 3,
  ROUNDED: 4,
  HYBRID: 5,
  INLAY: 6,
  RAISED: 7,
  WASHED: 8,
});

const OPTICS = Object.freeze({
  LEGACY: 0,
  LINEAR_BLINN: 1,
  GGX_KEY: 2,
  GGX_ENVIRONMENT: 3,
  PATINA: 4,
  ANISOTROPIC: 5,
  PREFILTERED: 6,
  QUANTIZED_FOUR: 7,
  QUANTIZED_SIX: 8,
  POLISHED: 9,
});

const ENVIRONMENT = Object.freeze({
  BLACK: 0,
  OVERHEAD: 1,
  STRIP: 2,
  SPLIT: 3,
  FURNACE: 4,
  NORTHLIGHT: 5,
});

const GLYPHS = Object.freeze([
  { glyph: "⚙", finish: 0, name: "settings" },
  { glyph: "🖉", finish: 0, name: "rename" },
  { glyph: "➚", finish: 0, name: "export" },
  { glyph: "🖫", finish: 0, name: "save" },
  { glyph: "🗑", finish: 1, name: "delete" },
  { glyph: "♥", finish: 2, name: "heart" },
]);
const HERO_GLYPHS = [0, 4, 5];
const MEDIUM_GAUGE = 24;
const SDF_SUPERSAMPLE = 8;
const SDF_TILE = MEDIUM_GAUGE * SDF_SUPERSAMPLE;
const SDF_RANGE = 8;

const PRODUCTION = Object.freeze({
  id: "P0",
  family: "PRODUCTION",
  title: "Production composite",
  topology: TOPOLOGY.PRODUCTION,
  optics: OPTICS.LEGACY,
  environment: ENVIRONMENT.BLACK,
  sootWidth: 1,
  sootReflectance: 0,
  depth: 0.96,
  bevel: 0.42,
  roughness: 0.26,
  register: "8 square-offset masks · gamma-space palette · point key",
});

const KEYLINE_WIDTHS = [0.35, 0.55, 0.75, 1.0];
const SOOT_REFLECTANCES = [0.0, 0.04, 0.10, 0.18];
const KEYLINES = SOOT_REFLECTANCES.flatMap((sootReflectance, row) =>
  KEYLINE_WIDTHS.map((sootWidth, column) => ({
    id: `K${row + 1}${column + 1}`,
    family: "KEYLINE",
    title: `Keyline ${sootWidth.toFixed(2)} / ${sootReflectance.toFixed(2)}`,
    topology: TOPOLOGY.KEYLINE,
    optics: OPTICS.LEGACY,
    environment: ENVIRONMENT.BLACK,
    sootWidth,
    sootReflectance,
    depth: 0.96,
    bevel: 0.42,
    roughness: 0.26,
    register: `wₛ ${sootWidth.toFixed(2)} px · ρₛ ${sootReflectance.toFixed(2)} · radial SDF perimeter`,
  })),
);

const DIES = [
  {
    id: "D1",
    title: "Full perimeter control",
    topology: TOPOLOGY.KEYLINE,
    sootWidth: 1.0,
    sootReflectance: 0.0,
    depth: 0.96,
    bevel: 0.42,
    register: "No relief · circular 1 px keyline · physical optics only",
  },
  {
    id: "D2",
    title: "Tempered perimeter",
    topology: TOPOLOGY.KEYLINE,
    sootWidth: 0.48,
    sootReflectance: 0.06,
    depth: 0.96,
    bevel: 0.42,
    register: "No relief · fine charcoal perimeter · acuity control",
  },
  {
    id: "D3",
    title: "Flat-bottom stamp",
    topology: TOPOLOGY.FLAT_STAMP,
    sootWidth: 0.34,
    sootReflectance: 0.035,
    depth: 0.96,
    bevel: 0.42,
    register: "Steep bronze wall · sooted floor toe · material floor",
  },
  {
    id: "D4",
    title: "Shadow-wall deposition",
    topology: TOPOLOGY.SHADOW_STAMP,
    sootWidth: 0.58,
    sootReflectance: 0.03,
    depth: 0.96,
    bevel: 0.42,
    register: "Soot confined to the key-occluded wall · directional edge",
  },
  {
    id: "D5",
    title: "Radiused punch",
    topology: TOPOLOGY.ROUNDED,
    sootWidth: 0.42,
    sootReflectance: 0.055,
    depth: 0.78,
    bevel: 0.82,
    register: "Continuous bowl normal · soft toe occlusion · no hard border",
  },
  {
    id: "D6",
    title: "Finish-sensitive hybrid",
    topology: TOPOLOGY.HYBRID,
    sootWidth: 0.30,
    sootReflectance: 0.035,
    depth: 0.86,
    bevel: 0.54,
    register: "Bright bronze V-cut · painted flat floor · common cut depth",
  },
  {
    id: "D7",
    title: "Shallow flush inlay",
    topology: TOPOLOGY.INLAY,
    sootWidth: 0.20,
    sootReflectance: 0.08,
    depth: 0.26,
    bevel: 0.24,
    register: "Near-flush fill · narrow contact occlusion · restrained relief",
  },
  {
    id: "D8",
    title: "Raised punch cameo",
    topology: TOPOLOGY.RAISED,
    sootWidth: 0.38,
    sootReflectance: 0.035,
    depth: 0.50,
    bevel: 0.46,
    register: "Positive relief · burnished crest · soot at the die shoulder",
  },
  {
    id: "D9",
    title: "Oxide-washed bowl",
    topology: TOPOLOGY.WASHED,
    sootWidth: 0.68,
    sootReflectance: 0.12,
    depth: 0.72,
    bevel: 0.68,
    register: "Broken soot deposit · radiused strike · deliberately irregular toe",
  },
].map(candidate => ({
  ...candidate,
  family: "DIE",
  optics: OPTICS.GGX_ENVIRONMENT,
  environment: ENVIRONMENT.OVERHEAD,
  roughness: 0.27,
}));

const OPTICAL_TRIALS = [
  {
    id: "O1",
    title: "Legacy palette law",
    optics: OPTICS.LEGACY,
    register: "Bronze ramp · Lambert weight · s¹² broad 0.01 · s¹²⁸ glint 3.0",
  },
  {
    id: "O2",
    title: "Linear-light Blinn",
    optics: OPTICS.LINEAR_BLINN,
    register: "Same key topology · linear evaluation · smooth filmic shoulder",
  },
  {
    id: "O3",
    title: "Bare GGX conductor",
    optics: OPTICS.GGX_KEY,
    register: "Bronze Fresnel · GGX NDF · Smith masking · black room",
  },
  {
    id: "O4",
    title: "GGX studio bronze",
    optics: OPTICS.GGX_ENVIRONMENT,
    register: "Conductor GGX · warm ceiling card · narrow side strip · floor bounce",
  },
  {
    id: "O5",
    title: "Rough patina layer",
    optics: OPTICS.PATINA,
    register: "Rough dielectric oxide over GGX conductor · attenuated substrate",
  },
  {
    id: "O6",
    title: "Tool-mark anisotropy",
    optics: OPTICS.ANISOTROPIC,
    register: "Anisotropic GGX · horizontal burnish · environment stretched by role",
  },
  {
    id: "O7",
    title: "Pixel-prefiltered GGX",
    optics: OPTICS.PREFILTERED,
    register: "Roughness widened by pixel footprint · retained subpixel highlight energy",
  },
  {
    id: "O8",
    title: "Four-tone foundry",
    optics: OPTICS.QUANTIZED_FOUR,
    register: "Prefiltered environment GGX · four luminance charges · hue retained",
  },
  {
    id: "O9",
    title: "Six-tone foundry",
    optics: OPTICS.QUANTIZED_SIX,
    register: "Prefiltered environment GGX · six luminance charges · hue retained",
  },
  {
    id: "O10",
    title: "Polished bronze provocation",
    optics: OPTICS.POLISHED,
    register: "Low roughness conductor · broadened light cards · intentionally perilous glint",
  },
].map(candidate => ({
  ...candidate,
  family: "OPTICS",
  topology: TOPOLOGY.HYBRID,
  environment: candidate.optics === OPTICS.GGX_KEY
    ? ENVIRONMENT.BLACK
    : ENVIRONMENT.OVERHEAD,
  sootWidth: 0.30,
  sootReflectance: 0.035,
  depth: 0.86,
  bevel: 0.54,
  roughness: candidate.optics === OPTICS.POLISHED ? 0.09 : 0.27,
}));

const ENVIRONMENT_TRIALS = [
  { id: "E1", title: "Black room + key", environment: ENVIRONMENT.BLACK, register: "No cards · no floor · direct key only" },
  { id: "E2", title: "Warm overhead card", environment: ENVIRONMENT.OVERHEAD, register: "Broad warm ceiling · narrow right strip · subdued floor" },
  { id: "E3", title: "Vertical strip room", environment: ENVIRONMENT.STRIP, register: "Dominant tall side strip · dim ceiling · cold floor" },
  { id: "E4", title: "Split-card workshop", environment: ENVIRONMENT.SPLIT, register: "Warm left card · cool right card · low floor return" },
  { id: "E5", title: "Furnace line", environment: ENVIRONMENT.FURNACE, register: "Low hot horizon · dark ceiling · molten reflected band" },
  { id: "E6", title: "Northlight bay", environment: ENVIRONMENT.NORTHLIGHT, register: "Cool broad window · warm floor · low-key room body" },
].map(candidate => ({
  ...candidate,
  family: "ENVIRONMENT",
  topology: TOPOLOGY.HYBRID,
  optics: OPTICS.PATINA,
  sootWidth: 0.30,
  sootReflectance: 0.035,
  depth: 0.86,
  bevel: 0.54,
  roughness: 0.27,
}));

const CANDIDATES = [...KEYLINES, ...DIES, ...OPTICAL_TRIALS, ...ENVIRONMENT_TRIALS];
const CANDIDATE_BY_ID = new Map(CANDIDATES.map(candidate => [candidate.id, candidate]));
const INITIAL = CANDIDATE_BY_ID.get("D6");

const root = document.documentElement;
const canvas = document.getElementById("forge");
const keylineMatrix = document.getElementById("keyline-matrix");
const dieGrid = document.getElementById("die-grid");
const opticsGrid = document.getElementById("optics-grid");
const environmentGrid = document.getElementById("environment-grid");
const selectedRender = document.getElementById("selected-render");
const verdictRender = document.getElementById("verdict-render");
const selectedSpec = document.getElementById("selected-spec");
const selection = document.getElementById("selection");
const fault = document.getElementById("fault");

let selected = INITIAL;
let renderer;
let renderQueued = false;

function swatch(candidate) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "swatch";
  if (candidate.family === "OPTICS" || candidate.family === "ENVIRONMENT") {
    button.classList.add("coupon");
  }
  button.dataset.candidate = candidate.id;
  button.setAttribute("aria-label", `Select ${candidate.title}`);
  button.innerHTML = `
    <h3>${candidate.id} · ${candidate.title.toUpperCase()}</h3>
    <span class="family">${candidate.family}</span>
    <div class="render-target" data-candidate="${candidate.id}"></div>
    <span class="spec-register"><strong>${candidate.register}</strong><br>${parameterRegister(candidate)}</span>
  `;
  button.addEventListener("click", () => selectCandidate(candidate));
  return button;
}

function parameterRegister(candidate) {
  return `wₛ ${candidate.sootWidth.toFixed(2)} · ρₛ ${candidate.sootReflectance.toFixed(3)} · b ${candidate.bevel.toFixed(2)} · t ${candidate.depth.toFixed(2)}`;
}

function buildStudies() {
  for (const reflectance of SOOT_REFLECTANCES) {
    const row = document.createElement("div");
    row.className = "matrix-row";
    row.innerHTML = `
      <div class="matrix-label">
        <strong>ρₛ ${reflectance.toFixed(2)}</strong>
        soot reflectance<br>wₛ increases →
      </div>
      <div class="swatch-grid"></div>
    `;
    const grid = row.querySelector(".swatch-grid");
    for (const candidate of KEYLINES.filter(item => item.sootReflectance === reflectance)) {
      grid.append(swatch(candidate));
    }
    keylineMatrix.append(row);
  }
  for (const [grid, candidates] of [
    [dieGrid, DIES],
    [opticsGrid, OPTICAL_TRIALS],
    [environmentGrid, ENVIRONMENT_TRIALS],
  ]) {
    for (const candidate of candidates) {
      grid.append(swatch(candidate));
    }
  }
}

function selectCandidate(candidate) {
  selected = candidate;
  selectedSpec.textContent = `${candidate.id} · ${candidate.title.toUpperCase()} · ${candidate.register}`;
  selection.textContent = `SELECTED · ${candidate.id} · ${parameterRegister(candidate)} · ${candidate.register}`;
  for (const button of document.querySelectorAll(".swatch")) {
    button.classList.toggle("selected", button.dataset.candidate === candidate.id);
  }
  history.replaceState(null, "", `#${candidate.id}`);
  requestRender();
}

function restoreSelection() {
  const candidate = CANDIDATE_BY_ID.get(location.hash.slice(1).toUpperCase());
  selectCandidate(candidate ?? INITIAL);
}

function requestRender() {
  if (renderQueued || !renderer) {
    return;
  }
  renderQueued = true;
  requestAnimationFrame(() => {
    renderQueued = false;
    renderer.render(document.querySelectorAll(".render-target"));
  });
}

function fail(error) {
  root.dataset.forge = "failed";
  fault.textContent = error instanceof Error ? error.message : String(error);
}

function distanceTransform(features, width, height) {
  const infinite = 1e20;
  const field = new Float64Array(width * height);
  const work = new Float64Array(Math.max(width, height));
  const result = new Float64Array(Math.max(width, height));
  const sites = new Int32Array(Math.max(width, height));
  const boundaries = new Float64Array(Math.max(width, height) + 1);

  for (let index = 0; index < field.length; index += 1) {
    field[index] = features[index] ? 0 : infinite;
  }

  const edtLine = length => {
    let last = 0;
    sites[0] = 0;
    boundaries[0] = -infinite;
    boundaries[1] = infinite;
    for (let q = 1; q < length; q += 1) {
      let crossing;
      do {
        const site = sites[last];
        crossing = ((work[q] + q * q) - (work[site] + site * site)) / (2 * q - 2 * site);
        if (crossing <= boundaries[last]) {
          last -= 1;
        }
      } while (crossing <= boundaries[last]);
      last += 1;
      sites[last] = q;
      boundaries[last] = crossing;
      boundaries[last + 1] = infinite;
    }
    last = 0;
    for (let q = 0; q < length; q += 1) {
      while (boundaries[last + 1] < q) {
        last += 1;
      }
      const delta = q - sites[last];
      result[q] = delta * delta + work[sites[last]];
    }
  };

  for (let y = 0; y < height; y += 1) {
    const offset = y * width;
    for (let x = 0; x < width; x += 1) {
      work[x] = field[offset + x];
    }
    edtLine(width);
    for (let x = 0; x < width; x += 1) {
      field[offset + x] = result[x];
    }
  }
  for (let x = 0; x < width; x += 1) {
    for (let y = 0; y < height; y += 1) {
      work[y] = field[y * width + x];
    }
    edtLine(height);
    for (let y = 0; y < height; y += 1) {
      field[y * width + x] = result[y];
    }
  }
  return field;
}

function centeredGlyphMask(glyph) {
  const scratch = document.createElement("canvas");
  scratch.width = SDF_TILE;
  scratch.height = SDF_TILE;
  const context = scratch.getContext("2d", { willReadFrequently: true });
  context.clearRect(0, 0, SDF_TILE, SDF_TILE);
  context.fillStyle = "white";
  context.font = `${12.27 * SDF_SUPERSAMPLE}px "CMU Typewriter", "Noto Math", "Noto Symbols", sans-serif`;
  context.textAlign = "center";
  context.textBaseline = "middle";
  context.fillText(glyph, SDF_TILE * 0.5, SDF_TILE * 0.5);
  const source = context.getImageData(0, 0, SDF_TILE, SDF_TILE);
  let minX = SDF_TILE;
  let minY = SDF_TILE;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < SDF_TILE; y += 1) {
    for (let x = 0; x < SDF_TILE; x += 1) {
      if (source.data[(y * SDF_TILE + x) * 4 + 3] > 8) {
        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        maxX = Math.max(maxX, x);
        maxY = Math.max(maxY, y);
      }
    }
  }
  if (maxX < minX || maxY < minY) {
    throw new Error(`Could not rasterize the ${glyph} foundry die.`);
  }
  const shiftX = Math.round((SDF_TILE - minX - maxX) * 0.5);
  const shiftY = Math.round((SDF_TILE - minY - maxY) * 0.5);
  const alpha = new Uint8Array(SDF_TILE * SDF_TILE);
  for (let y = 0; y < SDF_TILE; y += 1) {
    for (let x = 0; x < SDF_TILE; x += 1) {
      const sourceX = x - shiftX;
      const sourceY = y - shiftY;
      if (sourceX >= 0 && sourceX < SDF_TILE && sourceY >= 0 && sourceY < SDF_TILE) {
        alpha[y * SDF_TILE + x] = source.data[(sourceY * SDF_TILE + sourceX) * 4 + 3];
      }
    }
  }
  return alpha;
}

function signedDistance(alpha) {
  const ink = new Uint8Array(alpha.length);
  const voids = new Uint8Array(alpha.length);
  for (let index = 0; index < alpha.length; index += 1) {
    ink[index] = alpha[index] >= 128 ? 1 : 0;
    voids[index] = ink[index] ? 0 : 1;
  }
  const toInk = distanceTransform(ink, SDF_TILE, SDF_TILE);
  const toVoid = distanceTransform(voids, SDF_TILE, SDF_TILE);
  const encoded = new Uint8Array(alpha.length);
  for (let index = 0; index < alpha.length; index += 1) {
    const distance = ink[index] ? Math.sqrt(toVoid[index]) : -Math.sqrt(toInk[index]);
    const coverageCorrection = (alpha[index] / 255 - 0.5) * 0.85;
    const logical = (distance + coverageCorrection) / SDF_SUPERSAMPLE;
    const normalized = Math.max(0, Math.min(1, 0.5 + logical / (2 * SDF_RANGE)));
    encoded[index] = Math.round(normalized * 255);
  }
  return encoded;
}

function glyphAtlas(gl) {
  const width = SDF_TILE * GLYPHS.length;
  const pixels = new Uint8Array(width * SDF_TILE);
  for (let glyphIndex = 0; glyphIndex < GLYPHS.length; glyphIndex += 1) {
    const sdf = signedDistance(centeredGlyphMask(GLYPHS[glyphIndex].glyph));
    for (let y = 0; y < SDF_TILE; y += 1) {
      pixels.set(
        sdf.subarray(y * SDF_TILE, (y + 1) * SDF_TILE),
        y * width + glyphIndex * SDF_TILE,
      );
    }
  }
  const texture = gl.createTexture();
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
  gl.texImage2D(gl.TEXTURE_2D, 0, gl.R8, width, SDF_TILE, 0, gl.RED, gl.UNSIGNED_BYTE, pixels);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  return texture;
}

function compileShader(gl, type, source) {
  const shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(shader);
    gl.deleteShader(shader);
    throw new Error(`WebGL shader compilation failed: ${log}`);
  }
  return shader;
}

function createProgram(gl) {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, `#version 300 es
    precision highp float;
    layout(location = 0) in vec2 aPosition;
    uniform vec2 uResolution;
    uniform vec2 uCenter;
    uniform float uScale;
    uniform float uPixelRatio;
    uniform float uGauge;
    out vec2 vLocal;
    void main() {
      vLocal = aPosition * (uGauge * 0.5 + 1.25);
      vec2 screen = uCenter + vLocal * uScale;
      vec2 clip = screen / uResolution * 2.0 - 1.0;
      gl_Position = vec4(clip.x, -clip.y, 0.0, 1.0);
    }
  `);
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, `#version 300 es
    precision highp float;
    const float PI = 3.141592653589793;
    const float TAU = 6.283185307179586;
    const vec3 LIGHT = vec3(0.0, -0.5, 0.8660254);
    const vec3 VIEW = vec3(0.0, 0.0, 1.0);
    const vec3 BRONZE_SHADOW = vec3(34.0, 28.0, 19.0) / 255.0;
    const vec3 BRONZE_BODY = vec3(104.0, 86.0, 58.0) / 255.0;
    const vec3 BRONZE_GLINT = vec3(196.0, 170.0, 124.0) / 255.0;
    const vec3 DANGER = vec3(212.0, 74.0, 40.0) / 255.0;
    const vec3 LOVE = vec3(191.0, 61.0, 105.0) / 255.0;
    in vec2 vLocal;
    uniform sampler2D uGlyphAtlas;
    uniform vec2 uResolution;
    uniform float uScale;
    uniform float uPixelRatio;
    uniform float uGauge;
    uniform float uSootWidth;
    uniform float uSootReflectance;
    uniform float uDepth;
    uniform float uBevel;
    uniform float uRoughness;
    uniform int uGlyph;
    uniform int uFinish;
    uniform int uTopology;
    uniform int uOptics;
    uniform int uEnvironment;
    uniform int uObject;
    out vec4 color;

    float saturate(float value) {
      return clamp(value, 0.0, 1.0);
    }

    vec3 toLinear(vec3 value) {
      return pow(max(value, vec3(0.0)), vec3(2.2));
    }

    vec3 toSrgb(vec3 value) {
      return pow(max(value, vec3(0.0)), vec3(1.0 / 2.2));
    }

    float hash(vec2 point) {
      return fract(sin(dot(point, vec2(127.1, 311.7))) * 43758.5453123);
    }

    float glyphDistance(vec2 point) {
      vec2 mediumPoint = point * (24.0 / uGauge);
      vec2 uv = mediumPoint / 24.0 + 0.5;
      if (any(lessThan(uv, vec2(0.0))) || any(greaterThan(uv, vec2(1.0)))) {
        return -${SDF_RANGE}.0 * (uGauge / 24.0);
      }
      uv.x = (float(uGlyph) + uv.x) / ${GLYPHS.length}.0;
      float encoded = texture(uGlyphAtlas, uv).r;
      return (encoded - 0.5) * ${SDF_RANGE * 2}.0 * (uGauge / 24.0);
    }

    float glyphMask(vec2 point, float antialias) {
      return smoothstep(-antialias, antialias, glyphDistance(point));
    }

    float productionDilation(vec2 point, float antialias) {
      float mask = 0.0;
      float pixel = 1.0 / uPixelRatio;
      for (int y = -1; y <= 1; y += 1) {
        for (int x = -1; x <= 1; x += 1) {
          mask = max(mask, glyphMask(point + vec2(float(x), float(y)) * pixel, antialias));
        }
      }
      return mask;
    }

    vec3 bronzePalette(float tone) {
      tone = saturate(tone);
      vec3 result = tone < 0.6
        ? mix(BRONZE_SHADOW, BRONZE_BODY, tone / 0.6)
        : mix(BRONZE_BODY, BRONZE_GLINT, (tone - 0.6) / 0.4);
      return min(result * 1.2, vec3(1.0));
    }

    vec3 fresnelSchlick(float cosine, vec3 f0) {
      return f0 + (1.0 - f0) * pow(1.0 - saturate(cosine), 5.0);
    }

    float ggxDistribution(float nDotH, float roughness) {
      float alpha = roughness * roughness;
      float alpha2 = alpha * alpha;
      float denominator = nDotH * nDotH * (alpha2 - 1.0) + 1.0;
      return alpha2 / max(PI * denominator * denominator, 0.0001);
    }

    float ggxAnisotropic(vec3 halfVector, vec3 normal, float roughness) {
      vec3 tangent = normalize(vec3(1.0, 0.0, max(0.001, -normal.x / max(normal.z, 0.08))));
      vec3 bitangent = normalize(cross(normal, tangent));
      float alphaX = max(0.025, roughness * roughness * 0.36);
      float alphaY = max(0.05, roughness * roughness * 1.85);
      float hx = dot(halfVector, tangent);
      float hy = dot(halfVector, bitangent);
      float hz = max(dot(halfVector, normal), 0.001);
      float denominator = hx * hx / (alphaX * alphaX)
        + hy * hy / (alphaY * alphaY) + hz * hz;
      return 1.0 / max(PI * alphaX * alphaY * denominator * denominator, 0.0001);
    }

    float smithMask(float nDotDirection, float roughness) {
      float k = pow(roughness + 1.0, 2.0) / 8.0;
      return nDotDirection / max(nDotDirection * (1.0 - k) + k, 0.0001);
    }

    vec3 environmentRadiance(vec3 direction, float roughness) {
      float blur = roughness * 0.72;
      vec3 room = vec3(0.009, 0.007, 0.005);
      float ceiling = exp(-pow(abs((direction.y + 0.52) / (0.18 + blur)), 4.0))
        * exp(-pow(abs(direction.x / (0.82 + blur)), 6.0));
      float rightStrip = exp(-pow(abs((direction.x - 0.55) / (0.055 + blur * 0.42)), 2.0))
        * exp(-pow(abs(direction.y / (0.72 + blur)), 6.0));
      float leftStrip = exp(-pow(abs((direction.x + 0.56) / (0.09 + blur * 0.45)), 2.0))
        * exp(-pow(abs((direction.y + 0.05) / (0.76 + blur)), 6.0));
      float floorBand = smoothstep(0.0 - blur, 0.68 + blur, direction.y);
      float horizon = exp(-pow(abs((direction.y - 0.28) / (0.055 + blur * 0.38)), 2.0));
      if (uEnvironment == 0) {
        return room;
      }
      if (uEnvironment == 1) {
        return room + ceiling * vec3(1.55, 1.08, 0.58)
          + rightStrip * vec3(2.0, 1.44, 0.78)
          + floorBand * vec3(0.055, 0.035, 0.018);
      }
      if (uEnvironment == 2) {
        return room + rightStrip * vec3(2.7, 1.95, 1.12)
          + ceiling * vec3(0.16, 0.18, 0.19)
          + floorBand * vec3(0.025, 0.035, 0.055);
      }
      if (uEnvironment == 3) {
        return room + leftStrip * vec3(1.48, 0.88, 0.38)
          + rightStrip * vec3(0.52, 0.82, 1.16)
          + floorBand * vec3(0.06, 0.035, 0.02);
      }
      if (uEnvironment == 4) {
        return room * 0.45 + horizon * vec3(3.0, 0.74, 0.12)
          + floorBand * vec3(0.19, 0.045, 0.008);
      }
      return room + ceiling * vec3(0.54, 0.82, 1.18)
        + leftStrip * vec3(0.68, 0.94, 1.34)
        + floorBand * vec3(0.16, 0.075, 0.025);
    }

    vec3 filmic(vec3 value) {
      value = max(value, vec3(0.0));
      return clamp((value * (2.51 * value + 0.03)) / (value * (2.43 * value + 0.59) + 0.14), 0.0, 1.0);
    }

    vec3 quantizeLuminance(vec3 srgb, float levels) {
      float luminance = dot(srgb, vec3(0.2126, 0.7152, 0.0722));
      float quantized = floor(luminance * (levels - 1.0) + 0.5) / (levels - 1.0);
      return clamp(srgb * quantized / max(luminance, 0.035), 0.0, 1.0);
    }

    vec3 legacyMaterial(vec3 normal, int role) {
      float diffuse = max(dot(normal, LIGHT), 0.0);
      vec3 halfway = normalize(LIGHT + VIEW);
      float reflection = max(dot(normal, halfway), 0.0);
      if (role == 4) {
        return toSrgb(vec3(uSootReflectance));
      }
      if (role == 2 || role == 3) {
        vec3 paint = role == 2 ? DANGER : LOVE;
        return paint * (0.84 + 0.16 * diffuse);
      }
      float tone = role == 1
        ? 0.24 + 0.68 * diffuse + pow(reflection, 14.0)
        : min(0.72, 0.13 + 0.32 * diffuse + 0.01 * pow(reflection, 12.0) + 3.0 * pow(reflection, 128.0));
      return bronzePalette(tone);
    }

    vec3 physicalMaterial(vec3 normal, vec3 position, int role) {
      float nDotL = max(dot(normal, LIGHT), 0.0);
      float nDotV = max(dot(normal, VIEW), 0.001);
      vec3 halfway = normalize(LIGHT + VIEW);
      float nDotH = max(dot(normal, halfway), 0.001);
      float vDotH = max(dot(VIEW, halfway), 0.001);
      if (role == 4) {
        vec3 soot = vec3(uSootReflectance) * (0.18 + 0.62 * nDotL);
        return toSrgb(soot);
      }
      if (role == 2 || role == 3) {
        vec3 albedo = toLinear(role == 2 ? DANGER : LOVE);
        float dielectric = ggxDistribution(nDotH, 0.62) * smithMask(nDotL, 0.62)
          * smithMask(nDotV, 0.62) * 0.04 / max(4.0 * nDotL * nDotV, 0.001);
        vec3 paint = albedo * (0.055 + 0.72 * nDotL) + vec3(dielectric * nDotL * 0.22);
        paint += environmentRadiance(reflect(-VIEW, normal), 0.62) * 0.025;
        return toSrgb(filmic(paint));
      }

      vec3 f0 = role == 1 ? vec3(0.88, 0.56, 0.22) : vec3(0.73, 0.42, 0.16);
      float roughness = role == 1 ? max(0.11, uRoughness * 0.64) : uRoughness;
      if (uOptics == 6 || uOptics == 7 || uOptics == 8) {
        roughness = sqrt(roughness * roughness + pow(0.22 / max(uScale, 0.75), 2.0));
      }
      if (uOptics == 9) {
        roughness = 0.075;
      }
      if (uOptics == 1) {
        vec3 body = toLinear(BRONZE_BODY);
        vec3 linear = body * (0.045 + 0.56 * nDotL)
          + f0 * pow(nDotH, 30.0) * 1.15 * nDotL;
        return toSrgb(filmic(linear));
      }

      float distribution = uOptics == 5
        ? ggxAnisotropic(halfway, normal, roughness)
        : ggxDistribution(nDotH, roughness);
      float geometry = smithMask(nDotL, roughness) * smithMask(nDotV, roughness);
      vec3 fresnel = fresnelSchlick(vDotH, f0);
      vec3 direct = distribution * geometry * fresnel
        / max(4.0 * nDotL * nDotV, 0.001) * nDotL * 1.15;
      vec3 radiance = direct;
      if (uOptics != 2) {
        vec3 reflected = reflect(-VIEW, normal);
        if (uOptics == 5) {
          reflected.x *= 0.58;
          reflected = normalize(reflected);
        }
        vec3 room = environmentRadiance(reflected, roughness);
        vec3 grazing = fresnelSchlick(nDotV, f0);
        radiance += room * grazing * (0.76 - 0.23 * roughness);
      }
      if (uOptics == 4) {
        float coatRoughness = 0.48;
        float coatD = ggxDistribution(nDotH, coatRoughness);
        float coatG = smithMask(nDotL, coatRoughness) * smithMask(nDotV, coatRoughness);
        float coatF = 0.04 + 0.96 * pow(1.0 - vDotH, 5.0);
        float coat = coatD * coatG * coatF / max(4.0 * nDotL * nDotV, 0.001);
        vec3 oxide = vec3(0.045, 0.027, 0.011) * (0.16 + 0.44 * nDotL);
        radiance = radiance * (1.0 - coatF) * 0.78 + oxide + vec3(coat * nDotL * 0.34);
      }
      if (uOptics == 9) {
        radiance *= 1.28;
      }
      vec3 srgb = toSrgb(filmic(radiance));
      if (uOptics == 7) {
        srgb = quantizeLuminance(srgb, 4.0);
      } else if (uOptics == 8) {
        srgb = quantizeLuminance(srgb, 6.0);
      }
      return srgb;
    }

    vec3 material(vec3 normal, vec3 position, int role) {
      return uOptics == 0
        ? legacyMaterial(normal, role)
        : physicalMaterial(normal, position, role);
    }

    vec3 objectSpecimen(vec2 point) {
      vec3 normal = vec3(0.0, 0.0, 1.0);
      float z = 0.0;
      if (uObject == 1) {
        float radius = 10.0;
        float squared = dot(point, point);
        if (squared > radius * radius) discard;
        z = sqrt(max(radius * radius - squared, 0.0));
        normal = normalize(vec3(point, z));
      } else if (uObject == 2) {
        float radius = 7.0;
        if (abs(point.y) > 10.0 || abs(point.x) > radius) discard;
        z = sqrt(max(radius * radius - point.x * point.x, 0.0));
        normal = normalize(vec3(point.x, 0.0, z));
      } else if (uObject == 3) {
        vec2 absolute = abs(point);
        if (max(absolute.x, absolute.y) > 10.0) discard;
        vec2 over = max(absolute - 7.0, 0.0);
        normal = normalize(vec3(sign(point.x) * over.x, sign(point.y) * over.y, 1.2));
        z = 2.0 - length(over) * 0.7;
      } else if (uObject == 4) {
        float radius = length(point);
        if (radius > 10.0) discard;
        float well = saturate(1.0 - radius / 8.2);
        z = -2.4 * well * well;
        normal = normalize(vec3(-4.8 * well * point / (8.2 * max(radius, 0.4)), 1.0));
      } else {
        float radius = 5.6;
        if (abs(point.y) > 10.0 || abs(point.x) > radius) discard;
        float barrel = sqrt(max(radius * radius - point.x * point.x, 0.08));
        float phase = TAU * point.y / 5.4 - asin(clamp(point.x / radius, -1.0, 1.0));
        z = barrel + 0.42 * cos(phase);
        float dzdx = -point.x / barrel + 0.42 * sin(phase) / barrel;
        float dzdy = -0.42 * sin(phase) * TAU / 5.4;
        normal = normalize(vec3(-dzdx, -dzdy, 1.0));
      }
      return material(normal, vec3(point, z), 0);
    }

    void main() {
      if (uObject > 0) {
        color = vec4(objectSpecimen(vLocal), 1.0);
        return;
      }

      float socketHalf = uGauge * 0.5;
      float rimInner = socketHalf - 1.0;
      float bodyHalf = socketHalf * (49.0 / 66.0);
      float topHalf = socketHalf * (89.0 / 132.0);
      vec2 absolute = abs(vLocal);
      float boxRadius = max(absolute.x, absolute.y);
      if (boxRadius > socketHalf) discard;

      if (boxRadius >= rimInner) {
        vec2 edge = max(absolute - rimInner, 0.0);
        vec3 rimNormal = normalize(vec3(sign(vLocal.x) * edge.x, sign(vLocal.y) * edge.y, 0.42));
        color = vec4(material(rimNormal, vec3(vLocal, 0.42), 0), 1.0);
        return;
      }
      if (boxRadius > bodyHalf) {
        float catchLight = smoothstep(bodyHalf, rimInner, boxRadius);
        color = vec4(mix(vec3(0.006, 0.005, 0.004), vec3(0.045, 0.035, 0.024), catchLight), 1.0);
        return;
      }

      vec3 crownNormal = vec3(0.0, 0.0, 1.0);
      if (boxRadius > topHalf) {
        vec2 edge = max(absolute - topHalf, 0.0);
        crownNormal = normalize(vec3(sign(vLocal.x) * edge.x, sign(vLocal.y) * edge.y, 0.95));
        color = vec4(material(crownNormal, vec3(vLocal, 2.7), 0), 1.0);
        return;
      }

      vec3 base = material(crownNormal, vec3(vLocal, 3.25), 0);
      float antialias = max(0.08, 0.68 / max(uScale * uPixelRatio, 0.65));
      float sootWidth = uSootWidth / uPixelRatio;
      if (uTopology == 0 || uTopology == 1) {
        vec2 wallShift = uFinish == 0 ? vec2(0.0, 0.42) : vec2(0.0);
        float inside = glyphMask(vLocal - wallShift, antialias);
        float dilation = uTopology == 0
          ? productionDilation(vLocal - wallShift, antialias)
          : smoothstep(-sootWidth - antialias, -sootWidth + antialias, glyphDistance(vLocal - wallShift));
        if (uFinish != 0) {
          float rearWall = glyphMask(vLocal - vec2(0.0, 0.42), antialias);
          base = mix(base, material(normalize(vec3(0.0, -0.96, 0.42)), vec3(vLocal, 2.29), 1), rearWall);
        }
        base = mix(base, material(crownNormal, vec3(vLocal, 3.25), 4), dilation);
        int faceRole = uFinish == 0 ? 1 : uFinish + 1;
        base = mix(base, material(crownNormal, vec3(vLocal, 3.25), faceRole), inside);
        color = vec4(base, 1.0);
        return;
      }

      float distance = glyphDistance(vLocal);
      if (distance <= -antialias) {
        color = vec4(base, 1.0);
        return;
      }
      float sampleStep = 0.16;
      vec2 gradient = vec2(
        glyphDistance(vLocal + vec2(sampleStep, 0.0)) - glyphDistance(vLocal - vec2(sampleStep, 0.0)),
        glyphDistance(vLocal + vec2(0.0, sampleStep)) - glyphDistance(vLocal - vec2(0.0, sampleStep))
      ) / (2.0 * sampleStep);
      float progress = saturate(distance / max(uBevel, 0.05));
      float profile = progress;
      float derivative = 1.0 / max(uBevel, 0.05);
      if (uTopology == 4 || uTopology == 8 || (uTopology == 5 && uFinish != 0)) {
        profile = progress * progress * (3.0 - 2.0 * progress);
        derivative = 6.0 * progress * (1.0 - progress) / max(uBevel, 0.05);
      }
      if (distance >= uBevel) {
        derivative = 0.0;
      }
      float signDepth = uTopology == 7 ? 1.0 : -1.0;
      float dzds = signDepth * uDepth * derivative;
      vec3 cutNormal = normalize(vec3(-dzds * gradient.x, -dzds * gradient.y, 1.0));
      float floorMix = smoothstep(uBevel - antialias, uBevel + antialias, distance);
      int floorRole = uFinish == 0 ? 1 : uFinish + 1;
      vec3 wall = material(cutNormal, vec3(vLocal, 3.25 + signDepth * uDepth * profile), 1);
      vec3 floorColor = material(vec3(0.0, 0.0, 1.0), vec3(vLocal, 3.25 + signDepth * uDepth), floorRole);
      vec3 struck = mix(wall, floorColor, floorMix);
      float toe = exp(-abs(distance - uBevel) / max(sootWidth, 0.06 / uPixelRatio));
      float soot = 0.0;
      if (uTopology == 2) {
        soot = toe;
      } else if (uTopology == 3) {
        soot = toe * smoothstep(-0.18, 0.78, normalize(gradient).y);
      } else if (uTopology == 4) {
        soot = toe * 0.56 + floorMix * 0.13;
      } else if (uTopology == 5) {
        soot = uFinish == 0 ? toe * 0.72 + floorMix * 0.24 : toe * 0.78;
      } else if (uTopology == 6) {
        soot = toe * 0.28;
      } else if (uTopology == 7) {
        soot = exp(-abs(distance) / max(sootWidth, 0.06 / uPixelRatio)) * 0.72;
      } else {
        float broken = smoothstep(0.20, 0.82, hash(floor(vLocal * 1.7)));
        soot = toe * (0.22 + 0.68 * broken) + floorMix * 0.08;
      }
      struck = mix(struck, material(cutNormal, vec3(vLocal, 3.25 - uDepth), 4), saturate(soot));
      float cutCoverage = smoothstep(-antialias, antialias, distance);
      color = vec4(mix(base, struck, cutCoverage), 1.0);
    }
  `);
  const program = gl.createProgram();
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  gl.deleteShader(vertex);
  gl.deleteShader(fragment);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    throw new Error(`WebGL program link failed: ${gl.getProgramInfoLog(program)}`);
  }
  return program;
}

class AtelierRenderer {
  constructor(target) {
    const gl = target.getContext("webgl2", {
      alpha: true,
      antialias: true,
      depth: false,
      premultipliedAlpha: true,
    });
    if (!gl) {
      throw new Error("The Foundry Optics Atelier requires WebGL2.");
    }
    this.canvas = target;
    this.gl = gl;
    this.program = createProgram(gl);
    this.atlas = glyphAtlas(gl);
    this.uniforms = new Map();
    for (const name of [
      "uGlyphAtlas", "uResolution", "uCenter", "uScale", "uPixelRatio", "uGauge", "uSootWidth",
      "uSootReflectance", "uDepth", "uBevel", "uRoughness", "uGlyph", "uFinish",
      "uTopology", "uOptics", "uEnvironment", "uObject",
    ]) {
      this.uniforms.set(name, gl.getUniformLocation(this.program, name));
    }
    const vao = gl.createVertexArray();
    const buffer = gl.createBuffer();
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW,
    );
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
    this.vao = vao;
    gl.useProgram(this.program);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.atlas);
    gl.uniform1i(this.uniforms.get("uGlyphAtlas"), 0);
    gl.uniform1f(this.uniforms.get("uPixelRatio"), 1);
    gl.bindVertexArray(this.vao);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
  }

  scalar(name, value) {
    this.gl.uniform1f(this.uniforms.get(name), value);
  }

  integer(name, value) {
    this.gl.uniform1i(this.uniforms.get(name), value);
  }

  candidate(spec) {
    this.scalar("uSootWidth", spec.sootWidth);
    this.scalar("uSootReflectance", spec.sootReflectance);
    this.scalar("uDepth", spec.depth);
    this.scalar("uBevel", spec.bevel);
    this.scalar("uRoughness", spec.roughness);
    this.integer("uTopology", spec.topology);
    this.integer("uOptics", spec.optics);
    this.integer("uEnvironment", spec.environment);
  }

  draw(spec, center, scale, gauge, glyphIndex, object = 0) {
    const glyph = GLYPHS[glyphIndex];
    this.candidate(spec);
    this.gl.uniform2fv(this.uniforms.get("uCenter"), center);
    this.scalar("uScale", scale);
    this.scalar("uGauge", gauge);
    this.integer("uGlyph", glyphIndex);
    this.integer("uFinish", glyph.finish);
    this.integer("uObject", object);
    this.gl.drawArrays(this.gl.TRIANGLES, 0, 6);
  }

  drawStudy(spec, width, height) {
    const inspectionScale = Math.min(5, Math.max(2.15, (width - 44) / 94));
    const coupon = spec.family === "OPTICS" || spec.family === "ENVIRONMENT";
    const heroY = height * (coupon ? 0.26 : 0.37);
    for (let index = 0; index < HERO_GLYPHS.length; index += 1) {
      this.draw(
        spec,
        [width * (index + 0.5) / HERO_GLYPHS.length, heroY],
        inspectionScale,
        MEDIUM_GAUGE,
        HERO_GLYPHS[index],
      );
    }
    if (coupon) {
      const couponY = height * 0.66;
      const couponSpan = Math.min(47, (width - 28) / 5);
      const couponStart = width * 0.5 - couponSpan * 2;
      for (let object = 1; object <= 5; object += 1) {
        this.draw(spec, [couponStart + (object - 1) * couponSpan, couponY], 1.35, 24, 0, object);
      }
    }
    const armoryY = height * (coupon ? 0.91 : 0.86);
    const spacing = Math.min(39, (width - 24) / GLYPHS.length);
    const start = width * 0.5 - spacing * (GLYPHS.length - 1) * 0.5;
    for (let glyph = 0; glyph < GLYPHS.length; glyph += 1) {
      this.draw(spec, [start + glyph * spacing, armoryY], 1, MEDIUM_GAUGE, glyph);
    }
  }

  drawVerdict(spec, width, height) {
    const gauges = [20, 24, 32];
    const spacing = Math.min(45, (width - 34) / GLYPHS.length);
    const start = width * 0.5 - spacing * (GLYPHS.length - 1) * 0.5;
    for (let row = 0; row < gauges.length; row += 1) {
      const y = (row + 0.5) * height / gauges.length;
      for (let glyph = 0; glyph < GLYPHS.length; glyph += 1) {
        this.draw(spec, [start + glyph * spacing, y], 1, gauges[row], glyph);
      }
    }
  }

  render(targets) {
    const gl = this.gl;
    const documentWidth = Math.ceil(document.documentElement.scrollWidth);
    const documentHeight = Math.ceil(document.documentElement.scrollHeight);
    const maxSize = gl.getParameter(gl.MAX_RENDERBUFFER_SIZE);
    const density = Math.max(
      0.25,
      Math.min(window.devicePixelRatio || 1, 2, maxSize / documentWidth, maxSize / documentHeight),
    );
    const pixelWidth = Math.max(1, Math.floor(documentWidth * density));
    const pixelHeight = Math.max(1, Math.floor(documentHeight * density));
    if (this.canvas.width !== pixelWidth || this.canvas.height !== pixelHeight) {
      this.canvas.width = pixelWidth;
      this.canvas.height = pixelHeight;
      this.canvas.style.width = `${documentWidth}px`;
      this.canvas.style.height = `${documentHeight}px`;
    }
    gl.useProgram(this.program);
    gl.bindVertexArray(this.vao);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.atlas);
    this.scalar("uPixelRatio", density);
    gl.disable(gl.SCISSOR_TEST);
    gl.viewport(0, 0, pixelWidth, pixelHeight);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.enable(gl.SCISSOR_TEST);

    for (const target of targets) {
      const bounds = target.getBoundingClientRect();
      const left = bounds.left + window.scrollX;
      const top = bounds.top + window.scrollY;
      const width = bounds.width;
      const height = bounds.height;
      if (width <= 1 || height <= 1) {
        continue;
      }
      const viewportX = Math.floor(left * density);
      const viewportY = Math.floor((documentHeight - top - height) * density);
      const viewportWidth = Math.max(1, Math.ceil(width * density));
      const viewportHeight = Math.max(1, Math.ceil(height * density));
      gl.viewport(viewportX, viewportY, viewportWidth, viewportHeight);
      gl.scissor(viewportX, viewportY, viewportWidth, viewportHeight);
      gl.clear(gl.COLOR_BUFFER_BIT);
      gl.uniform2f(this.uniforms.get("uResolution"), width, height);
      const spec = target.dataset.fixed === "production"
        ? PRODUCTION
        : target === selectedRender || target === verdictRender
          ? selected
          : CANDIDATE_BY_ID.get(target.dataset.candidate);
      if (!spec) {
        continue;
      }
      if (target.dataset.layout === "verdict") {
        this.drawVerdict(spec, width, height);
      } else {
        this.drawStudy(spec, width, height);
      }
    }
    gl.disable(gl.SCISSOR_TEST);
  }
}

async function start() {
  buildStudies();
  await document.fonts.ready;
  renderer = new AtelierRenderer(canvas);
  restoreSelection();
  new ResizeObserver(requestRender).observe(document.body);
  window.addEventListener("resize", requestRender, { passive: true });
  root.dataset.forge = "ready";
  requestRender();
}

start().catch(fail);
