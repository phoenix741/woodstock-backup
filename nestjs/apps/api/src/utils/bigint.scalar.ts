import { CustomScalar, Scalar } from '@nestjs/graphql';
import { Kind, ValueNode } from 'graphql';

@Scalar('BigInt', () => BigInt)
export class BigIntScalar implements CustomScalar<string, bigint> {
  description =
    'The `BigInt` scalar type represents non-fractional signed whole numeric values. BigInt can represent values between -(2^63) + 1 and 2^63 - 1.';

  parseValue(value: number | bigint | string): bigint {
    if (value === '') {
      throw new TypeError('The value cannot be converted from BigInt because it is empty string');
    }

    if (typeof value !== 'number' && typeof value !== 'bigint') {
      throw new TypeError(`The value ${value} cannot be converted to a BigInt because it is not an integer`);
    }

    return BigInt(value); // value from the client
  }

  serialize(value: bigint | string): string {
    if (value === '') {
      throw new TypeError('The value cannot be converted from BigInt because it is empty string');
    }

    try {
      return BigInt(value.toString()).toString();
    } catch {
      throw new TypeError(`The value ${value} cannot be converted to a BigInt because it is not an integer`);
    }
  }

  parseLiteral(ast: ValueNode): bigint {
    if (ast.kind === Kind.INT) {
      return BigInt(ast.value);
    } else {
      throw new TypeError(`BigInt cannot represent non-integer value: ${JSON.stringify(ast)}`);
    }
  }
}
