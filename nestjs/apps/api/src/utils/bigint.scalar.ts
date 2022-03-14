import { Scalar } from '@nestjs/graphql';
import ScalarBigInt from 'apollo-type-bigint';

@Scalar('BigInt', () => BigInt)
export class BigIntScalar extends ScalarBigInt {
  constructor() {
    super('bigInt');
  }
}
