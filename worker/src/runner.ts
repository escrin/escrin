import { Body, Fetcher, Request } from '@cloudflare/workers-types/experimental';

import { ApiResponse, ApiError, wrapFetch } from './rpc.js';

type WorkerConfig = {
  code: string;
  type?: 'sw' | 'module';
  name?: string;
  schedule?: string;
  args?: object;
  config?: string | Record<string, object>;
  debug?: boolean;
};

async function parseWorkerConfig(contentType: string | null, body: Body): Promise<WorkerConfig> {
  if (!contentType || contentType === 'application/json') {
    try {
      return await body.json();
    } catch {
      throw new ApiError(400, 'the request body could not be parsed as JSON');
    }
  }
  if (
    contentType === 'application/x-www-form-urlencoded' ||
    contentType.startsWith('multipart/form-data')
  ) {
    try {
      const formData = await body.formData();
      const rawConfig = formData.get('config');
      let userConfig: Record<string, object> = {};
      if (rawConfig instanceof File) {
        userConfig = JSON.parse(await readFile(rawConfig));
      } else if (typeof rawConfig === 'string') {
        userConfig = JSON.parse(rawConfig);
      }
      // TODO: validation
      const workerConfig: any = {
        config: userConfig,
      };
      for (const [k, v] of formData.entries()) {
        workerConfig[k] = v instanceof File ? await readFile(v) : v;
      }
      return workerConfig;
    } catch (e: any) {
      throw new ApiError(400, `the request body could not be parsed as form data: ${e}`);
    }
  }
  throw new ApiError(400, `unsupported content type: ${contentType}`);
}

const readFile = (file: File): Promise<string> => new Response(file).text();

type RunnerEnv = {
  workerd: { newWorker(args: any): Promise<Fetcher> };
  config: {
    tpm: boolean;
  };
};

export default new (class {
  #nextWorkerId = 0;
  #schedules: Map<number, ReturnType<typeof setInterval>> = new Map();

  public readonly fetch = wrapFetch(async (req: Request, env: RunnerEnv, ctx: ExecutionContext) => {
    const config = await parseWorkerConfig(req.headers.get('content-type'), req);
    if (config.schedule && config.schedule !== '*/5 * * * *') {
      throw new ApiError(400, 'unsupported schedule');
    }
    const workerId = this.#nextWorkerId++;
    const worker = await this.#newWorker(env, config);

    if (config.schedule) {
      this.#schedules.set(
        workerId,
        setInterval(
          async () => await this.#dispatchScheduledEvent(worker, workerId, config.schedule!),
          5 * 60 * 1000,
        ),
      );
    }
    ctx.waitUntil(this.#dispatchScheduledEvent(worker, workerId, ''));

    return new ApiResponse(204);
  });

  async #newWorker(env: RunnerEnv, config: WorkerConfig): Promise<Fetcher> {
    const modular = config.type === 'module';
    return env.workerd.newWorker({
      worker: {
        compatibilityDate: '2023-02-28',
        serviceWorkerScript: modular ? undefined : config.code,
        modules: modular ? [{ name: 'main', esModule: config.code }] : undefined,
        bindings: [
          { name: 'iam', service: { name: '@escrin/iam' } },
          {
            name: 'config',
            json: typeof config.config === 'string' ? config.config : JSON.stringify(config.config),
          },
          ...(env.config.tpm ? [{ name: 'tpm', service: { name: '@escrin/tpm' } }] : []),
        ],
        schedule: config.schedule,
      },
    });
  }

  async #dispatchScheduledEvent(worker: Fetcher, workerId: number, cron: string): Promise<void> {
    const { outcome, noRetry } = await worker.scheduled({ cron });
    if (noRetry) {
      clearInterval(this.#schedules.get(workerId));
      this.#schedules.delete(workerId);
    }
    if (outcome !== 'ok') {
      console.warn('dispatch of scheduled event to worker', workerId, 'failed:', outcome);
    }
  }
})();
