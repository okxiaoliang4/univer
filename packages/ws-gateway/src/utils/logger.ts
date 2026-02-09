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

import process from 'node:process';
import pino from 'pino';

/**
 * Logger using Pino
 *
 * Features:
 * - High-performance structured logging (JSON output)
 * - Pretty-printed output in development (via pino-pretty)
 * - Configurable log level via LOG_LEVEL env var
 * - Compatible with log aggregation tools (ELK, Loki, etc.)
 */

const logLevel = (process.env.LOG_LEVEL || 'info') as pino.Level;
const isDevelopment = process.env.NODE_ENV !== 'production';

const pinoLogger = pino({
    level: logLevel,
    // Pretty print in development, JSON in production
    transport: isDevelopment
        ? {
            target: 'pino-pretty',
            options: {
                colorize: true,
                translateTime: 'SYS:standard',
                ignore: 'pid,hostname',
            },
        }
        : undefined,
    // Base log properties
    base: {
        service: 'ws-gateway',
    },
});

/**
 * Logger adapter to maintain compatibility with existing code
 */
export const logger = {
    debug(msg: string, ...args: unknown[]) {
        if (args.length === 0) {
            pinoLogger.debug(msg);
        } else if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null) {
            // If single object arg, treat as context
            pinoLogger.debug(args[0], msg);
        } else {
            // Multiple args or primitive values
            pinoLogger.debug({ args }, msg);
        }
    },
    info(msg: string, ...args: unknown[]) {
        if (args.length === 0) {
            pinoLogger.info(msg);
        } else if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null) {
            pinoLogger.info(args[0], msg);
        } else {
            pinoLogger.info({ args }, msg);
        }
    },
    warn(msg: string, ...args: unknown[]) {
        if (args.length === 0) {
            pinoLogger.warn(msg);
        } else if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null) {
            pinoLogger.warn(args[0], msg);
        } else {
            pinoLogger.warn({ args }, msg);
        }
    },
    error(msg: string, ...args: unknown[]) {
        if (args.length === 0) {
            pinoLogger.error(msg);
        } else if (args.length === 1 && typeof args[0] === 'object' && args[0] !== null) {
            pinoLogger.error(args[0], msg);
        } else {
            pinoLogger.error({ args }, msg);
        }
    },
};
