import { graphql } from '@/generated';
import { useMutation } from '@vue/apollo-composable';

export const JobPoolResponseFragmentNode = graphql(/* GraphQL */ `
  fragment JobPoolResponse on JobResponse {
    id
  }
`);

export function usePool() {
  const { mutate: cleanupPool } = useMutation(
    graphql(/* GraphQL */ `
      mutation cleanupPool {
        cleanupPool {
          ...JobPoolResponse
        }
      }
    `),
  );

  const { mutate: fsckPool } = useMutation(
    graphql(/* GraphQL */ `
      mutation fsckPool($fix: Boolean!) {
        checkAndFixPool(fix: $fix) {
          ...JobPoolResponse
        }
      }
    `),
  );

  const { mutate: verifyChecksum } = useMutation(
    graphql(/* GraphQL */ `
      mutation verifyChecksum {
        verifyChecksum {
          ...JobPoolResponse
        }
      }
    `),
  );

  return { cleanupPool, fsckPool, verifyChecksum };
}
