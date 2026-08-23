import { graphql } from '@/generated';

export const MergedApplicationEventFragment = graphql(/* GraphQL */ `
  fragment MergedApplicationEvent on MergedApplicationEvent {
    uuid
    type
    source
    startDate
    endDate
    errorMessages
    status
    information {
      __typename
      ... on EventBackupInformation {
        ...EventBackupInformation
      }
      ... on EventPoolInformation {
        ...EventPoolInformation
      }
      ... on EventPoolCleanedInformation {
        ...EventPoolCleanedInformation
      }
      ... on EventHashConversionInformation {
        ...EventHashConversionInformation
      }
    }
  }
`);

export const EventBackupInformationFragment = graphql(/* GraphQL */ `
  fragment EventBackupInformation on EventBackupInformation {
    hostname
    number
    sharePath
    backupId
  }
`);

export const EventPoolInformationFragment = graphql(/* GraphQL */ `
  fragment EventPoolInformation on EventPoolInformation {
    fix
    refcount
    refcountError
    inUnused
    inRefcnt
    inNothing
    missing
    chunkCount
    chunkError
  }
`);

export const EventPoolCleanedInformationFragment = graphql(/* GraphQL */ `
  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {
    size
    count
    removedHashes
  }
`);

export const EventHashConversionInformationFragment = graphql(/* GraphQL */ `
  fragment EventHashConversionInformation on EventHashConversionInformation {
    count
    algorithm
  }
`);
