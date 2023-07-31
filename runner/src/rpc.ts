import {
  ExportedHandler,
  Fetcher,
  Request,
  Response,
} from '@cloudflare/workers-types/experimental';

export class ApiResponse extends Response {
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

export class ErrorResponse extends ApiResponse {
  constructor(status: number, message: string) {
    super(status, {
      error: message,
    });
  }
}

type FetchHandler<Env> = Exclude<ExportedHandler<Env>['fetch'], undefined>;
export function wrapFetch<Env>(
  fetchHandler: (...args: Parameters<FetchHandler<Env>>) => unknown,
): FetchHandler<Env> {
  return async (req, env, ctx) => {
    try {
      const result = await fetchHandler(req, env, ctx);
      if (result instanceof Response) return result;
      const statusCode = result === null || result === undefined || result === '' ? 204 : 200;
      return new ApiResponse(statusCode, result ?? '');
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
): Promise<{ method: string; params: Record<string, any> }> {
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
  if (!('params' in bodyJson) || !Array.isArray(bodyJson.params))
    throw new ApiError(400, `invalid request body: missing or invalid params`);
  return {
    method: bodyJson.method,
    params: bodyJson.params,
  };
}

export function encodeBase64Bytes(bytes: Uint8Array): string {
  const binaryParts: string[] = [];
  const len = bytes.byteLength;
  for (let i = 0; i < len; i++) {
    binaryParts.push(String.fromCharCode(bytes[i]));
  }
  const binary = binaryParts.join('');
  return btoa(binary);
}

export function decodeBase64Bytes(base64: string): Uint8Array {
  const binary = atob(base64);
  let bytes = new Uint8Array(base64.length);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}

export async function rpc<ResponseType>(
  remote: Fetcher,
  method: string,
  params: Record<string, any>,
): Promise<ResponseType> {
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
  return res.json();
}

export class ApiError extends Error {
  constructor(public readonly statusCode: number, message: string) {
    super(message);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}
