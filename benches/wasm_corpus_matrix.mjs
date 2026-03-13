import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import fs from "node:fs/promises";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.dirname(here);
const pkgDir = path.join(rootDir, "pkg-web");
const packageUrl = pathToFileURL(path.join(pkgDir, "usfm_onion_web.js")).href;
const wasmPath = path.join(pkgDir, "usfm_onion_web_bg.wasm");

const pkg = await import(packageUrl);
await pkg.default({ module_or_path: await fs.readFile(wasmPath) });

const corpora = [
  { name: "examples.bsb", relativePath: "example-corpora/examples.bsb" },
  { name: "bdf_reg", relativePath: "example-corpora/bdf_reg" },
  { name: "en_ult", relativePath: "example-corpora/en_ult" },
];

const operations = [
  { label: "usfm -> ast", run: benchIntoAst },
  { label: "usfm -> tokens", run: benchIntoTokens },
  { label: "lint usfm", run: benchLintUsfm },
  { label: "format usfm", run: benchFormatUsfm },
  { label: "usfm -> usj", run: benchUsfmToUsj },
  { label: "usfm -> usx", run: benchUsfmToUsx },
  { label: "usfm -> html", run: benchUsfmToHtml },
  { label: "usfm -> vref", run: benchUsfmToVref },
  { label: "usj -> usfm", run: benchUsjToUsfm },
  { label: "usx -> usfm", run: benchUsxToUsfm },
];

const config = parseArgs(process.argv.slice(2));
const loadedCorpora = [];
for (const spec of corpora) {
  if (config.corpusFilter && !spec.name.includes(config.corpusFilter)) {
    continue;
  }
  loadedCorpora.push(await loadCorpus(spec));
}

const results = [];
for (const corpus of loadedCorpora) {
  const timings = [];
  for (const operation of operations) {
    if (config.operationFilter && !operation.label.includes(config.operationFilter)) {
      continue;
    }
    console.error(`benchmarking wasm ${corpus.name} / ${operation.label}`);
    const duration = measure(operation.run, corpus, config.iterations, config.warmup);
    timings.push({ operation: operation.label, duration });
  }
  results.push({ corpus, timings, iterations: config.iterations });
}

if (config.markdown) {
  process.stdout.write(renderMarkdown(results));
} else {
  process.stdout.write(renderText(results));
}

function parseArgs(args) {
  let iterations = 3;
  let warmup = 1;
  let markdown = false;
  let corpusFilter = null;
  let operationFilter = null;

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--iterations" && args[index + 1]) {
      iterations = Number(args[index + 1]);
      index += 1;
      continue;
    }
    if (arg === "--warmup" && args[index + 1]) {
      warmup = Number(args[index + 1]);
      index += 1;
      continue;
    }
    if (arg === "--corpus" && args[index + 1]) {
      corpusFilter = args[index + 1];
      index += 1;
      continue;
    }
    if (arg === "--operation" && args[index + 1]) {
      operationFilter = args[index + 1];
      index += 1;
      continue;
    }
    if (arg === "--markdown") {
      markdown = true;
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      process.stdout.write(
        "Usage: node benches/wasm_corpus_matrix.mjs [--iterations N] [--warmup N] [--corpus NAME] [--operation TEXT] [--markdown]\n",
      );
      process.exit(0);
    }
  }

  return { iterations, warmup, markdown, corpusFilter, operationFilter };
}

async function loadCorpus(spec) {
  const root = path.join(rootDir, spec.relativePath);
  const files = (await collectUsfmFiles(root)).sort();
  const usfmSources = await Promise.all(files.map((file) => fs.readFile(file, "utf8")));
  const totalUsfmBytes = usfmSources.reduce((sum, source) => sum + Buffer.byteLength(source), 0);

  const usjSources = usfmSources.map((source) => pkg.usfmToUsj(source));
  const usxSources = usfmSources.map((source) => pkg.usfmToUsx(source));

  return {
    name: spec.name,
    usfmSources,
    usjSources,
    usxSources,
    usfmFileCount: usfmSources.length,
    totalFileCount: await countFiles(root),
    totalUsfmBytes,
  };
}

async function collectUsfmFiles(root) {
  const entries = await fs.readdir(root, { withFileTypes: true });
  const out = [];
  for (const entry of entries) {
    const next = path.join(root, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await collectUsfmFiles(next)));
      continue;
    }
    if (/\.(usfm|sfm)$/i.test(entry.name)) {
      out.push(next);
    }
  }
  return out;
}

