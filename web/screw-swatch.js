"use strict";

const TAU = Math.PI * 2;
const PITCHES = [4.8, 5.6, 6.4, 7.2];
const DEPTHS = [0.35, 0.55, 0.75];
const MINOR_DIAMETERS = [2.6, 3.1, 3.6];
const CURRENT = Object.freeze({
  kind: "cord",
  pitch: 6.23,
  minor: 3.1,
  cordDiameter: 0.96,
});
const INITIAL = Object.freeze({
  kind: "integral",
  pitch: 6.4,
  depth: 0.55,
  minor: 3.1,
});
const SHAFT_LENGTH = 28;
const CAP_WIDTH = 11.6;
const CAP_HEIGHT = 4.6;
const CAP_DEPTH = 6.2;
const NUT_WIDTH = 7.2;
const NUT_HEIGHT = 7.0;
const NUT_DEPTH = 6.8;

const root = document.documentElement;
const canvas = document.getElementById("forge");
const tables = document.getElementById("tables");
const fault = document.getElementById("fault");
const selection = document.getElementById("selection");
const selectedSpec = document.getElementById("selected-spec");
const selectedRender = document.getElementById("selected-render");

let selected = { ...INITIAL };
let nutPose = "bare";
let renderer;
let renderQueued = false;

const fixed = value => Number(value).toFixed(2);
const majorDiameter = spec => spec.minor + 2 * spec.depth;
const integralLabel = spec =>
  `P ${fixed(spec.pitch)} · h ${fixed(spec.depth)} · d₁ ${fixed(spec.minor)} · d ${fixed(majorDiameter(spec))}`;

function buildTables() {
  const fragment = document.createDocumentFragment();
  for (const minor of MINOR_DIAMETERS) {
    const section = document.createElement("section");
    section.className = "diameter-table";
    section.innerHTML = `
      <div class="table-heading">
        <h2>MINOR DIAMETER d₁ = ${fixed(minor)} px</h2>
        <span class="axis-label">columns vary P · rows vary h</span>
      </div>
      <div class="swatch-grid"></div>
    `;
    const grid = section.querySelector(".swatch-grid");
    for (const depth of DEPTHS) {
      for (const pitch of PITCHES) {
        const spec = {
          kind: "integral",
          pitch,
          depth,
          minor,
        };
        const swatch = document.createElement("button");
        swatch.type = "button";
        swatch.className = "swatch";
        swatch.dataset.pitch = String(pitch);
        swatch.dataset.depth = String(depth);
        swatch.dataset.minor = String(minor);
        swatch.setAttribute("aria-label", `Select ${integralLabel(spec)}`);
        swatch.innerHTML = `
          <div
            class="render-target"
            data-kind="integral"
            data-pitch="${pitch}"
            data-depth="${depth}"
            data-minor="${minor}"
          ></div>
          <span class="scale-key">5× INSPECTION · 1× GAUGE</span>
          <span class="swatch-spec">
            <span>P ${fixed(pitch)}</span><span>h ${fixed(depth)}</span>
            <span>d₁ ${fixed(minor)}</span><span>d ${fixed(majorDiameter(spec))}</span>
          </span>
        `;
        swatch.addEventListener("click", () => selectSpec(spec));
        grid.append(swatch);
      }
    }
    fragment.append(section);
  }
  tables.append(fragment);
}

function selectSpec(spec) {
  selected = { ...spec };
  selectedRender.dataset.pitch = String(spec.pitch);
  selectedRender.dataset.depth = String(spec.depth);
  selectedRender.dataset.minor = String(spec.minor);
  selectedSpec.textContent = integralLabel(spec);
  selection.textContent = `SELECTED · ${integralLabel(spec)}`;
  for (const swatch of document.querySelectorAll(".swatch")) {
    swatch.classList.toggle(
      "selected",
      Number(swatch.dataset.pitch) === spec.pitch &&
        Number(swatch.dataset.depth) === spec.depth &&
        Number(swatch.dataset.minor) === spec.minor,
    );
  }
  const hash = `p${fixed(spec.pitch)}-h${fixed(spec.depth)}-d1${fixed(spec.minor)}`;
  history.replaceState(null, "", `#${hash}`);
  requestRender();
}

function selectPose(pose) {
  nutPose = pose;
  for (const button of document.querySelectorAll(".pose")) {
    button.setAttribute("aria-pressed", String(button.dataset.pose === pose));
  }
  const url = new URL(location.href);
  if (pose === "bare") {
    url.searchParams.delete("pose");
  } else {
    url.searchParams.set("pose", pose);
  }
  history.replaceState(null, "", url);
  requestRender();
}

