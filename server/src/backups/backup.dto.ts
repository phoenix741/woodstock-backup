import { ObjectType } from '@nestjs/graphql';

@ObjectType()
export class Backup {
  number!: number;
  complete!: boolean;

  startDate!: Date;
  endDate?: Date;

  fileCount!: number;
  newFileCount!: number;
  existingFileCount!: number;

  fileSize!: number;
  existingFileSize!: number;
  newFileSize!: number;

  speed!: number;
}
