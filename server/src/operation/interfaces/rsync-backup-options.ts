import { Options, BackupProgression } from './options';

export class BackupContext extends BackupProgression {
  constructor(public sharePath: string) {
    super();
  }
}

export interface BackupOptions extends Options {
  includes: Array<string>;
  excludes: Array<string>;
  pathPrefix?: string;
  timeout?: number;
}
