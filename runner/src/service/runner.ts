import {
  Body,
  DurableObjectNamespace,
  DurableObjectState,
  ExecutionContext,
  Fetcher,
  Request,
} from '@cloudflare/workers-types/experimental';

import { ApiError } from '../error.js';

type WorkerConfig = {
  script: string;
  type?: 'classic' | 'module';
  name?: string;
  schedule?: string;
  args?: object;
  config?: string | Record<string, object>;
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
  async fetch(req: Request, env: RunnerEnv, _ctx: ExecutionContext) {
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
  }

  async #createWorker(workerd: Fetcher, config: WorkerConfig): Promise<WorkerId> {
    const res = await workerd.fetch('http://workerd/workers', {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        compatibilityDate: '2023-02-28',
        serviceWorkerScript: config.type === 'classic' ? config.script : undefined,
        modules:
          config.type !== 'classic' ? [{ name: 'main', esModule: config.script }] : undefined,
        bindings: [
          { name: 'escrin', service: '@escrin/env' },
          {
            name: 'config',
            json: typeof config.config === 'string' ? config.config : JSON.stringify(config.config),
          },
        ],
      }),
    });

    const { name: id }: { name: WorkerId } = await res.json();
    return id;
  }
})();

export class Waker {
  #workerd: Fetcher;

  constructor(_state: DurableObjectState, env: { workerd: Fetcher }) {
    this.#workerd = env.workerd;
  }

  async fetch(req: Request): Promise<Response> {
    const { worker, schedule }: { worker: WorkerId; schedule: string } = await req.json();
    if (schedule !== '*/5 * * * *') return new ErrorResponse(400, 'unsupported cron spec');

    this.#notifyAndSchedule(worker, 5 * 60 * 1000).catch(() => {
      console.error('failed to notify & schedule', worker);
      // TODO: terminate or something
    });

    return new ApiResponse(204);
  }

  async #notifyAndSchedule(workerId: WorkerId, period: number): Promise<void> {
    const scheduler: { wait(delay: number): Promise<void> } = (globalThis as any).scheduler;
    while (true) {
      const res = await this.#workerd.fetch(`http://workerd/workers/${workerId}/tasks`);
      if (!res.ok) throw new Error('unable to post scheduled tasks notifications');
      await scheduler.wait(period);
    }
  }
}

class ApiResponse extends Response {
  constructor(status?: number, content?: object) {
    if (status === 204) {
      super('', { status });
      return;
    }
    super(JSON.stringify(content), {
      status,
      headers: {
        'content-type': 'application/json',
      },
    });
  }
}

class ErrorResponse extends ApiResponse {
  constructor(status: number, message: string) {
    super(status, {
      error: message,
    });
  }
}
