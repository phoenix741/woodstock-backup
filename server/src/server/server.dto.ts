import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class CommandCheck {
  command!: string;
  isValid!: boolean;
  error?: string;
}

@ObjectType()
export class ServerChecks {
  commands: CommandCheck[] = [];

  push(...cmd: CommandCheck[]) {
    this.commands.push(...cmd);
  }
}
