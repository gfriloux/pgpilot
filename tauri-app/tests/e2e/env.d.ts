// Minimal Node.js ambient declarations for playwright.config.ts and test helpers.
// Keeps @types/node out of devDependencies while still type-checking the config.

declare const process: {
  env: Record<string, string | undefined>;
};

declare module 'node:fs' {
  export function existsSync(path: string): boolean;
}
