#!/usr/bin/env node

/**
 * AETHER_02 — npm postinstall script
 *
 * Downloads the pre-built Linux binary from GitHub Releases and places it in
 * the package's bin/ directory so `npx @foursecondfivefour/aether-linux-mcp-server`
 * can launch the MCP server directly.
 */

const https = require("https");
const fs = require("fs");
const path = require("path");

const REPO = "foursecondfivefour/aether-linux-mcp-server";
const VERSION = process.env.AETHER_RELEASE_TAG || "latest";
const BINARY = "aether-mcp-server";
const ROOT_DIR = path.resolve(__dirname, "..", "..");
const BIN_DIR = path.join(ROOT_DIR, "bin");
const BIN_PATH = path.join(BIN_DIR, BINARY);
const MIN_SIZE = 10240; // 10 KB — catch obviously failed downloads

function releaseUrl() {
  if (VERSION === "latest") {
    return `https://github.com/${REPO}/releases/latest/download/${BINARY}`;
  }
  return `https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}`;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    const request = https.get(url, (response) => {
      if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        file.close();
        if (fs.existsSync(dest)) fs.unlinkSync(dest);
        return download(response.headers.location, dest).then(resolve).catch(reject);
      }

      if (response.statusCode !== 200) {
        file.close();
        if (fs.existsSync(dest)) fs.unlinkSync(dest);
        return reject(new Error(`HTTP ${response.statusCode} for ${url}`));
      }

      response.pipe(file);
      file.on("finish", () => {
        file.close();
        resolve();
      });
    });

    request.on("error", (err) => {
      file.close();
      if (fs.existsSync(dest)) fs.unlinkSync(dest);
      reject(err);
    });

    request.setTimeout(300000, () => {
      request.destroy();
      reject(new Error("Download timed out after 300s"));
    });
  });
}

function verifyBinary(filePath) {
  const stats = fs.statSync(filePath);
  if (stats.size < MIN_SIZE) {
    throw new Error(`Downloaded file too small: ${stats.size} bytes (expected >= ${MIN_SIZE})`);
  }

  const fd = fs.openSync(filePath, "r");
  const buffer = Buffer.alloc(4);
  fs.readSync(fd, buffer, 0, 4, 0);
  fs.closeSync(fd);

  // Linux ELF executables start with 0x7F 'E' 'L' 'F'.
  if (buffer[0] !== 0x7f || buffer[1] !== 0x45 || buffer[2] !== 0x4c || buffer[3] !== 0x46) {
    throw new Error("Downloaded file is not a valid Linux ELF executable");
  }
}

async function main() {
  if (process.platform !== "linux") {
    console.error(`[aether-linux-mcp-server] This package is for Linux only (detected: ${process.platform})`);
    console.error(`[aether-linux-mcp-server] Use the Linux installer on a Linux machine:`);
    console.error(`[aether-linux-mcp-server]   curl -sSL https://raw.githubusercontent.com/${REPO}/main/scripts/install/install.sh | bash`);
    process.exit(1);
  }

  if (!["x64", "arm64"].includes(process.arch)) {
    console.error(`[aether-linux-mcp-server] Supported architectures: x64, arm64 (detected: ${process.arch})`);
    process.exit(1);
  }

  if (fs.existsSync(BIN_PATH)) {
    try {
      verifyBinary(BIN_PATH);
      fs.chmodSync(BIN_PATH, 0o755);
      console.log(`[aether-linux-mcp-server] Binary already installed: ${BIN_PATH}`);
      return;
    } catch {
      console.log("[aether-linux-mcp-server] Existing binary invalid, re-downloading...");
      fs.unlinkSync(BIN_PATH);
    }
  }

  fs.mkdirSync(BIN_DIR, { recursive: true });

  const url = releaseUrl();
  console.log(`[aether-linux-mcp-server] Downloading ${VERSION} (Linux ${process.arch})...`);
  console.log(`[aether-linux-mcp-server] ${url}`);

  try {
    await download(url, BIN_PATH);
    verifyBinary(BIN_PATH);
    fs.chmodSync(BIN_PATH, 0o755);
    console.log(`[aether-linux-mcp-server] Installed: ${BIN_PATH} (${(fs.statSync(BIN_PATH).size / 1024 / 1024).toFixed(1)} MB)`);
  } catch (err) {
    if (fs.existsSync(BIN_PATH)) fs.unlinkSync(BIN_PATH);
    console.error(`[aether-linux-mcp-server] Download failed: ${err.message}`);
    console.error("[aether-linux-mcp-server] Alternative: build from source:");
    console.error(`[aether-linux-mcp-server]   git clone https://github.com/${REPO} && cd aether-linux-mcp-server && cargo build --release`);
    process.exit(1);
  }
}

main();
