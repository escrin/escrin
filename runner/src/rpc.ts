import { ApiError } from './error.js';

export async function decodeRequest(req: Request): Promise<{ method: string; params: any[] }> {
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
