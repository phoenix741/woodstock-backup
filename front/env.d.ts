/// <reference types="vite/client" />

/** Front build version, injected from package.json at build time (see vite.config.ts). */
declare const __APP_VERSION__: string;

declare module '*.md' {
  import type { ComponentOptions } from 'vue';

  const Component: ComponentOptions;
  export default Component;
}