async function countFiles(root) {
  const entries = await fs.readdir(root, { withFileTypes: true });
  let count = 0;
  for (const entry of entries) {
    const next = path.join(root, entry.name);
    if (entry.isDirectory()) {
      count += await countFiles(next);
    } else {
      count += 1;
    }
  }
  return count;
}

function measure(run, corpus, iterations, warmup) {
  for (let index = 0; index < warmup; index += 1) {
    run(corpus);
  }

  const samples = [];
  for (let index = 0; index < iterations; index += 1) {
    const start = performance.now();
    const checksum = run(corpus);
    void checksum;
    samples.push(performance.now() - start);
  }
  samples.sort((left, right) => left - right);
  return samples[Math.floor(samples.length / 2)];
}

function benchIntoAst(corpus) {
  return corpus.usfmSources.reduce(
    (sum, source) => sum + pkg.usfmToAst(source).content.length,
    0,
  );
}

function benchIntoTokens(corpus) {
  return corpus.usfmSources.reduce((sum, source) => sum + pkg.usfmToTokens(source).length, 0);
}

function benchLintUsfm(corpus) {
  return pkg
    .lintContents({
      sources: corpus.usfmSources,
      format: "usfm",
      batchOptions: { parallel: false },
    })
    .reduce((sum, row) => sum + (row.error ? 0 : row.value.length), 0);
}

function benchFormatUsfm(corpus) {
  return pkg
    .formatContents({
      sources: corpus.usfmSources,
      format: "usfm",
      batchOptions: { parallel: false },
    })
    .reduce((sum, row) => sum + (row.error ? 0 : row.value.tokens.length), 0);
}

function benchUsfmToUsj(corpus) {
  return corpus.usfmSources.reduce(
    (sum, source) => sum + JSON.stringify(pkg.usfmToUsj(source)).length,
    0,
  );
}

function benchUsfmToUsx(corpus) {
  return corpus.usfmSources.reduce((sum, source) => sum + pkg.usfmToUsx(source).length, 0);
}

function benchUsfmToHtml(corpus) {
  return corpus.usfmSources.reduce((sum, source) => sum + pkg.usfmToHtml(source).length, 0);
}

function benchUsfmToVref(corpus) {
  return corpus.usfmSources.reduce((sum, source) => sum + pkg.usfmToVref(source).length, 0);
}

function benchUsjToUsfm(corpus) {
  return corpus.usjSources.reduce((sum, source) => sum + pkg.fromUsj(source).length, 0);
}

function benchUsxToUsfm(corpus) {
  return corpus.usxSources.reduce((sum, source) => sum + pkg.usxToUsfm(source).length, 0);
}

function renderText(results) {
  let out = "WASM corpus timing matrix\n========================\n\n";
  for (const result of results) {
    out += `${result.corpus.name}: ${result.corpus.usfmFileCount} USFM files, ${result.corpus.totalFileCount} total files, ${bytesToMib(result.corpus.totalUsfmBytes).toFixed(2)} MiB\n`;
    for (const timing of result.timings) {
      out += `  ${timing.operation.padEnd(24)} ${formatDuration(timing.duration)}\n`;
    }
    out += "\n";
  }
  return out;
}

function renderMarkdown(results) {
  let out = "<!-- wasm-corpus-bench:begin -->\n";
  if (results[0]) {
    out += `_Local release measurements of the web-target WASM package in Node, median wall-clock over ${results[0].iterations} runs, after module initialization._\n\n`;
  }
  for (const result of results) {
    out += `### \`${result.corpus.name}\`\n\n`;
    out += `- USFM files: ${result.corpus.usfmFileCount}\n`;
    out += `- Total files in corpus directory: ${result.corpus.totalFileCount}\n`;
    out += `- Total USFM source size: ${bytesToMib(result.corpus.totalUsfmBytes).toFixed(2)} MiB\n\n`;
    out += "| Operation | WASM |\n";
    out += "| --- | ---: |\n";
    for (const timing of result.timings) {
      out += `| ${timing.operation} | ${formatDuration(timing.duration)} |\n`;
    }
    out += "\n";
  }
  out += "<!-- wasm-corpus-bench:end -->\n";
  return out;
}

function formatDuration(ms) {
  if (ms >= 1000) {
    return `${(ms / 1000).toFixed(2)}s`;
  }
  return `${ms.toFixed(1)}ms`;
}

function bytesToMib(bytes) {
  return bytes / (1024 * 1024);
}
