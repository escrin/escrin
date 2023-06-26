import { Body, Request, ExecutionContext } from '@cloudflare/workers-types/experimental';

import { Environment } from './env';
import sapphire from './env/sapphire';
import { Service } from './service';

class ApiError extends Error {
  constructor(public readonly statusCode: number, message: string) {
    super(message);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

type AgentSpec = {
  script: string;
  type?: 'classic' | 'module';
  name?: string;
  schedule?: string;
  args?: object;
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
      const fd = await body.formData();
      return Object.fromEntries(fd.entries()) as any;
    } catch {
      throw new ApiError(400, 'the request body could not be parsed as form data');
    }
  }
  throw new ApiError(400, `unsupported content type: ${contentType}`);
}

export class TaskManager {
  private services: Service[] = [];

  async fetch(req: Request, env: { gasKey?: string }, _ctx: ExecutionContext) {
    if (!/^(0x)?[0-9a-f]{64,64}$/i.test(env.gasKey ?? '')) {
      console.log(env.gasKey)
      console.error('missing or invalid `env.gasKey`');
      return new Response('', { status: 500 });
    }

    let spec: AgentSpec;
    try {
      spec = await parseRequestBody(req.headers.get('content-type'), req);
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        return new ErrorResponse(e.statusCode, e.message);
      }
      throw e;
    }

    const scriptUrl = URL.createObjectURL(new Blob([spec.script], { type: 'application/json' }));
    const worker = new Worker(scriptUrl, { type: spec.type, name: spec.name });
    URL.revokeObjectURL(scriptUrl);

    let svc: Service;
    try {
      svc = new Service(worker);
    } catch (e: unknown) {
      console.error(e);
      return new Response('', { status: 400 });
    }
    svc.env = new Environment({
      'sapphire-mainnet': sapphire('mainnet', env.gasKey!),
      'sapphire-testnet': sapphire('testnet', env.gasKey!),
    });

    if (spec.schedule) {
      if (spec.schedule !== '*/5 * * * *') {
        // throw new ApiError(400, 'unsupported cron spec')
        return new Response('', { status: 501 });
      }
      console.log('schedule');
      svc.schedule(5 * 60 * 1000);
    }
    this.services.push(svc);
    return new Response('', { status: 201 });
  }
}

export default new TaskManager();

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
