import { Injectable, Logger } from '@nestjs/common';
import * as fs from 'fs';
import { rename, writeFile } from 'fs/promises';
import * as yaml from 'js-yaml';
import * as Long from 'long';
import * as mkdirp from 'mkdirp';
import { dirname } from 'path';
import { compact, tmpNameAsync } from '../utils';

// Custom BigInt type
const BigIntType = new yaml.Type('!big', {
  kind: 'scalar',
  predicate(data) {
    return typeof data == 'bigint';
  },
  construct(data) {
    return BigInt(data);
  },
  represent(data) {
    return data.toString();
  },
});

// Custom Long.js type
const LongType = new yaml.Type('!long', {
  kind: 'scalar',
  predicate(data) {
    return data instanceof Long;
  },
  construct(data) {
    return Long.fromString(data);
  },
  represent(data) {
    return data.toString();
  },
});

export const YAML_SCHEMA = yaml.DEFAULT_SCHEMA.extend([BigIntType, LongType]);

@Injectable()
export class YamlService {
  private logger = new Logger(YamlService.name);

  /**
   * Load the content of the file in yaml format
   */
  static loadFileSync<T>(filename: string, defaultValue: T): T {
    try {
      const hostsFromStr = fs.readFileSync(filename, 'utf8');
      const value = yaml.load(hostsFromStr, { schema: YAML_SCHEMA }) as never as T | undefined;
      return value || defaultValue;
    } catch (err) {
      if (defaultValue) {
        return defaultValue;
      } else {
        throw err;
      }
    }
  }

  /**
   * Load the content of the file in yaml format
   */
  async loadFile<T>(filename: string, defaultValue: T): Promise<T> {
    this.logger.verbose(`Read the file ${filename}`);

    try {
      await mkdirp(dirname(filename));

      const hostsFromStr = await fs.promises.readFile(filename, 'utf8');
      const value = yaml.load(hostsFromStr, { schema: YAML_SCHEMA }) as never as T | undefined;
      return value || defaultValue;
    } catch (err) {
      if (err instanceof yaml.YAMLException || !defaultValue) {
        this.logger.error(`Can't read the file ${filename}: ${err instanceof Error ? err.message : err}`, err);
        throw err;
      } else {
        this.logger.debug(`Can't read the file ${filename}: ${err instanceof Error ? err.message : err}`);
        return defaultValue;
      }
    }
  }

  /**
   * Save the object in the file with the filename
   */
  async writeFile<T>(filename: string, obj: T): Promise<void> {
    this.logger.verbose(`Write the file ${filename} with ${JSON.stringify(obj)}`);
    await mkdirp(dirname(filename));

    const hostsFromStr = yaml.dump(compact(obj), { schema: YAML_SCHEMA });
    const tmpFilename = await tmpNameAsync({
      tmpdir: dirname(filename),
    });

    await writeFile(tmpFilename, hostsFromStr, 'utf-8');
    await rename(tmpFilename, filename);
  }

  /**
   * Save the object in the buffer
   */
  async writeBuffer<T>(obj: T): Promise<string> {
    this.logger.verbose(`Write the buffer with ${JSON.stringify(obj)}`);

    return yaml.dump(compact(obj), { schema: YAML_SCHEMA });
  }
}
