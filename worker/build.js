const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

// Install Rust if not installed
console.log("Installing Rust toolchain...");
try {
  execSync("rustc --version");
  console.log("Rust already installed");
} catch {
  console.log("Installing Rust...");
  execSync(
    'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y',
    {
      stdio: "inherit",
    },
  );
  // Add cargo to PATH
  process.env.PATH = `${process.env.HOME}/.cargo/bin:${process.env.PATH}`;
}

// Install wasm-pack if not installed
console.log("Installing wasm-pack...");
try {
  execSync("wasm-pack --version");
  console.log("wasm-pack already installed");
} catch {
  console.log("Installing wasm-pack...");
  execSync(
    "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh",
    {
      stdio: "inherit",
    },
  );
}

if (!fs.existsSync("worker/pkg")) {
  fs.mkdirSync("worker/pkg", { recursive: true });
}

console.log("Building WebAssembly module...");
execSync(
  "wasm-pack build --target web --out-dir worker/pkg --no-default-features --features worker",
  {
    stdio: "inherit",
    cwd: process.cwd(),
  },
);

console.log("Generating TypeScript types...");
const typesContent = fs.readFileSync(
  path.join(process.cwd(), "worker", "types.d.ts"),
  "utf-8",
);

fs.writeFileSync(
  path.join(process.cwd(), "worker/pkg", "bytes_radar.d.ts"),
  typesContent,
);

console.log("Build complete!");
