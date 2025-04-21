import { Field, ObjectType, registerEnumType } from '@nestjs/graphql';
import { JsRemoveErrorState, JsRemoveExecutionState, JsRemoveState } from '@woodstock/shared-rs';

// --- ENUMS ---

registerEnumType(JsRemoveExecutionState, { name: 'RemoveExecutionState' });
registerEnumType(JsRemoveErrorState, { name: 'RemoveErrorState' });

// --- OBJECTS ---

@ObjectType()
export class RemoveTaskState implements JsRemoveState {
  @Field(() => JsRemoveExecutionState)
  executionState: JsRemoveExecutionState;

  @Field(() => JsRemoveErrorState, { nullable: true })
  errorState?: JsRemoveErrorState;

  @Field({ nullable: true })
  errorMessage?: string;
}
