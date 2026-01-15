import createConfig from '@univerjs-infra/shared/vitest';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';

export default createConfig({
    plugins: [
        wasm(),
        topLevelAwait(),
    ],
    optimizeDeps: {
        exclude: ['@univerjs/univer-ot-wasm'],
    },
    test: {
        deps: {
            inline: ['@univerjs/univer-ot-wasm'],
        },
    },
});
