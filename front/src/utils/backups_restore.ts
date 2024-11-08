import { graphql } from '@/generated';
import { useMutation } from '@vue/apollo-composable';

export const JobPoolResponseFragmentNode = graphql(/* GraphQL */ `
  fragment JobPoolResponse on JobResponse {
    id
  }
`);

export function useBackupRestore() {
  const { mutate } = useMutation(
    graphql(/* GraphQL */ `
      mutation restoreBackup($input: RestoreInput!) {
        restoreBackup(input: $input) {
          ...JobPoolResponse
        }
      }
    `),
  );

  const restoreBackup = (
    hostname: string,
    number: number,
    share: string,
    path: string,
    destinationDirectory: string,
  ) => {
    return mutate({
      input: {
        hostname,
        number,
        files: [{ share, selection: [path] }],
        destinationDirectory,
      },
    });
  };

  return { restoreBackup };
}
