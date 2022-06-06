import { ApplicationLogger } from '@woodstock/shared';
import { BootstrapConsole } from 'nestjs-console';
import { AppCommandModule } from './app-command.module';

const bootstrap = new BootstrapConsole({
  module: AppCommandModule,
  useDecorators: true,
  contextOptions: {
    logger: new ApplicationLogger('console', true),
  },
});
bootstrap.init().then(async (app) => {
  try {
    await app.init();
    await bootstrap.boot();
    await app.close();
    process.exit(0);
  } catch (e) {
    console.error(e);
    await app.close();
    process.exit(1);
  }
});
