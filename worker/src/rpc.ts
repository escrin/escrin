import type { ExecutionContext, Fetcher, Request } from '@cloudflare/workers-types/experimental';

export class ApiResponse extends Response {
  constructor(status?: number, content?: unknown) {
    if (status === 204) {
      super(null, { status });
      return;
    }
    super(content ? JSON.stringify(content) : '{}', {
      status,
      headers: {
        'content-type': 'application/json',
      },
    });
  }
}

export class ErrorResponse extends ApiResponse {
  constructor(status: number, message: string) {
    super(status, {
      error: message,
    });
  }
}

export function wrapFetch<Env>(
  handler: (req: Request, env: Env, ctx: ExecutionContext) => unknown,
): (req: Request, env: Env, ctx: ExecutionContext) => Promise<Response> {
  return async (req: Request, env: Env, ctx: ExecutionContext) => {
    try {
      const result = await handler(req, env, ctx);
      if (result instanceof Response) return result;
      const statusCode = result === null || result === undefined || result === '' ? 204 : 200;
      return new ApiResponse(statusCode, result);
    } catch (e: any) {
      if (e instanceof ApiError) {
        return new ApiResponse(e.statusCode, { error: e.message });
      }
      throw e;
    }
  };
}

export async function decodeRequest(
  req: Request,
): Promise<{ method: string; params: Record<string, unknown> }> {
  if (req.headers.get('content-type') !== 'application/json')
    throw new ApiError(400, 'invalid content-type. application/json is required');
  let bodyJson: unknown;
  try {
    bodyJson = await req.json();
  } catch (e: any) {
    throw new ApiError(400, `unable to decode request body: ${e}`);
  }
  if (typeof bodyJson !== 'object' || bodyJson === null)
    throw new ApiError(400, `invalid request body`);
  if (!('method' in bodyJson) || typeof bodyJson.method !== 'string')
    throw new ApiError(400, `invalid request body: missing or invalid method`);
  if (!('params' in bodyJson) || typeof bodyJson.params !== 'object' || bodyJson.params === null)
    throw new ApiError(400, `invalid request body: missing or invalid params`);
  return {
    method: bodyJson.method,
    params: bodyJson.params as Record<string, unknown>,
  };
}

export type RequestType = {
  method: string;
  params: Record<string, unknown>;
  response: void | Record<string, unknown>;
};

export async function rpc<T extends RequestType>(
  remote: Fetcher,
  method: T['method'],
  params: T['params'],
): Promise<T['response']> {
  const res = await remote.fetch(`http://escrin`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      method,
      params,
    }),
  });
  if (!res.ok) {
    const resText = await res.text();
    let errorMsg = resText;
    try {
      errorMsg = JSON.parse(errorMsg).error;
    } catch {}
    throw new ApiError(res.status, errorMsg);
  }
  if (res.status === 204) return;
  return res.json();
}

export class ApiError extends Error {
  constructor(public readonly statusCode: number, message: string) {
    super(message);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}
