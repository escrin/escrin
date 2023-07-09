import {
  Body,
  DurableObjectNamespace,
  DurableObjectState,
  ExecutionContext,
  Request,
} from '@cloudflare/workers-types/experimental';

import { ApiError } from '../error.js';
import { Service } from '../service.js';
import { Environment } from '../env/index.js';
import sapphire from '../env/sapphire.js';

type AgentSpec = {
  script: string;
  type?: 'classic' | 'module';
  name?: string;
  schedule?: string;
  args?: object;
  config?: string | Record<string, object>;
};

async function parseRequestBody(contentType: string | null, body: Body): Promise<AgentSpec> {
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

async function createService(spec: AgentSpec, gasKey: string): Promise<Service> {
  const scriptUrl = URL.createObjectURL(
    new Blob([spec.script], { type: 'application/javascript' }),
  );
  const worker = new Worker(scriptUrl, { type: spec.type, name: spec.name });
  URL.revokeObjectURL(scriptUrl);

  let svc: Service;
  try {
    svc = new Service(spec.name ?? 'unnamed', worker);
  } catch (e: any) {
    throw new ApiError(400, e.message ?? JSON.stringify(e));
  }
  svc.env = new Environment({
    'sapphire-mainnet': sapphire('mainnet', gasKey),
    'sapphire-testnet': sapphire('testnet', gasKey),
    config: {
      'get-config': () => spec.config,
    },
  });

  return svc;
}

type RunnerEnv = {
  spawner: DurableObjectNamespace;
};

export default new (class {
  async fetch(req: Request, env: RunnerEnv, _ctx: ExecutionContext) {
    const id = env.spawner.idFromName('default');
    return env.spawner.get(id).fetch(req);
  }
})();

class ErrorResponse extends Response {
  constructor(status: number, message: string) {
    const resp = JSON.stringify({
      message,
    });
    super(resp, {
      status,
      headers: {
        'content-type': 'application/json',
      },
    });
  }
}

type SpawnerEnv = {
  gasKey?: string;
};

export class EscrinSpawner {
  #gasKey: string;

  constructor(_state: DurableObjectState, env: SpawnerEnv) {
    if (!env.gasKey || !/^(0x)?[0-9a-f]{64,64}$/i.test(env.gasKey)) {
      throw new Error('missing or invalid `env.gasKey`');
    }
    this.#gasKey = env.gasKey;
  }

  async fetch(req: Request): Promise<Response> {
    try {
      const spec = await parseRequestBody(req.headers.get('content-type'), req);
      const service = await createService(spec, this.#gasKey);

      if (spec.schedule) {
        if (spec.schedule !== '*/5 * * * *') throw new ApiError(400, 'unsupported cron spec');
        service.schedule(5 * 60 * 1000);
      }
      service.notify();

      return new Response('', { status: 201 });
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        return new ErrorResponse(e.statusCode, e.message);
      }
      return new ErrorResponse(500, 'internal server error');
    }
  }
}
