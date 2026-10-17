// Node's env file parser, for the native server's differential tests.
import { parseEnv as parse } from 'node:util';

export const parseEnv = (text) => Object.fromEntries(Object.entries(parse(text)).sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
