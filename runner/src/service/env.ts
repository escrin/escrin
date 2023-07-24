import { ExecutionContext, Request } from '@cloudflare/workers-types/experimental';

export default new (class {
  async fetch(req: Request, env: { gasKey?: string }, _ctx: ExecutionContext) {
  }
})();

    // const rnr: EscrinRunner = {
    //   async getConfig() {
    //     const handler = svc?.env.get('config', 'getUserConfig');
    //     return handler ? handler() : {};
    //   },
    //   async getOmniKey(store) {
    //     const handler = svc?.env.get(store, 'getKey'); // TODO: type
    //     if (!handler) throw new Error(`unrecognized key store: ${store}`);
    //     let keyBytes = await handler('omni');
    //     if (keyBytes.item) {
    //       keyBytes = keyBytes.item;
    //     }
    //     if (!keyBytes) throw new Error(`unable to fetch omnikey from ${store}`);
    //     return keyBytes;
    //   },
    //   // async getEthProvider(network) {
    //   //   const handler = env.get(network, 'get-provider'); // TODO: type
    //   //   return handler ? handler() : undefined;
    //   // },
    // };
  // }
