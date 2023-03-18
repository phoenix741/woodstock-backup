import { useMutation } from '@vue/apollo-composable';
import { CleanupPoolDocument, FsckPoolDocument, VerifyChecksumDocument } from '@/generated/graphql';

export function usePool() {
  const { mutate: cleanupPool } = useMutation(CleanupPoolDocument);
  const { mutate: fsckPool } = useMutation(FsckPoolDocument);
  const { mutate: verifyChecksum } = useMutation(VerifyChecksumDocument);

  return { cleanupPool, fsckPool, verifyChecksum };
}
