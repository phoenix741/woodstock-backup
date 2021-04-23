import { createReadStream, createWriteStream } from 'fs';
import * as mkdirp from 'mkdirp';
import { cpuUsage } from 'node:process';
import { dirname } from 'path';
import { Type, Writer } from 'protobufjs';
import { from, Observable, of } from 'rxjs';
import { concatMap, mapTo, tap } from 'rxjs/operators';
import { Writable } from 'stream';

import { ProtobufMessageReader, ProtobufMessageWithPosition } from './protobuf-message-reader.utils';

const WRITE_BUFFER_SIZE = Math.pow(2, 16);

export function readAllMessages<T>(path: string, type: Type): Observable<ProtobufMessageWithPosition<T>> {
  return new Observable((subscribe) => {
    const reader = createReadStream(path);
    const transform = new ProtobufMessageReader(type);

    transform.on('data', (message: ProtobufMessageWithPosition<T>) => subscribe.next(message));
    transform.on('end', () => subscribe.complete());
    transform.on('error', (err) => subscribe.error(err));
    reader.on('error', (err) => subscribe.error(err));

    reader.pipe(transform);
  });
}

export function writeAllMessages<O, T>(path: () => string, mapping: (v: O) => T | undefined, type: Type) {
  return function (source: Observable<O>): Observable<O> {
    return new Observable((subscriber) => {
      const grpcWriter = new Writer();
      let waiting = false;
      let stream: Writable;

      function write() {
        const chunk = grpcWriter.finish();
        grpcWriter.reset();

        waiting = stream.write(chunk);
        if (waiting) {
          stream.once('drain', () => {
            write();
            waiting = false;
          });
        }
      }
      const subscription = source
        .pipe(
          concatMap((value, index) => {
            if (index === 0) {
              const p = path();
              return from(mkdirp(dirname(p))).pipe(
                tap(() => {
                  stream = createWriteStream(p, { flags: 'a' });
                  stream.on('finish', () => subscriber.complete());
                  stream.on('error', (err) => subscriber.error(err));
                }),
                mapTo(value),
              );
            }
            return of(value);
          }),
        )
        .subscribe({
          next(message) {
            const toEncodeMessage = mapping(message);
            if (toEncodeMessage) {
              try {
                type.encodeDelimited(toEncodeMessage, grpcWriter);

                if (grpcWriter.len > WRITE_BUFFER_SIZE && !waiting) {
                  write();
                }

                subscriber.next(message);
              } catch (err) {
                subscriber.error(err);
              }
            } else {
              subscriber.next(message);
            }
          },
          error(err) {
            if (!!grpcWriter.len && stream) {
              if (waiting) {
                stream.once('drain', () => stream.end(grpcWriter.finish()));
              } else {
                stream.end(grpcWriter.finish());
              }
            }
            subscriber.error(err);
          },
          complete() {
            if (!!grpcWriter.len && stream) {
              if (waiting) {
                stream.once('drain', () => stream.end(grpcWriter.finish()));
              } else {
                stream.end(grpcWriter.finish());
              }
            }
          },
        });

      return () => subscription.unsubscribe();
    });
  };
}
