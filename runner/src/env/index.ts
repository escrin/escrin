export * from './km';

type HandlerModules = Readonly<Record<string, Module>>;

export type Module = Readonly<Record<string, Handler>>;

export type Handler<T extends object | undefined | null = object> = (...args: any[]) => Promise<T>;

export class Cacheable<T> {
  constructor(public readonly item: T, public readonly expiry: Date) {}
}

export class Enviroment {
  public static builder(): EnvironmentBuilder {
    return new EnvironmentBuilder();
  }

  constructor(private readonly modules: HandlerModules) {
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

export class EnvironmentBuilder {
  private readonly modules: Writable<HandlerModules> = {};

  private error: Error | undefined;

  constructor() {}

  public addModule(name: string, handlers: Record<string, Handler>): typeof this {
    if (this.error) return this;
    if (this.modules.hasOwnProperty(name)) {
      this.error = new Error(`module named ${name} already defined`);
    }
    this.modules[name] = Object.freeze(handlers);
    return this;
  }

  public build(): HandlerModules {
    if (this.error) throw this.error;
    return Object.freeze(this.modules);
  }
}

type Writable<T> = {
  -readonly [P in keyof T]: T[P];
};

export class RpcError extends Error {
  constructor(public readonly status: number, message: string) {
    super(`RPC failed with status ${status}: ${message}`);
    this.name = new.target.name;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}
