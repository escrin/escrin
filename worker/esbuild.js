import fs from 'node:fs';
import path from 'node:path';

import esbuild from 'esbuild';

const SRC_DIR = path.join('.', 'src', 'env');

const ctx = await esbuild.context({
  entryPoints: fs.readdirSync(SRC_DIR).map((service) => {
    const entryPoint = path.extname(service) === '.ts' ? service : path.join(service, 'index.ts');
    return {
      in: path.join(SRC_DIR, entryPoint),
      out: path.basename(service, path.extname(service)),
    };
  }),
  bundle: true,
  format: 'esm',
  minify: true,
  outdir: path.join('dist', 'env'),
  platform: 'browser',
  sourcemap: 'external',
  target: 'es2022',
});

let watch = false;
const arg = process.argv[2];
if (arg) {
  if (arg !== '--watch' && arg !== '-w') {
    console.error('unknown argument:', arg);
    process.exit(1);
  }
  watch = true;
}

if (watch) await ctx.watch();
else await ctx.rebuild();
await ctx.dispose();
