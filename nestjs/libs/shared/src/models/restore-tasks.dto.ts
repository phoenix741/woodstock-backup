import { Field, ObjectType, registerEnumType } from '@nestjs/graphql';
import { BackupProgression } from './backup-tasks.dto';
import { JsRestoreErrorState, JsRestoreExecutionState, JsRestoreState } from '@woodstock/shared-rs';

// --- ENUMS ---

registerEnumType(JsRestoreExecutionState, { name: 'RestoreExecutionState' });
registerEnumType(JsRestoreErrorState, { name: 'RestoreErrorState' });

// --- OBJECTS ---

@ObjectType()
export class RestoreTaskState implements JsRestoreState {
  @Field(() => JsRestoreExecutionState)
  executionState: JsRestoreExecutionState;

  @Field(() => BackupProgression)
  globalProgression: BackupProgression;

  @Field(() => JsRestoreErrorState, { nullable: true })
  errorState?: JsRestoreErrorState;

  @Field({ nullable: true })
  errorMessage?: string;
}
