(()=>{"use strict";try{if(!self.WebGL2RenderingContext||!self.Promise||!self.PointerEvent||!self.matchMedia||matchMedia("(prefers-reduced-motion: reduce)").matches||navigator.connection&&navigator.connection.saveData)return}catch{return}const u=document.currentScript;if(!u||!u.src)return;const p=new URL(".",u.src),v=252,d=6,E=8*1024*1024,m=2,x=`#version 300 es
void main() {
  vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
  gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}`,T=`#version 300 es
precision highp float;

const int WAKE_SLOTS = ${d};
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
  color = vec4(submerged(document_px, flow), 1.0);
}`;class f{static forge(){const s=Math.max(1,Math.ceil(devicePixelRatio));if(!isFinite(s)||s>3)return null;const i=document.createElement("canvas");i.className="riptide",i.setAttribute("aria-hidden","true"),i.style.cssText="position:absolute;z-index:0;opacity:0;pointer-events:none",i.width=i.height=1;const o=a=>a.preventDefault();i.addEventListener("webglcontextcreationerror",o);const t=i.getContext("webgl2",{alpha:!0,antialias:!1,depth:!1,desynchronized:!0,failIfMajorPerformanceCaveat:!0,powerPreference:"low-power",premultipliedAlpha:!1,preserveDrawingBuffer:!1,stencil:!1});if(i.removeEventListener("webglcontextcreationerror",o),!t)return null;const n=new Image;return new Promise((a,e)=>{n.onload=()=>a(n),n.onerror=e,n.src=new URL(s===1?"brass-tiles.png":`brass-tiles@${s}x.png`,p).href}).then(a=>{const e=t.createProgram(),r=[[t.VERTEX_SHADER,x],[t.FRAGMENT_SHADER,T]];for(let _=0;_<r.length;++_){const h=t.createShader(r[_][0]);if(t.shaderSource(h,r[_][1]),t.compileShader(h),!t.getShaderParameter(h,t.COMPILE_STATUS))throw 0;t.attachShader(e,h),t.deleteShader(h)}if(t.linkProgram(e),!t.getProgramParameter(e,t.LINK_STATUS))throw 0;t.useProgram(e),t.bindVertexArray(t.createVertexArray()),t.activeTexture(t.TEXTURE0),t.bindTexture(t.TEXTURE_2D,t.createTexture()),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MIN_FILTER,t.LINEAR),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_MAG_FILTER,t.LINEAR),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_S,t.REPEAT),t.texParameteri(t.TEXTURE_2D,t.TEXTURE_WRAP_T,t.REPEAT),t.texImage2D(t.TEXTURE_2D,0,t.RGBA,t.RGBA,t.UNSIGNED_BYTE,a);const l=_=>t.getUniformLocation(e,_);return t.uniform1i(l("floor_tex"),0),t.clearColor(0,0,0,0),new f(i,t,s,{density:l("density"),origin:l("origin"),tide:l("tide"),viewport:l("viewport"),wakes:l("wakes[0]")})}).catch(()=>(f.quench(t),null))}static quench(s){try{const i=s.getExtension("WEBGL_lose_context");i&&i.loseContext()}catch{}}constructor(s,i,o,t){this._canvas=s,this._gl=i,this._tileDpr=o,this._uniforms=t,this._wakes=new Float32Array(d*4),this._victim=0,this._frame=0,this._motion=null,this._flood=[],this._field=null,this._dead=!1,this._unbind=[]}bind(){const s=(e,r)=>this._strike(e.clientX+scrollX,e.clientY+scrollY,r*m),i=e=>e instanceof Element?e.closest(".poolrooms-frame,.plate")||e.closest("a,button"):null;this._listen(self,"pointerdown",e=>s(e,1.6),{passive:!0}),this._listen(self,"pointermove",e=>this._trail(e),{passive:!0}),this._listen(self,"pointerover",e=>{const r=i(e.target);r&&!r.contains(e.relatedTarget)&&s(e,.9)},{passive:!0}),this._listen(self,"pointerout",e=>{const r=i(e.target);r&&!r.contains(e.relatedTarget)&&s(e,.42)},{passive:!0}),this._listen(self,"focusin",e=>{const r=i(e.target);if(!r)return;const l=r.getBoundingClientRect();this._strike(l.left+l.width/2+scrollX,l.top+l.height/2+scrollY,.9*m)});const o=()=>{this._motion=null,(this._canvas.parentNode||this._frame)&&this._still("ready")};this._listen(self,"scroll",o,{passive:!0}),this._listen(self,"resize",o,{passive:!0}),this._listen(document,"visibilitychange",()=>{document.hidden&&this._still("ready")});const t=matchMedia("(prefers-reduced-motion: reduce)"),n=e=>{e.matches&&this._abort()};t.addEventListener?this._listen(t,"change",n):(t.addListener(n),this._unbind.push(()=>t.removeListener(n)));const a=navigator.connection;a&&a.addEventListener&&this._listen(a,"change",()=>{a.saveData&&this._abort()}),this._listen(self,"pagehide",e=>{e.persisted||this._abort()}),this._listen(this._canvas,"webglcontextlost",()=>this._abort()),document.documentElement.setAttribute("data-riptide","ready")}_listen(s,i,o,t){const n=a=>{if(!this._dead)try{o(a)}catch{this._abort()}};s.addEventListener(i,n,t),this._unbind.push(()=>s.removeEventListener(i,n,t))}_trail(s){if(s.pointerType!=="mouse")return;const i=performance.now(),o=[s.clientX+scrollX,s.clientY+scrollY,i];if(!this._motion){this._motion=o;return}const t=this._motion[0],n=this._motion[1],a=this._motion[2],e=Math.hypot(o[0]-t,o[1]-n);e<54||i-a<72||(this._motion=o,this._strike(o[0],o[1],Math.min(.72,.2+e/Math.max(i-a,1)*.15)*m))}_strike(s,i,o){if(this._dead)return;if(!this._canvas.parentNode&&!this._raise()){this._abort();return}const t=this._victim++%d*4;this._wakes.set([s,i,performance.now()/1e3,o],t),this._summon()}_raise(){const s=innerWidth,i=innerHeight,o=devicePixelRatio;if(!isFinite(o)||s<=0||i<=0||o<=0||o>this._tileDpr)return!1;const t=Math.ceil(s*o),n=Math.ceil(i*o),a=this._gl.getParameter(this._gl.MAX_VIEWPORT_DIMS),e=this._gl.getParameter(this._gl.MAX_RENDERBUFFER_SIZE);if(t*n>E||t>e||n>e||t>a[0]||n>a[1])return!1;const r=scrollX,l=scrollY;return this._field=[o,r,l],this._canvas.style.left=`${r}px`,this._canvas.style.top=`${l}px`,this._canvas.style.width=`${s}px`,this._canvas.style.height=`${i}px`,this._canvas.width=t,this._canvas.height=n,this._gl.drawingBufferWidth!==t||this._gl.drawingBufferHeight!==n?!1:(this._gl.viewport(0,0,t,n),this._excavate(s,i,o),document.body.insertBefore(this._canvas,document.body.firstChild),!0)}_summon(){!this._dead&&!document.hidden&&!this._frame&&(this._frame=requestAnimationFrame(s=>{try{this._render(s)}catch{this._abort()}}))}_still(s){this._frame&&cancelAnimationFrame(this._frame),this._frame=0,this._wakes.fill(0),this._canvas.style.opacity=0,this._canvas.parentNode&&this._canvas.parentNode.removeChild(this._canvas),this._canvas.width=this._canvas.height=1,this._field=null,s?document.documentElement.setAttribute("data-riptide",s):document.documentElement.removeAttribute("data-riptide")}_abort(){if(!this._dead){for(this._dead=!0;this._unbind.length;)try{this._unbind.pop()()}catch{}f.quench(this._gl);try{this._still(null)}catch{}}}_excavate(s,i,o){const t=Array.prototype.map.call(document.querySelectorAll(".poolrooms-frame,.plate"),e=>{const r=e.getBoundingClientRect();return{left:Math.max(0,r.left),top:Math.max(0,r.top),right:Math.min(s,r.right),bottom:Math.min(i,r.bottom)}}).filter(e=>e.left<e.right&&e.top<e.bottom),n=[0,i];for(let e=0;e<t.length;++e)n.push(t[e].top,t[e].bottom);n.sort((e,r)=>e-r);for(let e=n.length-1;e>0;--e)n[e]===n[e-1]&&n.splice(e,1);const a=[];for(let e=1;e<n.length;++e){const r=n[e-1],l=n[e],_=t.filter(c=>c.top<l&&c.bottom>r).map(c=>[c.left,c.right]).sort((c,w)=>c[0]-w[0]);let h=0;for(let c=0;c<_.length;++c)_[c][0]>h&&a.push([h,r,_[c][0],l]),h=Math.max(h,_[c][1]);h<s&&a.push([h,r,s,l])}this._flood=a.map(e=>{const r=Math.floor(e[0]*o),l=Math.ceil(e[2]*o),_=Math.floor((i-e[3])*o),h=Math.ceil((i-e[1])*o);return[r,_,l-r,h-_]})}_render(s){this._frame=0;const[i,o,t]=this._field,n=innerWidth,a=innerHeight,e=this._canvas.width,r=this._canvas.height;if(devicePixelRatio!==i||e!==Math.ceil(n*i)||r!==Math.ceil(a*i)){this._still("ready");return}const l=s/1e3,_=(Math.hypot(n,a)+56)/320;let h=!1;for(let c=2;c<this._wakes.length;c+=4)this._wakes[c+1]>0&&l-this._wakes[c]<_?h=!0:this._wakes[c+1]=0;this._gl.uniform1f(this._uniforms.density,i),this._gl.uniform2f(this._uniforms.origin,o,t),this._gl.uniform1f(this._uniforms.tide,l),this._gl.uniform2f(this._uniforms.viewport,e,r),this._gl.uniform4fv(this._uniforms.wakes,this._wakes),this._gl.disable(this._gl.SCISSOR_TEST),this._gl.clear(this._gl.COLOR_BUFFER_BIT),this._gl.enable(this._gl.SCISSOR_TEST);for(let c=0;c<this._flood.length;++c)this._gl.scissor(...this._flood[c]),this._gl.drawArrays(this._gl.TRIANGLES,0,3);this._canvas.style.opacity=1,document.documentElement.setAttribute("data-riptide","live"),h?this._summon():this._still("ready")}}Promise.resolve().then(()=>f.forge()).then(g=>{if(g)try{g.bind()}catch{g._abort()}},()=>{})})();