function restorePose() {
  const pose = new URLSearchParams(location.search).get("pose");
  selectPose(["top", "middle", "bottom"].includes(pose) ? pose : "bare");
}

function restoreHashSelection() {
  const match = /^#p([\d.]+)-h([\d.]+)-d1([\d.]+)$/.exec(location.hash);
  if (!match) {
    selectSpec(INITIAL);
    return;
  }
  const candidate = {
    kind: "integral",
    pitch: Number(match[1]),
    depth: Number(match[2]),
    minor: Number(match[3]),
  };
  const admitted =
    PITCHES.includes(candidate.pitch) &&
    DEPTHS.includes(candidate.depth) &&
    MINOR_DIAMETERS.includes(candidate.minor);
  selectSpec(admitted ? candidate : INITIAL);
}

function requestRender() {
  if (renderQueued || !renderer) {
    return;
  }
  renderQueued = true;
  requestAnimationFrame(() => {
    renderQueued = false;
    renderer.render(document.querySelectorAll(".render-target"), nutPose);
  });
}

function fail(error) {
  root.dataset.forge = "failed";
  fault.textContent = error instanceof Error ? error.message : String(error);
}

function normalize([x, y, z]) {
  const length = Math.hypot(x, y, z) || 1;
  return [x / length, y / length, z / length];
}

function cross([ax, ay, az], [bx, by, bz]) {
  return [ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx];
}

function pushVertex(vertices, position, normal) {
  vertices.push(...position, ...normal);
}

function makeIntegralThread(spec) {
  const vertices = [];
  const indices = [];
  const radialSegments = 48;
  const axialSegments = Math.ceil((SHAFT_LENGTH / spec.pitch) * 40);
  const rootRadius = spec.minor * 0.5;
  const phaseOffset = 0.18;

  for (let axial = 0; axial <= axialSegments; axial += 1) {
    const y = -SHAFT_LENGTH * 0.5 + SHAFT_LENGTH * axial / axialSegments;
    for (let radial = 0; radial <= radialSegments; radial += 1) {
      const theta = TAU * radial / radialSegments;
      const turn = y / spec.pitch - theta / TAU + phaseOffset;
      const phase = turn - Math.floor(turn);
      const slope = phase < 0.5 ? 2 : -2;
      const ridge = 1 - 2 * Math.abs(phase - 0.5);
      const radius = rootRadius + spec.depth * ridge;
      const radialY = spec.depth * slope / spec.pitch;
      const radialTheta = -spec.depth * slope / TAU;
      const cos = Math.cos(theta);
      const sin = Math.sin(theta);
      const alongY = [radialY * cos, 1, radialY * sin];
      const around = [
        radialTheta * cos - radius * sin,
        0,
        radialTheta * sin + radius * cos,
      ];
      pushVertex(
        vertices,
        [radius * cos, y, radius * sin],
        normalize(cross(alongY, around)),
      );
    }
  }
  const stride = radialSegments + 1;
  for (let axial = 0; axial < axialSegments; axial += 1) {
    for (let radial = 0; radial < radialSegments; radial += 1) {
      const a = axial * stride + radial;
      const b = a + 1;
      const c = a + stride;
      const d = c + 1;
      indices.push(a, c, b, b, c, d);
    }
  }
  return { vertices, indices };
}

function appendCylinder(target, radius, length, radialSegments = 48) {
  const base = target.vertices.length / 6;
  for (const y of [-length * 0.5, length * 0.5]) {
    for (let radial = 0; radial <= radialSegments; radial += 1) {
      const theta = TAU * radial / radialSegments;
      const cos = Math.cos(theta);
      const sin = Math.sin(theta);
      pushVertex(target.vertices, [radius * cos, y, radius * sin], [cos, 0, sin]);
    }
  }
  const stride = radialSegments + 1;
  for (let radial = 0; radial < radialSegments; radial += 1) {
    const a = base + radial;
    const b = a + 1;
    const c = a + stride;
    const d = c + 1;
    target.indices.push(a, c, b, b, c, d);
  }
}

