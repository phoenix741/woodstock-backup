import { FileBrowserService, globStringToRegex, HostsService, longToBigInt } from '@woodstock/shared';
import * as Long from 'long';
import { Command, Console, createSpinner } from 'nestjs-console';
import { join } from 'path';

@Console({
  command: 'browser',
})
export class BrowserCommand {
  constructor(private hostService: HostsService, private browser: FileBrowserService) {}

  @Command({
    command: 'path <host> <share>',
    description: 'List all file of a local directory reading the file of the host (for testing purposes)',
    options: [
      {
        flags: '--verbose',
        required: false,
        description: 'Show all files',
      },
    ],
  })
  async browse(host: string, path: string, options: { verbose: boolean }): Promise<void> {
    const { verbose } = options;
    const spinner = createSpinner();
    spinner.start(`Get config for ${host}`);

    // Get the host configuration
    const config = await this.hostService.getHostConfiguration(host);

    spinner.text = `Get shares for ${host} - ${path}`;
    // Search the shares that matches the path
    const shares = config.operations?.operation?.shares
      .map((share) => ({
        name: share.name,
        includes: [...(share.includes || []), ...(config.operations?.operation?.includes || [])],
        excludes: [...(share.excludes || []), ...(config.operations?.operation?.excludes || [])],
      }))
      .filter((share) => path.startsWith(share.name));

    // Browser each shares
    for (const share of shares || []) {
      spinner.text = `Browse ${host} - ${path}`;
      const files = this.browser.getFiles(Buffer.from(path))(
        Buffer.from(''),
        share.includes?.map((s) => globStringToRegex(s)) || [],
        share.excludes?.map((s) => globStringToRegex(s)) || [],
      );

      let fileCount = 0;
      let fileSize = 0n;
      for await (const file of files) {
        if (verbose) {
          spinner.succeed(join(host, path, file.path.toString()));
        }
        fileCount++;
        fileSize += longToBigInt(file.stats?.size || Long.ZERO);
      }

      spinner.succeed(`${host} - ${path} (${fileCount} files), ${fileSize} bytes`);
    }

    spinner.stop();
  }
}
