import * as fs from 'fs';
import * as path from 'path';
import * as yaml from 'js-yaml';
import * as mkdirp from 'mkdirp';
import { Injectable, Logger } from '@nestjs/common';
import { compact } from './lodash';
import * as tmp from 'tmp';
import * as util from 'util';

const tmpNameAsync = util.promisify((options: tmp.TmpNameOptions, cb: tmp.TmpNameCallback) => tmp.tmpName(options, cb));

@Injectable()
export class YamlService {
  private logger = new Logger(YamlService.name);

  /**
   * Load the content of the file in yaml format
   */
  static loadFileSync<T>(filename: string, defaultValue?: T): T {
    try {
      const hostsFromStr = fs.readFileSync(filename, 'utf8');
      return yaml.safeLoad(hostsFromStr) || defaultValue;
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
  async loadFile<T>(filename: string, defaultValue?: T): Promise<T> {
    this.logger.verbose(`Read the file ${filename}`);

    try {
      await mkdirp(path.dirname(filename));

      const hostsFromStr = await fs.promises.readFile(filename, 'utf8');
      return yaml.safeLoad(hostsFromStr) || defaultValue;
    } catch (err) {
      this.logger.debug(`Can't read the file ${filename}: ${err.message}`);
      if (defaultValue) {
        return defaultValue;
      } else {
        throw err;
      }
    }
  }

  /**
   * Save the object in the file with the filename
   */
  async writeFile<T>(filename: string, obj: T): Promise<void> {
    this.logger.verbose(`Write the file ${filename} with ${JSON.stringify(obj)}`);
    await mkdirp(path.dirname(filename));

    const hostsFromStr = yaml.safeDump(compact(obj));
    const tmpFilename = await tmpNameAsync({
      tmpdir: path.dirname(filename),
    } as tmp.TmpNameOptions);
    await fs.promises.writeFile(tmpFilename, hostsFromStr, 'utf-8');
    await fs.promises.rename(tmpFilename, filename);
  }
}
