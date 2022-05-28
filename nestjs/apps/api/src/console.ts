import { ApplicationLogger } from '@woodstock/shared';
import { BootstrapConsole } from 'nestjs-console';
import { AppCommandModule } from './app-command.module';

const bootstrap = new BootstrapConsole({
  module: AppCommandModule,
  useDecorators: true,
  contextOptions: {
    logger: new ApplicationLogger('console'),
  },
});
bootstrap.init().then(async (app) => {
  try {
    // init your app
    await app.init();
    // boot the cli
    await bootstrap.boot();
    process.exit(0);
  } catch (e) {
    process.exit(1);
  }
});
