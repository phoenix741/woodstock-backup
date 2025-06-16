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

  const { mutate: checkAndFixPool } = useMutation(
    graphql(/* GraphQL */ `
      mutation checkAndFixPool($fix: Boolean!, $verifyChunks: Boolean!) {
        checkAndFixPool(fix: $fix, verifyChunks: $verifyChunks) {
          ...JobPoolResponse
        }
      }
    `),
  );

  return { cleanupPool, fsckPool: checkAndFixPool };
}
