import canonicalize from 'canonicalize';

import { Cacheable, Environment } from './env';

type CacheEntry = {
  value: object;
  expiry: Date;
};

export class Service {
  public env: Environment = new Environment({});

  private timer: ReturnType<typeof setInterval> | undefined;

  private responseCache: Map<string, Map<string, CacheEntry>> = new Map();

  private isTerminated = false;

  constructor(public readonly worker: Worker) {
    worker.onmessage = (req) => {
      this.handleRequest(req).catch((e: any) => {
        console.log('error handling request:', JSON.stringify(e));
        this.terminate();
      });
    };
    worker.onerror = (e) => {
      console.error('worker encountered an error:', JSON.stringify(e));
      this.terminate();
    };
  }

  public get terminated() {
    return this.isTerminated;
  }

  public async handleRequest({ data: req }: MessageEvent): Promise<void> {
    if (!req || typeof req === 'object') return this.sendError(null, 'malformed RPC body');
    if (!('id' in req)) return this.sendError(null, 'malformed RPC body: missing ID');
    if (!('module' in req)) return this.sendError(req.id, 'malformed RPC body: missing module');
    if (!('method' in req)) return this.sendError(req.id, 'malformed RPC body: missing method');

    const getCacheKey = () => canonicalize(req.args) ?? '';

    const moduleResponseCache = this.responseCache.get(req.module);
    if (moduleResponseCache) {
      const cacheKey = getCacheKey();
      const { value, expiry } = moduleResponseCache.get(cacheKey) ?? {};
      if (expiry) {
        if (expiry > new Date()) {
          this.sendResponse(req.id, value);
          return;
        } else {
          moduleResponseCache.delete(cacheKey);
        }
      }
    }

    const handler = this.env.get(req.module, req.method);
    if (!handler) return this.sendError(req.id, 'unable to fulfil RPC: no such handler');
    try {
      let result = await handler(...req.args);
      if (result instanceof Cacheable) {
        if (!this.responseCache.has(req.module)) {
          this.responseCache.set(req.module, new Map());
        }
        this.responseCache.get(req.module)!.set(getCacheKey(), result.item);
        result = result.item;
      } else {
        this.sendResponse(req.id, result);
      }
    } catch (e: any) {
      this.sendError(req.id, e);
    }
  }

  public notify(): void {
    this.worker.postMessage({
      method: 'tasks',
    });
  }

  public terminate(): void {
    this.isTerminated = true;
    this.worker.terminate();
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = undefined;
    }
  }

  public schedule(period: number) {
    if (this.isTerminated) throw new Error('Worker has already terminated.');
    this.timer = setInterval(() => {
      console.log('notifying');
      this.notify();
    }, period);
  }

  private sendError(requestId: number | null, e: Error | string): void {
    this.worker.postMessage({
      type: 'error',
      requestId,
      error: typeof e === 'string' ? new Error(e) : e,
    });
  }

  private sendResponse(requestId: number | null, response?: object): void {
    this.worker.postMessage({
      type: 'response',
      requestId,
      response,
    });
  }
}
