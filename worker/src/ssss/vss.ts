import { H2CPoint } from '@noble/curves/abstract/hash-to-curve';
import * as mod from '@noble/curves/abstract/modular';
import type { CurveFn, ProjConstructor, ProjPointType } from '@noble/curves/abstract/weierstrass';
import { Hex, bytesToBigInt, bytesToHex, hexToBigInt, numberToHex } from 'viem';

export type Share = {
  index: number;
  secret: Hex;
  blinder: Hex;
};

type Point = ProjPointType<bigint>;

export class Pedersen {
  public static H_SEED = 'escrin-pedersen-vss-blinder-generator';

  private readonly field: mod.IField<bigint>;
  private readonly randomScalar: () => Uint8Array;
  private readonly ProjectivePoint: ProjConstructor<bigint>;
  private readonly g: Point;
  private readonly h: Point;

  constructor(curve: CurveFn, hashToCurve: (msg: Uint8Array) => H2CPoint<bigint>) {
    this.randomScalar = () => curve.utils.randomPrivateKey();
    this.ProjectivePoint = curve.ProjectivePoint;
    this.field = mod.Field(curve.CURVE.n);

    this.g = curve.ProjectivePoint.BASE;
    const h = hashToCurve(new TextEncoder().encode(Pedersen.H_SEED));
    this.h = curve.utils.precompute(8, curve.ProjectivePoint.fromAffine(h.toAffine()));
  }

  public generate(
    threshold: number,
    numShares: number,
  ): { secret: Hex; shares: Share[]; commitments: Hex[] } {
    if (numShares < 2) throw new Error('vss: numShares must be at least 2');
    if (threshold < 2) throw new Error('vss: threshold must be at least 2');
    if (threshold > numShares) throw new Error('vss: threshold cannot exceed numShares');

    const secretBytes = this.randomScalar();
    const secret = bytesToBigInt(secretBytes);
    const blinder = bytesToBigInt(this.randomScalar());

    const s = this.split(secret, threshold, numShares);
    const b = this.split(blinder, threshold, numShares);

    const commitments: Hex[] = [];
    for (let i = 0; i < threshold; i++) {
      commitments.push(`0x${this.commit(s.coeffs[i], b.coeffs[i]).toHex()}` as Hex);
    }

    const shares = [];
    for (let i = 0; i < numShares; i++) {
      shares.push({
        index: i + 1,
        secret: numberToHex(s.shares[i]),
        blinder: numberToHex(b.shares[i]),
      });
    }

    return { secret: bytesToHex(secretBytes), shares, commitments };
  }

  public reconstruct(shares: Share[], verifiers: Hex[]): Hex {
    if (verifiers.length < 2) throw new Error('vss: at least two shares are required');
    if (shares.length < verifiers.length) throw new Error('vss: too few shares provided');

    const commitments = verifiers.map((v) => this.ProjectivePoint.fromHex(v.replace(/^0x/, '')));

    const secretShares: Array<[bigint, bigint]> = [];
    const blinderShares: Array<[bigint, bigint]> = [];
    const seenIndices = new Set<number>();
    for (const s of shares) {
      try {
        if (seenIndices.has(s.index)) continue;
        const { x, si, bi } = this.verifyShare(s, commitments);
        secretShares.push([x, si]);
        blinderShares.push([x, bi]);
        seenIndices.add(s.index);
      } catch {}
    }

    if (secretShares.length < verifiers.length)
      throw new Error(`vss: too few valid shares to reconstruct`);

    return numberToHex(this.combine(secretShares));
  }

  private verifyShare(share: Share, commitments: Point[]): { x: bigint; si: bigint; bi: bigint } {
    const x = BigInt(share.index);
    let recoveredCommitment = commitments[0];
    for (let i = 1; i < commitments.length; i++) {
      recoveredCommitment = recoveredCommitment.add(
        commitments[i].multiplyUnsafe(this.field.pow(x, BigInt(i))),
      );
    }

    const si = hexToBigInt(share.secret);
    const bi = hexToBigInt(share.blinder);

    if (!recoveredCommitment.equals(this.commit(si, bi)))
      throw new Error(`vss: faulty share at x=${share.index}`);

    return { x, si, bi };
  }

  private commit(s: bigint, b: bigint): Point {
    return this.g.multiply(s).add(this.h.multiply(b));
  }

  private split(
    secret: bigint,
    threshold: number,
    numShares: number,
  ): { shares: bigint[]; coeffs: bigint[] } {
    const coeffs = [secret];
    for (let i = 0; i < threshold - 1; i++) {
      coeffs.push(bytesToBigInt(this.randomScalar()));
    }
    const shares: bigint[] = [];
    for (let i = 1; i < numShares + 1; i++) {
      const x = BigInt(i);
      shares.push(this.evaluatePolynomial(coeffs, x));
    }
    return { coeffs, shares };
  }

  private evaluatePolynomial(coeffs: bigint[], at: bigint): bigint {
    // Evaluate the polynomial using Horner's method.
    let result = 0n;
    for (let i = coeffs.length - 1; i >= 0; --i) {
      result = this.field.add(this.field.mul(result, at), coeffs[i]);
    }
    return result;
  }

  private combine(shares: Array<[bigint, bigint]>): bigint {
    const xs = [];
    const ys = [];
    for (const [x, y] of shares) {
      xs.push(x);
      ys.push(y);
    }
    return this.lagrangeInterpolate(0n, xs, ys);
  }

  private lagrangeInterpolate(at: bigint, xs: bigint[], ys: bigint[]): bigint {
    const fp = this.field;

    let result = 0n;

    for (let i = 0; i < xs.length; i++) {
      let numerator = 1n;
      let denominator = 1n;

      for (let j = 0; j < xs.length; j++) {
        if (i === j) continue;
        numerator = fp.mul(numerator, fp.sub(at, xs[j]));
        denominator = fp.mul(denominator, fp.sub(xs[i], xs[j]));
      }

      result = fp.add(result, fp.mul(fp.mul(ys[i], numerator), fp.inv(denominator)));
    }

    return result;
  }
}