function appendHelicalCord(target, spec) {
  const turns = SHAFT_LENGTH / spec.pitch;
  const axialSegments = Math.ceil(turns * 36);
  const tubeSegments = 14;
  const tubeRadius = spec.cordDiameter * 0.5;
  const centerRadius = spec.minor * 0.5 + 0.35 * tubeRadius;
  const base = target.vertices.length / 6;

  for (let axial = 0; axial <= axialSegments; axial += 1) {
    const y = -SHAFT_LENGTH * 0.5 + SHAFT_LENGTH * axial / axialSegments;
    const theta = TAU * y / spec.pitch + 0.55;
    const cos = Math.cos(theta);
    const sin = Math.sin(theta);
    const turnRate = TAU / spec.pitch;
    const tangent = normalize([
      -centerRadius * sin * turnRate,
      1,
      centerRadius * cos * turnRate,
    ]);
    const radial = [cos, 0, sin];
    const binormal = normalize(cross(tangent, radial));
    const center = [centerRadius * cos, y, centerRadius * sin];
    for (let tube = 0; tube <= tubeSegments; tube += 1) {
      const phi = TAU * tube / tubeSegments;
      const ring = [
        radial[0] * Math.cos(phi) + binormal[0] * Math.sin(phi),
        radial[1] * Math.cos(phi) + binormal[1] * Math.sin(phi),
        radial[2] * Math.cos(phi) + binormal[2] * Math.sin(phi),
      ];
      pushVertex(
        target.vertices,
        [
          center[0] + tubeRadius * ring[0],
          center[1] + tubeRadius * ring[1],
          center[2] + tubeRadius * ring[2],
        ],
        normalize(ring),
      );
    }
  }
  const stride = tubeSegments + 1;
  for (let axial = 0; axial < axialSegments; axial += 1) {
    for (let tube = 0; tube < tubeSegments; tube += 1) {
      const a = base + axial * stride + tube;
      const b = a + 1;
      const c = a + stride;
      const d = c + 1;
      target.indices.push(a, c, b, b, c, d);
    }
  }
}

function makeCordThread(spec) {
  const target = { vertices: [], indices: [] };
  appendCylinder(target, spec.minor * 0.5, SHAFT_LENGTH);
  appendHelicalCord(target, spec);
  return target;
}

