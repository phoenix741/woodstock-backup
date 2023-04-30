import { TransformFnParams, TransformationType } from 'class-transformer';

export function bigIntTransformation({ value, type }: TransformFnParams) {
  switch (type) {
    case TransformationType.PLAIN_TO_CLASS:
      return value && BigInt(value);
    case TransformationType.CLASS_TO_PLAIN:
      return value?.toString();
    case TransformationType.CLASS_TO_CLASS:
      return value;
  }
}
