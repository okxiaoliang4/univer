/**
 * Copyright 2023-present DreamNum Co., Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import path from 'node:path';

const packages: string[] = [];
for await (const pkgGlob of [
    'packages/core',
    'packages/design',
    'packages/docs-drawing-ui',
    'packages/docs-mention-ui',
    'packages/docs-quick-insert-ui',
    'packages/docs-ui',
    'packages/drawing-ui',
    'packages/drawing',
    'packages/engine-formula',
    'packages/engine-render',
    'packages/find-replace',
    'packages/keyboard-ui',
    'packages/sheets-conditional-formatting-ui',
    'packages/sheets-conditional-formatting',
    'packages/sheets-data-validation-ui',
    'packages/sheets-drawing-ui',
    'packages/sheets-drawing',
    'packages/sheets-find-replace',
    'packages/sheets-formula-ui',
    'packages/sheets-formula',
    'packages/sheets-hyper-link-ui',
    'packages/sheets-numfmt-ui',
    'packages/sheets-pivot-table-ui',
    'packages/sheets-pivot-table',
    'packages/sheets-sort-ui',
    'packages/sheets-table-ui',
    'packages/sheets-table',
    'packages/sheets-thread-comment-ui',
    'packages/sheets-ui',
    'packages/sheets',
    'packages/slides-ui',
    'packages/thread-comment-ui',
    'packages/ui',
    'packages/uniscript',
]) {
    const file = `${pkgGlob}/package.json`;
    const json = (await Bun.file(file).json()) as Record<string, unknown>;
    if (json.private === true) continue;
    path.resolve(file, '..');
    packages.push(path.resolve(file, '..'));
}

await Promise.all(
    packages.map(async (p) => {
        await Bun.$.cwd(p)`pnpm build`;
    })
);

const results = await Promise.allSettled(
    packages.map(async (p) => {
        try {
            await Bun.$.cwd(p)`pnpm unpublish -f`;
            await Bun.$.cwd(p)`pnpm publish`;
            return { status: 'success', package: p };
        } catch (error) {
            return { status: 'error', package: p, error };
        }
    })
);

console.log(results.map((result) => `${result.status === 'fulfilled' ? result.value.package : result.reason}: ${result.status}`));
