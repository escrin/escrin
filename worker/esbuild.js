import fs from 'node:fs';
import { basename, extname, join } from 'node:path';

import esbuild from 'esbuild';

const SRC_DIR = 'src';
const TEST_DIR = join('test', 'worker');
const ENV_DIR = join(SRC_DIR, 'env');

const srcs = [join(SRC_DIR, 'runner.ts'), ...fs.readdirSync(ENV_DIR).map((p) => join(ENV_DIR, p))];
const tests = fs.existsSync(TEST_DIR) ? fs.readdirSync(TEST_DIR).map((p) => join(TEST_DIR, p)) : [];

const ctx = await esbuild.context({
  entryPoints: [
    ...srcs.map((service) => ({
      in: extname(service) === '.ts' ? service : join(service, 'index.ts'),
      out: basename(service, extname(service)),
    })),
    ...tests.map((test) => ({
      in: extname(test) === '.ts' ? test : join(test, 'index.ts'),
      out: join('test', basename(test, extname(test))),
    })),
  ],
  bundle: true,
  format: 'esm',
  minify: true,
  outdir: join('dist', 'worker'),
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
