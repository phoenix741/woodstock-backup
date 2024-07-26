import * as EventEmitter from 'events';
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
EventEmitter.defaultMaxListeners = 50; // FIXME: NbBackup concurency * NbChunk concurency
