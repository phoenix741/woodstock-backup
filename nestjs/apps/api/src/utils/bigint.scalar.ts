import { CustomScalar, Scalar } from '@nestjs/graphql';
import { Kind, ValueNode } from 'graphql';

@Scalar('BigInt', () => BigInt)
export class BigIntScalar implements CustomScalar<string, bigint> {
  description = 'Date custom scalar type';

  parseValue(value: string): bigint {
    return BigInt(value); // value from the client
  }

  serialize(value: bigint): string {
    return value.toString(); // value sent to the client
  }

  parseLiteral(ast: ValueNode): bigint | null {
    if (ast.kind === Kind.STRING) {
      return BigInt(ast.value);
    }
    return null;
  }
}
