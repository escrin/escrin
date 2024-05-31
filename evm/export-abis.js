import { promises as fs } from 'fs';
import path from 'path';
import url from 'url';

import { canonicalize } from '@tufjs/canonical-json';

async function* findFiles(dir, ext) {
  for (const file of await fs.readdir(dir)) {
    const filePath = path.join(dir, file);
    const stats = await fs.stat(path.join(dir, file));

    if (stats.isDirectory()) {
      yield* findFiles(filePath, ext);
    } else if (stats.isFile() && path.extname(file) === ext) {
      yield file;
    }
  }
}

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const srcdir = path.join(__dirname, 'contracts');
const outdir = path.join(__dirname, 'out');
const abidir = path.join(__dirname, 'abi');

await fs.rm(abidir, { recursive: true, force: true });
await Promise.all([fs.mkdir(outdir, { recursive: true }), fs.mkdir(abidir, { recursive: true })]);

const abiStrs = [];
for await (const solFile of findFiles(srcdir, '.sol')) {
  const solOutDir = path.join(outdir, solFile);
  const stats = await fs.stat(solOutDir);
  if (!stats.isDirectory()) continue;
  for await (const abiFile of findFiles(solOutDir, '.json')) {
    const abiName = path.basename(abiFile, '.json');
    const abiPath = path.join(solOutDir, abiFile);
    const parsedAbi = JSON.parse(await fs.readFile(abiPath, 'utf-8'));
    const abi = parsedAbi.abi;
    if (abi.length === 0) continue;
    await fs.writeFile(path.join(abidir, abiFile), canonicalize(parsedAbi));
    abiStrs.push(`export const ${abiName} = ${canonicalize(abi, null, 2)} as const;`);
  }
}

const outfile = path.join(abidir, 'index.ts');
await fs.writeFile(outfile, abiStrs.join('\n\n'));
