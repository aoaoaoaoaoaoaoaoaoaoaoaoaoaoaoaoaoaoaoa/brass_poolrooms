(()=>{"use strict";try{if(!self.WebGL2RenderingContext||!self.Promise||!self.PointerEvent||!self.matchMedia||matchMedia("(prefers-reduced-motion: reduce)").matches||navigator.connection&&navigator.connection.saveData)return}catch{return}const m=document.currentScript;if(!m||!m.src)return;const g=new URL(".",m.src),v=252,p=6,T=8*1024*1024,d=2,x=1e4,_=250,A=`#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`,R=`#version 300 es
precision highp float;

const int WAKE_SLOTS = ${p};
const float TILE_PITCH = ${v}.0;
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
  if (flow == vec2(0.0)) discard;
  color = vec4(submerged(document_px, flow), min(1.0, 4.0 * REFRACT_PX * length(flow)));
}`;class u{static R(){const s=Math.max(1,Math.ceil(devicePixelRatio));if(!isFinite(s)||s>3)return null;const i=document.createElement("canvas");i.className="riptide",i.setAttribute("aria-hidden","true"),i.style.cssText="position:absolute;z-index:0;pointer-events:none",i.width=i.height=1;const r=a=>a.preventDefault();i.addEventListener("webglcontextcreationerror",r);const t=i.getContext("webgl2",{alpha:!0,antialias:!1,depth:!1,failIfMajorPerformanceCaveat:!0,powerPreference:"low-power",premultipliedAlpha:!1,preserveDrawingBuffer:!1,stencil:!1});if(i.removeEventListener("webglcontextcreationerror",r),!t)return null;const n=new Image;return new Promise((a,e)=>{n.onload=()=>a(n),n.onerror=e,n.src=new URL(s===1?"brass-tiles.png":`brass-tiles@${s}x.png`,g).href}).then(a=>{const e=t.createProgram(),o=[[t.VERTEX_SHADER,A],[t.FRAGMENT_SHADER,R]];for(let c=0;c<o.length;++c){const f=t.createShader(o[c][0]);if(t.shaderSource(f,o[c][1]),t.compileShader(f),!t.getShaderParameter(f,t.COMPILE_STATUS))throw 0;t.attachShader(e,f),t.deleteShader(f)}if(t.linkProgram(e),!t.getProgramParameter(e,t.LINK_STATUS))throw 0;t.useProgram(e),t.bindVertexArray(t.createVertexArray()),t.activeTexture(t.TEXTURE0),t.bindTexture(t.TEXTURE_2D,t.createTexture()),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MIN_FILTER,t.LINEAR),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MAG_FILTER,t.LINEAR),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_S,t.REPEAT),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_T,t.REPEAT),t.texImage2D(t.TEXTURE_2D,0,t.RGBA,t.RGBA,t.UNSIGNED_BYTE,a);const h=c=>t.getUniformLocation(e,c);return t.uniform1i(h("floor_tex"),0),t.clearColor(0,0,0,0),new u(i,t,s,[h("density"),h("origin"),h("tide"),h("viewport"),h("wakes[0]")])}).catch(()=>(u.g(t),null))}static g(s){try{const i=s.getExtension("WEBGL_lose_context");i&&i.loseContext()}catch{}}constructor(s,i,r,t){this.e=s,this.t=i,this.w=r,this.c=t,this.o=new Float32Array(p*4),this.M=0,this.n=0,this.r=null,this.u=[],this.d=null,this.v=null,this.m=0,this.a=!1,this.h=[]}bind(){const s=(o,h)=>this.l(o.clientX+scrollX,o.clientY+scrollY,h*d),i=o=>o instanceof Element?o.closest(".poolrooms-frame,.plate")||o.closest("a,button"):null;this.i(self,"pointerdown",o=>s(o,1.6),{passive:!0}),this.i(self,"pointermove",o=>this.y(o),{passive:!0});const r=o=>h=>{const c=this.r&&i(h.target);c&&!c.contains(h.relatedTarget)&&s(h,o)};this.i(self,"pointerover",r(.9),{passive:!0}),this.i(self,"pointerout",r(.42),{passive:!0}),this.i(self,"focusin",o=>{const h=i(o.target);if(!h)return;const c=h.getBoundingClientRect();this.l(c.left+c.width/2+scrollX,c.top+c.height/2+scrollY,.9*d)});const t=()=>{this.r=null};this.i(self,"scroll",t,{passive:!0}),this.i(self,"resize",t,{passive:!0}),this.i(document,"visibilitychange",()=>{document.hidden&&this.f("ready")});const n=matchMedia("(prefers-reduced-motion: reduce)"),a=o=>{o.matches&&this.s()};n.addEventListener?this.i(n,"change",a):(n.addListener(a),this.h.push(()=>n.removeListener(a)));const e=navigator.connection;e&&e.addEventListener&&this.i(e,"change",()=>{e.saveData&&this.s()}),this.i(self,"pagehide",o=>{o.persisted||this.s()}),this.i(this.e,"webglcontextlost",()=>this.s()),document.body.dataset.riptideRain==="passing"&&(this.v=[0,0,0,0,0].map(()=>Math.random()*Math.PI*2),this.m=setTimeout(()=>this.T(),Math.max(0,x-performance.now()))),document.documentElement.setAttribute("data-riptide","ready")}i(s,i,r,t){const n=a=>{if(!this.a)try{r(a)}catch{this.s()}};s.addEventListener(i,n,t),this.h.push(()=>s.removeEventListener(i,n,t))}y(s){if(s.pointerType!=="mouse")return;const i=performance.now(),r=[s.clientX+scrollX,s.clientY+scrollY,i];if(!this.r){this.r=r;return}const t=this.r[0],n=this.r[1],a=this.r[2],e=Math.hypot(r[0]-t,r[1]-n);e<54||i-a<72||(this.r=r,this.l(r[0],r[1],Math.min(.72,.2+e/Math.max(i-a,1)*.15)*d))}T(){if(!this.a)try{const s=performance.now()/1e3,i=this.v,r=(Math.sin(s/29+i[0])+Math.sin(s/43+i[1])+Math.sin(s/71+i[2]))/3,t=Math.min(1,Math.max(0,(r-.54)/.46));t&&!document.hidden&&Math.random()<t&&this.P(s,i,t),this.m=setTimeout(()=>this.T(),_)}catch{this.s()}}P(s,i,r){const t=()=>Math.random()+Math.random()-1,n=.5+.42*Math.sin(s/53+i[3])+.24*t(),a=.5+.42*Math.sin(s/67+i[4])+.24*t();this.l(n*innerWidth+scrollX,a*innerHeight+scrollY,(.35+.45*r)*d)}l(s,i,r){if(this.a)return;if(!this.e.parentNode&&!this.S()){this.s();return}const t=this.M++%p*4;this.o.set([s,i,performance.now()/1e3,r],t),this.x()}S(){const s=innerWidth,i=innerHeight,r=devicePixelRatio;if(!isFinite(r)||s<=0||i<=0||r<=0||r>this.w)return!1;const t=Math.ceil(s*r),n=Math.ceil(i*r),a=this.t.getParameter(this.t.MAX_VIEWPORT_DIMS),e=this.t.getParameter(this.t.MAX_RENDERBUFFER_SIZE);return t*n>T||t>e||n>e||t>a[0]||n>a[1]||(this.d=r,this.e.style.width=`${s}px`,this.e.style.height=`${i}px`,this.e.width=t,this.e.height=n,this.t.drawingBufferWidth!==t||this.t.drawingBufferHeight!==n)?!1:(this.t.viewport(0,0,t,n),document.body.prepend(this.e),!0)}x(){!this.a&&!document.hidden&&!this.n&&(this.n=requestAnimationFrame(s=>{try{this.I(s)}catch{this.s()}}))}f(s){this.n&&cancelAnimationFrame(this.n),this.n=0,this.o.fill(0),this.e.remove(),this.e.width=this.e.height=1,this.d=null,s?document.documentElement.setAttribute("data-riptide",s):document.documentElement.removeAttribute("data-riptide")}s(){if(!this.a){for(this.a=!0,clearTimeout(this.m);this.h.length;)try{this.h.pop()()}catch{}u.g(this.t);try{this.f(null)}catch{}}}b(s,i,r){const t=Array.prototype.map.call(document.querySelectorAll(".poolrooms-frame,.plate"),e=>{const o=e.getBoundingClientRect();return{_:Math.max(0,o.left),p:Math.max(0,o.top),A:Math.min(s,o.right),E:Math.min(i,o.bottom)}}).filter(e=>e._<e.A&&e.p<e.E),n=[0,i];for(let e=0;e<t.length;++e)n.push(t[e].p,t[e].E);n.sort((e,o)=>e-o);for(let e=n.length-1;e>0;--e)n[e]===n[e-1]&&n.splice(e,1);const a=[];for(let e=1;e<n.length;++e){const o=n[e-1],h=n[e],c=t.filter(l=>l.p<h&&l.E>o).map(l=>[l._,l.A]).sort((l,w)=>l[0]-w[0]);let f=0;for(let l=0;l<c.length;++l)c[l][0]>f&&a.push([f,o,c[l][0],h]),f=Math.max(f,c[l][1]);f<s&&a.push([f,o,s,h])}this.u=a.map(e=>{const o=Math.floor(e[0]*r),h=Math.ceil(e[2]*r),c=Math.floor((i-e[3])*r),f=Math.ceil((i-e[1])*r);return[o,c,h-o,f-c]})}I(s){this.n=0;const i=this.d,r=innerWidth,t=innerHeight,n=this.e.width,a=this.e.height;if(devicePixelRatio!==i||n!==Math.ceil(r*i)||a!==Math.ceil(t*i)){this.f("ready");return}const e=s/1e3,o=(Math.hypot(r,t)+56)/320;let h=!1;for(let l=2;l<this.o.length;l+=4)this.o[l+1]>0&&e-this.o[l]<o?h=!0:this.o[l+1]=0;const c=scrollX,f=scrollY;this.b(r,t,i),this.e.style.left=`${c}px`,this.e.style.top=`${f}px`,this.t.uniform1f(this.c[0],i),this.t.uniform2f(this.c[1],c,f),this.t.uniform1f(this.c[2],e),this.t.uniform2f(this.c[3],n,a),this.t.uniform4fv(this.c[4],this.o),this.t.disable(this.t.SCISSOR_TEST),this.t.clear(this.t.COLOR_BUFFER_BIT),this.t.enable(this.t.SCISSOR_TEST);for(let l=0;l<this.u.length;++l)this.t.scissor(...this.u[l]),this.t.drawArrays(this.t.TRIANGLES,0,3);document.documentElement.setAttribute("data-riptide","live"),h?this.x():this.f("ready")}}Promise.resolve().then(()=>u.R()).then(E=>{if(E)try{E.bind()}catch{E.s()}},()=>{})})();
