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

type WorkerId = string;

type RunnerEnv = {
  workerd: Fetcher;
  mode: string;
  config: {
    tpm: boolean;
  };
};

export default new (class {
  #schedules: Map<string, ReturnType<typeof setInterval>> = new Map();

  public readonly fetch = wrapFetch(async (req: Request, env: RunnerEnv, ctx: ExecutionContext) => {
    const config = await parseWorkerConfig(req.headers.get('content-type'), req);
    if (config.schedule && config.schedule !== '*/5 * * * *') {
      throw new ApiError(400, 'unsupported schedule');
    }
    const workerId = await this.#createWorker(env, config);

    if (config.schedule) {
      const cron = config.schedule;
      let iters = 0;
      const interval = setInterval(
        async () => {
          if (env.mode === 'demo') {
            iters++;
            if (iters === 5) {
              clearInterval(interval);
              this.#schedules.delete(workerId);
            }
          }
          await this.#dispatchScheduledEvent(env.workerd, workerId, cron);
        },
        5 * 60 * 1000,
      );
      this.#schedules.set(workerId, interval);
    }
    ctx.waitUntil(this.#dispatchScheduledEvent(env.workerd, workerId, ''));

    return new ApiResponse(201, {
      id: workerId,
    });
  });

  async #createWorker(env: RunnerEnv, config: WorkerConfig): Promise<WorkerId> {
    const modular = config.type === 'module';
    const res = await env.workerd.fetch('http://workerd.local/workers', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
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
      }),
    });

    const { name: id }: { name: WorkerId } = await res.json();
    return id;
  }

  async #dispatchScheduledEvent(workerd: Fetcher, workerId: string, cron: string): Promise<void> {
    const res = await workerd.fetch(`http://workerd.local/workers/${workerId}/events/scheduled`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        cron,
        scheduledTime: Date.now(),
      }),
    });
    if (!res.ok) {
      console.error('failed to post scheduled event', await res.text());
    }
  }
})();
