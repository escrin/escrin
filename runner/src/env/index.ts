export type Module = Readonly<Record<string, Handler>>;

export type Handler<T extends object | undefined | null = object> = (...args: any[]) => Promise<T>;

export class Cacheable<T> {
  constructor(public readonly item: T, public readonly expiry: Date) {}
}

export class Environment {
  constructor(private readonly modules: Readonly<Record<string, Module>>) {
    this.modules = Object.freeze(
      Object.fromEntries(
        Object.entries(modules).map(([k, v]) => {
          return [k, Object.freeze(v)];
        }),
      ),
    );
  }

  public get(module: string, method: string): Handler | undefined {
    if (!this.modules.hasOwnProperty(module)) return undefined;
    if (!this.modules[module].hasOwnProperty(method)) return undefined;
    return this.modules[module][method];
  }
}

export class RpcError extends Error {
  constructor(public readonly status: number, message: string) {
    super(`RPC failed with status ${status}: ${message}`);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}
