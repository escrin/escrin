import { Body, Fetcher, Request } from '@cloudflare/workers-types/experimental';

import { ApiResponse, ApiError, ErrorResponse, wrapFetch } from '../rpc.js';

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
      let config: Record<string, object> = {};
      if (rawConfig instanceof File) {
        throw new ApiError(400, 'unsupported config format');
      } else if (typeof rawConfig === 'string') {
        config = JSON.parse(rawConfig);
      }
      // TODO: validation
      return {
        ...Object.fromEntries(formData.entries()),
        config,
      } as any;
    } catch {
      throw new ApiError(400, 'the request body could not be parsed as form data');
    }
  }
  throw new ApiError(400, `unsupported content type: ${contentType}`);
}

type WorkerId = string;

type RunnerEnv = {
  workerd: Fetcher;
  waker: DurableObjectNamespace;
};

export default new (class {
  public readonly fetch = wrapFetch(async (req: Request, env: RunnerEnv) => {
    const config = await parseWorkerConfig(req.headers.get('content-type'), req);
    const workerId = await this.#createWorker(env.workerd, config);

    if (config.schedule) {
      const id = env.waker.idFromName('default');
      const scheduleRes = await env.waker.get(id).fetch('http://waker', {
        body: JSON.stringify({
          worker: workerId,
          schedule: config.schedule,
        }),
      });
      if (!scheduleRes.ok) return scheduleRes;
    }

    return new ApiResponse(201, {
      id: workerId,
    });
  });

  async #createWorker(workerd: Fetcher, config: WorkerConfig): Promise<WorkerId> {
    const modular = config.type === 'module';
    const res = await workerd.fetch('http://workerd.escrin/workers', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        worker: {
          compatibilityDate: '2023-02-28',
          serviceWorkerScript: modular ? undefined : config.code,
          modules: modular ? [{ name: 'main', esModule: config.code }] : undefined,
          bindings: [
            { name: 'escrin', service: '@escrin/env' },
            {
              name: 'config',
              json:
                typeof config.config === 'string' ? config.config : JSON.stringify(config.config),
            },
          ],
          schedule: config.schedule,
        },
      }),
    });

    const { name: id }: { name: WorkerId } = await res.json();
    return id;
  }
})();
