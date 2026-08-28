import init from "./poolrooms.js";

const root = document.documentElement;
const status = document.getElementById("status");
const fail = error => {
  const message = error instanceof Error ? error.message : String(error);
  root.dataset.poolrooms = "failed";
  status.textContent = "WEBGPU RASTER ATELIER FAILED · " + message;
};

try {
  if (!window.isSecureContext) {
    throw new Error("serve this directory over HTTPS or localhost");
  }
  if (!navigator.gpu) {
    throw new Error("this browser does not expose WebGPU");
  }
  const adapter = await navigator.gpu.requestAdapter({
    powerPreference: "high-performance",
  });
  if (!adapter) {
    throw new Error("no usable WebGPU adapter");
  }
  const info = adapter.info ?? {};
  const identity = [
    info.vendor,
    info.architecture,
    info.device,
    info.description,
    info.backend,
    info.type,
  ].filter(Boolean).join(" ").toLowerCase();
  const software = ["swiftshader", "llvmpipe", "lavapipe", "software", "cpu"]
    .find(marker => identity.includes(marker));
  root.dataset.webgpuAdapter = identity || "undisclosed";
  root.dataset.webgpuSoftware = software ?? "hardware";
  root.dataset.devicePixelRatio = String(window.devicePixelRatio);
  await init();
  window.setTimeout(() => {
    if (root.dataset.poolrooms === "booting") {
      fail(new Error("the first rendered frame did not arrive"));
    }
  }, 15_000);
} catch (error) {
  fail(error);
}
