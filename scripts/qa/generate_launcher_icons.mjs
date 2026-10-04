/** Ordinary icon-format conversion only. Never redraw, crop or modify the approved source. */
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const EXPECTED_SOURCE = 'e2d6078dee6c4c5f57b3e2da67bf2c9b8b2357252e914850528e977b7264e88d';
const EXPECTED_SIZES = [16, 24, 32, 48, 64, 256];
const ARTIFACTS = ['icon.png', 'icon.ico', '32x32.png', '64x64.png', '128x128.png', '128x128@2x.png'];
const PNG_SIGNATURE = Buffer.from('89504e470d0a1a0a', 'hex');
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const assert = (condition, message) => { if (!condition) throw new Error(message); };

function crc32(bytes) {
  let result = 0xffffffff;
  for (const byte of bytes) {
    result ^= byte;
    for (let n = 0; n < 8; n++) result = (result >>> 1) ^ ((result & 1) ? 0xedb88320 : 0);
  }
  return (result ^ 0xffffffff) >>> 0;
}

function inspectPng(bytes) {
  assert(bytes.length <= 8 * 1024 * 1024 && bytes.subarray(0, 8).equals(PNG_SIGNATURE), 'Invalid or oversized PNG');
  let cursor = 8, metadata, ended = false;
  const compressed = [];
  while (cursor + 12 <= bytes.length) {
    const length = bytes.readUInt32BE(cursor), end = cursor + 12 + length;
    assert(end <= bytes.length, 'PNG chunk exceeds file');
    const type = bytes.subarray(cursor + 4, cursor + 8).toString('ascii');
    assert(crc32(bytes.subarray(cursor + 4, end - 4)) === bytes.readUInt32BE(end - 4), `PNG ${type} checksum mismatch`);
    if (type === 'IHDR') {
      assert(cursor === 8 && length === 13 && !metadata, 'Unexpected PNG header');
      metadata = { width: bytes.readUInt32BE(cursor + 8), height: bytes.readUInt32BE(cursor + 12), depth: bytes[cursor + 16], colourType: bytes[cursor + 17], compression: bytes[cursor + 18], filter: bytes[cursor + 19], interlace: bytes[cursor + 20] };
      assert(metadata.width > 0 && metadata.width <= 2048 && metadata.height > 0 && metadata.height <= 2048, 'PNG dimensions outside QA bound');
      assert(metadata.depth === 8 && metadata.colourType === 6 && metadata.compression === 0 && metadata.filter === 0 && metadata.interlace === 0, 'Expected noninterlaced 8-bit RGBA PNG');
    } else if (type === 'IDAT') compressed.push(bytes.subarray(cursor + 8, end - 4));
    else if (type === 'IEND') { assert(length === 0 && end === bytes.length, 'Invalid PNG end'); ended = true; break; }
    cursor = end;
  }
  assert(metadata && ended && compressed.length, 'Incomplete PNG');
  const expectedInflated = (metadata.width * 4 + 1) * metadata.height;
  const inflated = zlib.inflateSync(Buffer.concat(compressed), { maxOutputLength: expectedInflated });
  assert(inflated.length === expectedInflated, 'PNG inflated length mismatch');
  return { ...metadata, bytes: bytes.length, sha256: hash(bytes), crcAndInflatedLength: 'PASS' };
}

function inspectIco(bytes) {
  assert(bytes.length <= 2 * 1024 * 1024 && bytes.readUInt16LE(0) === 0 && bytes.readUInt16LE(2) === 1, 'Invalid ICO header');
  const count = bytes.readUInt16LE(4);
  assert(count === EXPECTED_SIZES.length, 'Unexpected ICO resolution count');
  let nextOffset = 6 + count * 16;
  const entries = [];
  for (let n = 0; n < count; n++) {
    const at = 6 + n * 16, width = bytes[at] || 256, height = bytes[at + 1] || 256;
    const length = bytes.readUInt32LE(at + 8), offset = bytes.readUInt32LE(at + 12);
    assert(width === EXPECTED_SIZES[n] && height === width && bytes[at + 3] === 0 && bytes.readUInt16LE(at + 6) === 32, 'ICO directory resolution or bit-depth mismatch');
    assert(length > 0 && offset === nextOffset && offset + length <= bytes.length, 'ICO payload overlaps, contains gaps or exceeds file');
    const png = inspectPng(bytes.subarray(offset, offset + length));
    assert(png.width === width && png.height === height, 'ICO embedded PNG dimensions mismatch');
    entries.push({ width, height, bits: 32, offset, ...png });
    nextOffset += length;
  }
  assert(nextOffset === bytes.length, 'ICO trailing bytes');
  return { bytes: bytes.length, sha256: hash(bytes), entries, directoryAndEmbeddedPng: 'PASS' };
}

