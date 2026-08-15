import { cpSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { basename, join, relative } from "node:path";

const [version, revision, dirty, source, output] = process.argv.slice(2);
if (!version || !revision || !dirty || !source || !output) {
  throw new Error("usage: node assemble.mjs VERSION REVISION DIRTY SOURCE OUTPUT");
}
if (dirty !== "true" && dirty !== "false") {
  throw new Error("DIRTY must be true or false");
}

const root = new URL(".", import.meta.url);
mkdirSync(output, { recursive: true });
cpSync(source, join(output, "dist"), { recursive: true });
cpSync(new URL("README.md", root), join(output, "README.md"));
cpSync(new URL("../LICENSE", root), join(output, "LICENSE"));

const paths = [];
const walk = directory => {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) walk(path);
    else if (entry.name !== ".gitkeep") paths.push(path);
  }
};
walk(join(output, "dist"));
paths.sort();

const files = {};
for (const path of paths) {
  const bytes = readFileSync(path);
  const digest = createHash("sha256").update(bytes).digest();
  files[relative(join(output, "dist"), path).replaceAll("\\", "/")] = {
    bytes: bytes.length,
    sha256: digest.toString("hex"),
    integrity: `sha256-${digest.toString("base64")}`,
  };
}

const manifest = {
  schema: 1,
  product: "brass_poolrooms",
  version,
  web_abi: 1,
  source: {
    commit: revision,
    dirty: dirty === "true",
  },
  files,
};
writeFileSync(join(output, "dist", "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
process.stdout.write(`${basename(output)} ${Object.keys(files).length} files\n`);
