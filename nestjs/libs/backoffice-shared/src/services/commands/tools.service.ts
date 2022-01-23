import { Injectable } from '@nestjs/common';
import { rendering, YamlService } from '@woodstock/shared';
import { resolve } from 'path';
import { ApplicationConfigService } from '../../config/application-config.service';

class Tools {
  tools!: Record<string, string>;
  command!: Record<string, string>;
  paths!: Record<string, string>;
}

const DEFAULT_TOOLS = YamlService.loadFileSync<Tools>(resolve('config', 'tools.yml'), {
  tools: {},
  command: {},
  paths: {},
});

export class CommandParameters {
  ip?: string;
  hostname?: string;
  destBackupNumber?: number;
  srcBackupNumber?: number;
}

@Injectable()
export class ToolsService {
  constructor(private configService: ApplicationConfigService, private yamlService: YamlService) {}

  private customTools?: Tools;

  private async loadTools(): Promise<Tools> {
    const customTools = await this.yamlService.loadFile(this.configService.configPathOfTools, DEFAULT_TOOLS);
    return {
      tools: { ...DEFAULT_TOOLS.tools, ...customTools.tools },
      command: { ...DEFAULT_TOOLS.command, ...customTools.command },
      paths: { ...DEFAULT_TOOLS.paths, ...customTools.paths },
    };
  }

  private async loadToolsOnlyOneTime(): Promise<Tools> {
    if (!this.customTools) {
      this.customTools = await this.loadTools();
    }
    return this.customTools;
  }

  async getTool(command: string): Promise<string> {
    const tools = await this.loadToolsOnlyOneTime();
    return tools.tools[command];
  }

  async getCommand(command: string, params: CommandParameters): Promise<string> {
    const tools = await this.loadToolsOnlyOneTime();
    const replacementParams = Object.assign(
      {},
      params,
      this.configService.toJSON(),
      tools.tools,
      await this.getPaths(params),
    );
    return rendering(tools.command[command], replacementParams);
  }

  async getPaths(params: CommandParameters): Promise<Record<string, string>> {
    const tools = await this.loadToolsOnlyOneTime();
    const replacementParams = Object.assign({}, params, this.configService.toJSON(), tools.tools);

    return {
      ...this.configService.toJSON(),
      ...Object.entries(tools.paths).reduce((acc, [key, value]) => {
        acc[key] = rendering(value, replacementParams);
        return acc;
      }, {} as Record<string, string>),
    };
  }

  async getPath(path: string, params: CommandParameters): Promise<string> {
    const paths = await this.getPaths(params);
    return paths[path];
  }
}
