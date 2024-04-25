import escrinWorker from '../../src/index.js';

export default escrinWorker({
  async tasks(): Promise<void> {
    const res = await fetch('https://example.org');
    if (!res.ok) throw new Error('fetch failed');
    console.log(await res.text());
  },
});
