import escrinWorker, * as escrin from '../../src/index.js';

export default escrinWorker({
  async tasks(rnr: escrin.Runner): Promise<void> {
    const { document } = await rnr.getAttestation({
      network: {
        chainId: 1337,
      },
      identity: {
        registry: '0xf9301d1aca7c9f4347f5cf4fa53824f6d4a42d7b',
        id: '0x949aa518c2327bd6dd30847cd1baa2317fa467b121965f25c58d8cd106a1443f',
      },
    });
    console.log(document);
  },
});