function main() {
  const argumentsList = process.argv.slice(2);
  assert(argumentsList.length === 4 || (argumentsList.length === 5 && argumentsList[4] === '--install'), 'Usage: node scripts/qa/generate_launcher_icons.mjs --source APPROVED_PNG --output reports/UNUSED_DIRECTORY [--install]');
  assert(argumentsList[0] === '--source' && argumentsList[2] === '--output', 'Unexpected argument order');
  const source = path.resolve(argumentsList[1]), output = path.resolve(ROOT, argumentsList[3]);
  const relativeOutput = path.relative(path.join(ROOT, 'reports'), output);
  assert(relativeOutput && !relativeOutput.startsWith('..') && !path.isAbsolute(relativeOutput), 'Output must be a new directory below this v0.2.1 worktree reports');
  assert(!fs.existsSync(output), 'Refuse to overwrite existing conversion evidence');
  const sourceStat = fs.lstatSync(source);
  assert(sourceStat.isFile() && !sourceStat.isSymbolicLink() && sourceStat.size === 2158832, 'Approved source ordinary-file or size mismatch');
  const sourceBytes = fs.readFileSync(source);
  assert(hash(sourceBytes) === EXPECTED_SOURCE, 'Approved source byte identity mismatch');
  const sourcePng = inspectPng(sourceBytes);
  assert(sourcePng.width === 1153 && sourcePng.height === 1101, 'Approved source dimensions mismatch');
  const cli = path.join(ROOT, 'node_modules/@tauri-apps/cli/tauri.js');
  const startedUtc = new Date().toISOString();
  const version = spawnSync(process.execPath, [cli, '--version'], { cwd: ROOT, encoding: 'utf8', timeout: 10000 });
  assert(version.status === 0 && version.stdout.trim() === 'tauri-cli 2.12.0', 'Expected existing locked Tauri CLI 2.12.0');
  fs.mkdirSync(output, { recursive: true });
  const cliArgs = [cli, 'icon', source, '--fit', 'contain', '--output', output];
  const conversion = spawnSync(process.execPath, cliArgs, { cwd: ROOT, encoding: 'utf8', timeout: 60000, maxBuffer: 1024 * 1024 });
  assert(conversion.status === 0, `Tauri format conversion failed: ${conversion.stderr}`);
  assert(hash(fs.readFileSync(source)) === EXPECTED_SOURCE, 'Original source changed during conversion');
  const icons = ARTIFACTS.map(name => {
    const bytes = fs.readFileSync(path.join(output, name));
    return { name, ...(name.endsWith('.ico') ? inspectIco(bytes) : inspectPng(bytes)) };
  });
  // Extract embedded PNG bytes for direct per-resolution inspection; no resampling or pixel changes.
  const icoBytes = fs.readFileSync(path.join(output, 'icon.ico'));
  for (const entry of icons.find(icon => icon.name === 'icon.ico').entries) {
    fs.writeFileSync(path.join(output, `ico-${entry.width}.png`), icoBytes.subarray(entry.offset, entry.offset + entry.bytes), { flag: 'wx' });
  }
  const expectedPngSizes = { 'icon.png': 512, '32x32.png': 32, '64x64.png': 64, '128x128.png': 128, '128x128@2x.png': 256 };
  for (const icon of icons.filter(icon => icon.name.endsWith('.png'))) assert(icon.width === expectedPngSizes[icon.name] && icon.height === icon.width, 'Generated PNG dimensions mismatch');
  const installedInSource = argumentsList.includes('--install');
  if (installedInSource) {
    for (const icon of icons) {
      const target = path.join(ROOT, 'src-tauri/icons', icon.name);
      if (fs.existsSync(target)) assert(fs.lstatSync(target).isFile() && !fs.lstatSync(target).isSymbolicLink(), 'Source icon destination is not an ordinary file');
      fs.copyFileSync(path.join(output, icon.name), target);
      assert(hash(fs.readFileSync(target)) === icon.sha256, 'Copied source icon byte mismatch');
    }
  }
  const receipt = { schema: 1, startedUtc, finishedUtc: new Date().toISOString(), source: { path: source, ...sourcePng }, node: process.version, platform: process.platform, architecture: process.arch, tauri: version.stdout.trim(), command: [process.execPath, ...cliArgs], fit: 'contain; aspect preserved; transparent short-side padding; no crop, stretch, redraw or source pixel removal', icons, installedInSource, cliStdout: conversion.stdout, cliStderr: conversion.stderr, result: 'PASS format-and-byte-structure-only', installedWindowsRendering: 'NOT RUN' };
  fs.writeFileSync(path.join(output, 'conversion-qa.json'), `${JSON.stringify(receipt, null, 2)}\n`, { flag: 'wx' });
  console.log(JSON.stringify(receipt, null, 2));
}

main();