function makeBox(width, height, depth) {
  const vertices = [];
  const indices = [];
  const x = width * 0.5;
  const y = height * 0.5;
  const z = depth * 0.5;
  const faces = [
    { normal: [0, 0, 1], points: [[-x, -y, z], [x, -y, z], [x, y, z], [-x, y, z]] },
    { normal: [0, 0, -1], points: [[x, -y, -z], [-x, -y, -z], [-x, y, -z], [x, y, -z]] },
    { normal: [1, 0, 0], points: [[x, -y, z], [x, -y, -z], [x, y, -z], [x, y, z]] },
    { normal: [-1, 0, 0], points: [[-x, -y, -z], [-x, -y, z], [-x, y, z], [-x, y, -z]] },
    { normal: [0, 1, 0], points: [[-x, y, z], [x, y, z], [x, y, -z], [-x, y, -z]] },
    { normal: [0, -1, 0], points: [[-x, -y, -z], [x, -y, -z], [x, -y, z], [-x, -y, z]] },
  ];
  for (const face of faces) {
    const base = vertices.length / 6;
    for (const point of face.points) {
      pushVertex(vertices, point, face.normal);
    }
    indices.push(base, base + 1, base + 2, base, base + 2, base + 3);
  }
  return { vertices, indices };
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
    layout(location = 0) in vec3 aPosition;
    layout(location = 1) in vec3 aNormal;
    uniform vec2 uViewport;
    uniform vec2 uCenter;
    uniform float uScale;
    uniform float uTranslateY;
    out vec3 vNormal;
    void main() {
      vec3 position = aPosition + vec3(0.0, uTranslateY, 0.0);
      vec2 screen = uCenter + position.xy * uScale;
      vec2 clip = screen / uViewport * 2.0 - 1.0;
      gl_Position = vec4(clip, -position.z * 0.025, 1.0);
      vNormal = aNormal;
    }
  `);
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, `#version 300 es
    precision highp float;
    in vec3 vNormal;
    out vec4 color;
    void main() {
      vec3 normal = normalize(vNormal);
      vec3 light = normalize(vec3(0.0, -0.5, 0.8660254));
      vec3 view = vec3(0.0, 0.0, 1.0);
      float diffuse = max(dot(normal, light), 0.0);
      float broad = pow(max(dot(normal, normalize(light + view)), 0.0), 7.0);
      float glint = pow(max(dot(normal, normalize(light + view)), 0.0), 38.0);
      vec3 bronze = vec3(0.29, 0.205, 0.118);
      vec3 lamplight = vec3(0.95, 0.70, 0.39);
      vec3 shaded = bronze * (0.23 + 0.69 * diffuse) + lamplight * (0.14 * broad + 0.46 * glint);
      color = vec4(shaded, 1.0);
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

class ForgeRenderer {
  constructor(target) {
    const gl = target.getContext("webgl2", {
      alpha: true,
      antialias: true,
      depth: true,
      premultipliedAlpha: true,
    });
    if (!gl) {
      throw new Error("This study requires WebGL2.");
    }
    this.canvas = target;
    this.gl = gl;
    this.program = createProgram(gl);
    this.viewportLocation = gl.getUniformLocation(this.program, "uViewport");
    this.centerLocation = gl.getUniformLocation(this.program, "uCenter");
    this.scaleLocation = gl.getUniformLocation(this.program, "uScale");
    this.translateLocation = gl.getUniformLocation(this.program, "uTranslateY");
    this.meshes = new Map();
    this.cap = this.upload(makeBox(CAP_WIDTH, CAP_HEIGHT, CAP_DEPTH));
    this.nut = this.upload(makeBox(NUT_WIDTH, NUT_HEIGHT, NUT_DEPTH));
    gl.useProgram(this.program);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.disable(gl.CULL_FACE);
  }

  upload(model) {
    const gl = this.gl;
    const vao = gl.createVertexArray();
    const vertexBuffer = gl.createBuffer();
    const indexBuffer = gl.createBuffer();
    gl.bindVertexArray(vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, vertexBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(model.vertices), gl.STATIC_DRAW);
    gl.enableVertexAttribArray(0);
    gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 24, 0);
    gl.enableVertexAttribArray(1);
    gl.vertexAttribPointer(1, 3, gl.FLOAT, false, 24, 12);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint32Array(model.indices), gl.STATIC_DRAW);
    gl.bindVertexArray(null);
    return { vao, count: model.indices.length };
  }

  thread(spec) {
    const key = spec.kind === "cord"
      ? `cord:${spec.pitch}:${spec.minor}:${spec.cordDiameter}`
      : `integral:${spec.pitch}:${spec.depth}:${spec.minor}`;
    if (!this.meshes.has(key)) {
      const model = spec.kind === "cord" ? makeCordThread(spec) : makeIntegralThread(spec);
      this.meshes.set(key, this.upload(model));
    }
    return this.meshes.get(key);
  }

  draw(mesh, center, scale, translateY = 0) {
    const gl = this.gl;
    gl.uniform2fv(this.centerLocation, center);
    gl.uniform1f(this.scaleLocation, scale);
    gl.uniform1f(this.translateLocation, translateY);
    gl.bindVertexArray(mesh.vao);
    gl.drawElements(gl.TRIANGLES, mesh.count, gl.UNSIGNED_INT, 0);
  }

  drawAssembly(spec, width, height, pose, scale, x) {
    const center = [x, height * 0.5];
    const thread = this.thread(spec);
    this.draw(thread, center, scale);
    const capOffset = SHAFT_LENGTH * 0.5 + CAP_HEIGHT * 0.5;
    this.draw(this.cap, center, scale, capOffset);
    this.draw(this.cap, center, scale, -capOffset);
    if (pose !== "bare") {
      const nutOffset = {
        top: SHAFT_LENGTH * 0.5 - NUT_HEIGHT * 0.5,
        middle: 0,
        bottom: -SHAFT_LENGTH * 0.5 + NUT_HEIGHT * 0.5,
      }[pose];
      this.draw(this.nut, center, scale, nutOffset);
    }
  }

  render(targets, pose) {
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
    gl.disable(gl.SCISSOR_TEST);
    gl.viewport(0, 0, pixelWidth, pixelHeight);
    gl.clearColor(0, 0, 0, 0);
    gl.clearDepth(1);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
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
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
      gl.uniform2f(this.viewportLocation, width, height);
      const spec = target.dataset.kind === "cord"
        ? CURRENT
        : {
            kind: "integral",
            pitch: Number(target.dataset.pitch),
            depth: Number(target.dataset.depth),
            minor: Number(target.dataset.minor),
          };
      this.drawAssembly(spec, width, height, pose, 5, width * 0.38);
      this.drawAssembly(spec, width, height, pose, 1, width * 0.82);
    }
    gl.disable(gl.SCISSOR_TEST);
  }
}

try {
  buildTables();
  for (const button of document.querySelectorAll(".pose")) {
    button.addEventListener("click", () => selectPose(button.dataset.pose));
  }
  renderer = new ForgeRenderer(canvas);
  restoreHashSelection();
  restorePose();
  new ResizeObserver(requestRender).observe(document.body);
  window.addEventListener("resize", requestRender, { passive: true });
  document.fonts?.ready.then(requestRender);
  root.dataset.forge = "ready";
  requestRender();
} catch (error) {
  fail(error);
}
