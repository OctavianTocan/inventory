import * as Effect from 'effect/Effect';
import * as RedactedValue from 'effect/Redacted';

export const resources = [];
export const Stage = Effect.succeed(process.env.STACK_FIXTURE_STAGE ?? 'staging');

/** Capture a stack without initializing cloud providers.
 * @param {string} name Stack identity.
 * @param {object} options Provider and state configuration.
 * @param {object} program Real Effect declaration program.
 * @returns {object} Executable fixture stack.
 */
export function Stack(name, options, program) { return { name, options, program }; }

/** Supply a local-state sentinel.
 * @returns {object} Private local-state marker.
 */
export function localState() { return { kind: 'local-state' }; }

/** Supply no live providers.
 * @returns {Array} Empty provider list.
 */
export function providers() { return []; }

// Record actual declaration inputs while retaining Effect's generator behavior.
function record(kind, id, props, output) {
  const resource = { kind, id, props, output };
  resources.push(resource);
  return resource;
}

export const R2 = {
  Bucket(id, props) { return Effect.succeed(record('bucket', id, props, { bucketName: props.name }).output); },
};

/** Record a Worker declaration and return its binding reference.
 * @param {string} id Logical resource identity.
 * @param {object} props Actual Worker properties.
 * @returns {object} Effect producing a Worker reference.
 */
export function Worker(id, props) { return Effect.succeed(record('worker', id, props, { workerName: props.name }).output); }

/** Record a container binding without building an image.
 * @param {string} id Logical resource identity.
 * @param {object} props Actual container properties.
 * @returns {object} Opaque container binding.
 */
export function Container(id, props) { return record('container', id, props); }

/** Record the raw Rust Durable Object mapping.
 * @param {string} id Logical namespace identity.
 * @param {object} props Actual namespace properties.
 * @returns {object} Opaque namespace binding.
 */
export function DurableObject(id, props) { return record('namespace', id, props); }

/** Supply synthetic redacted configuration without reading process secrets.
 * @param {string} name Requested configuration name.
 * @returns {object} Effect producing a synthetic redacted value.
 */
export function Redacted(name) { return Effect.succeed(RedactedValue.make(`synthetic-only-${name}`)); }
