import { beforeAll, beforeEach, describe, expect, it } from '@jest/globals';
import { hashToCurve, secp256k1 } from '@noble/curves/secp256k1';
import { Hex, bytesToHex } from 'viem';

import { Pedersen, Share } from '../../src/ssss/vss.js';

let vss: Pedersen;

beforeAll(() => {
  vss = new Pedersen(secp256k1, hashToCurve);
});

describe('Pedersen', () => {
  const threshold = 2;
  const numShares = 3;

  let secret: Hex;
  let shares: Share[];
  let commitments: Hex[];

  beforeEach(() => {
    ({ secret, shares, commitments } = vss.generate(threshold, numShares));
  });

  it('reconstructs', () => {
    expect(vss.reconstruct(shares.slice(0, threshold), commitments)).toEqual(secret);
  });

  it('reconstructs many shares', () => {
    const n = 64;
    const m = Math.ceil((n * 2) / 3);
    ({ secret, shares, commitments } = vss.generate(m, n));
    expect(commitments.length).toBe(m);
    expect(shares.length).toBe(n);
    expect(vss.reconstruct(shares.slice(n - m), commitments)).toEqual(secret);
  });

  it('fails to reconstruct with too few shares', () => {
    expect(() => vss.reconstruct(shares.slice(0, threshold - 1), commitments)).toThrow(
      'vss: too few shares provided',
    );
  });

  it('rejects duplicate shares', () => {
    shares.push(shares[0], shares[1]);
    expect(vss.reconstruct(shares, commitments)).toEqual(secret);
  });

  it('rejects invalid share.index', () => {
    shares[shares.length - 1].index = 0;
    expect(vss.reconstruct(shares, commitments)).toEqual(secret);
  });

  it('rejects invalid share.secret', () => {
    shares[shares.length - 1].secret = bytesToHex(secp256k1.utils.randomPrivateKey());
    expect(vss.reconstruct(shares, commitments)).toEqual(secret);
  });

  it('rejects invalid share.blinder', () => {
    shares[shares.length - 1].blinder = bytesToHex(secp256k1.utils.randomPrivateKey());
    expect(vss.reconstruct(shares, commitments)).toEqual(secret);
  });

  it('fails to reconstruct with too many invalid shares', () => {
    shares[0].secret = bytesToHex(secp256k1.utils.randomPrivateKey());
    shares[1].blinder = bytesToHex(secp256k1.utils.randomPrivateKey());
    expect(() => vss.reconstruct(shares, commitments)).toThrow(
      'vss: too few valid shares to reconstruct',
    );
  });

  it('will not generate with invalid paramters', () => {
    expect(() => vss.generate(1, 1)).toThrow('vss: numShares must be at least 2');
    expect(() => vss.generate(1, 3)).toThrow('vss: threshold must be at least 2');
    expect(() => vss.generate(3, 2)).toThrow('vss: threshold cannot exceed numShares');
    expect(() => vss.generate(2, 2)).not.toThrow();
  });
});

describe('lagrangeInterpolate', () => {
  let lagrangeInterpolate: (at: bigint, xs: bigint[], ys: bigint[]) => bigint;

  beforeAll(() => {
    lagrangeInterpolate = (vss as any).lagrangeInterpolate.bind(vss);
  });

  it('degree 0 polynomial', () => {
    const xs = [1n];
    const ys = [2n];
    const at = 1n;

    expect(lagrangeInterpolate(at, xs, ys)).toBe(2n);
  });

  it('degree 1 polynomial', () => {
    const xs = [1n, 2n];
    const ys = [1n, 3n];
    const at = 1n;

    expect(lagrangeInterpolate(at, xs, ys)).toBe(1n);
  });

  it('degree 2 polynomial', () => {
    const xs = [1n, 2n, 3n];
    const ys = [1n, 4n, 9n];
    const at = 2n;

    expect(lagrangeInterpolate(at, xs, ys)).toBe(4n);
  });

  it('should interpolate the correct value', () => {
    const xs = [1n, 2n, 3n];
    const ys = [5n, 7n, 10n];
    const at = 4n;

    expect(lagrangeInterpolate(at, xs, ys)).toBe(14n);
  });

  it('should handle empty input', () => {
    const xs: bigint[] = [];
    const ys: bigint[] = [];
    const at = 4n;
    const expected = 0n;

    expect(lagrangeInterpolate(at, xs, ys)).toBe(expected);
  });
});
