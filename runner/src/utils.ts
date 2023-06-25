const ENCODING = 'base64url';

export const encode = (b: Uint8Array) => Buffer.from(b).toString(ENCODING);
export const decode = (s: string) => Buffer.from(s, ENCODING);

export function memoizeAsync<T>(fn: (...args: any[]) => Promise<T>): typeof fn {
  const cache = new Map();
  const inProgress = new Map();
  return async function memoized(...args: any[]) {
    const key = JSON.stringify(args);
    if (cache.has(key)) return cache.get(key);
    if (inProgress.has(key)) return await inProgress.get(key);
    const promise = fn(...args);
    inProgress.set(key, promise);
    try {
      const result = await promise;
      cache.set(key, result);
      return result;
    } catch (err) {
      throw err;
    } finally {
      inProgress.delete(key);
    }
  };
}

export function deepFreeze<T extends object>(obj: T): Readonly<T> {
  Object.freeze(obj);

  Object.getOwnPropertyNames(obj).forEach((prop) => {
    const propValue = Reflect.get(obj, prop);
    if (
      propValue !== null &&
      (typeof propValue === 'object' || typeof propValue === 'function') &&
      !Object.isFrozen(propValue)
    ) {
      deepFreeze(propValue);
    }
  });

  return obj;
}
