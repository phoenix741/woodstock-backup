export class CommandCheck {
  command!: string;
  isValid!: boolean;
  error?: string;
}

export type CommandCheckFn = () => Promise<CommandCheck>;
export class ServerChecks {
  commands: CommandCheckFn[] = [];

  push(...cmd: CommandCheckFn[]): void {
    this.commands.push(...cmd);
  }
}
