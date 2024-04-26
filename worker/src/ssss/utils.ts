import { Address, Hex, PrivateKeyAccount, keccak256, zeroHash } from 'viem';

export type QuorumRequestParams = {
  url: string;
  method: Method;
  headers?: Record<string, string>;
  body?: string | Uint8Array;
};

type Method = 'GET' | 'POST' | 'DELETE';

type Awaitable<T> = T | Promise<T>;

export async function fetchAll<Upstream, Result>(
  upstreams: Upstream[],
  makeReqParams: (upstream: Upstream) => Awaitable<QuorumRequestParams>,
  decodeResponse: (res: Response) => Awaitable<Result>,
  timeout = 60,
): Promise<Result[]> {
  const abort = new AbortController();
  // TODO: build in retrying as long as abort signal hasn't been raised
  async function doFetch(upstream: Upstream): Promise<Result> {
    const { url, headers, body } = await makeReqParams(upstream);
    const res = await fetch(url, {
      headers,
      body: body ? JSON.stringify(body) : undefined,
      signal: abort.signal,
    });
    return decodeResponse(res);
  }

  const fetchTimeout = setTimeout(() => abort.abort(), timeout * 1000);
  const fetchResults = await Promise.allSettled(upstreams.map((upstream) => doFetch(upstream)));
  clearTimeout(fetchTimeout);

  const results = [];
  for (const res of fetchResults) {
    if (res.status === 'fulfilled') results.push(res.value);
  }

  return results;
}

export async function escrin1(
  account: PrivateKeyAccount,
  method: Method,
  url: string,
  body?: string | Uint8Array,
): Promise<{ requester: Address; signature: Hex }> {
  const signature = await account.signTypedData({
    domain: {
      name: 'SsssRequest',
      version: '1',
      chainId: 0,
      verifyingContract: '0x0000000000000000000000000000000000000000',
    },
    types: {
      SsssRequest: [
        { name: 'method', type: 'string' },
        { name: 'url', type: 'string' },
        { name: 'body', type: 'bytes32' },
      ],
    },
    primaryType: 'SsssRequest',
    message: {
      method,
      url: url.replace(/^https?:\/\//, ''),
      body: body
        ? keccak256(typeof body === 'string' ? new TextEncoder().encode(body) : body)
        : zeroHash,
    },
  });
  return { requester: account.address, signature };
}

export async function throwErrorResponse(res: Response): Promise<never> {
  let errMsg = await res.text();
  try {
    errMsg = JSON.parse(errMsg).error ?? errMsg;
  } catch {}
  throw new Error(`${errMsg} (${res.status})`);
}
