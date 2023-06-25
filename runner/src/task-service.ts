import { Envs } from './envs';
import { deepFreeze } from './utils';

type HandlerFunction = (...args: any[]) => Promise<object | undefined | null>;

const DEFAULT_KEY_STORE = 'sapphire';

class RpcError extends Error {
  constructor(public readonly statusCode: number, message: string) {
    super(message);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export class TaskService {
  private timer: ReturnType<typeof setInterval> | undefined;

  private isTerminated = false;

  constructor(public readonly worker: Worker, private readonly envs: Envs) {
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
    const handler = this.envs.get(req.module, req.method);
    if (!handler) return this.sendError(req.id, 'unable to fulfil RPC: no such handler');
    try {
      this.sendResponse(req.id, handler(...req.args));
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
