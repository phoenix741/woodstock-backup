import { Field, InputType, Int } from '@nestjs/graphql';
import { IsArray, IsInt, IsNotEmpty, IsString } from 'class-validator';

@InputType()
export class RestoreFilesInput {
  @IsString()
  @IsNotEmpty()
  share!: string;

  @IsString({ each: true })
  @IsNotEmpty({ each: true })
  selection!: string[];
}

@InputType()
export class RestoreInput {
  @IsString()
  @IsNotEmpty()
  hostname!: string;

  @IsInt()
  @Field(() => Int)
  number!: number;

  @IsString()
  destinationDirectory: string;

  @IsNotEmpty()
  @IsArray()
  files: RestoreFilesInput[];
}
