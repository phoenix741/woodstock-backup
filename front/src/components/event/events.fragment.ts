import { graphql } from '@/generated';

export const ApplicationEventFragment = graphql(/* GraphQL */ `
  fragment ApplicationEvent on ApplicationEvent {
    uuid
    type
    step
    source
    timestamp
    errorMessages
    status
    information {
      __typename
      ... on EventBackupInformation {
        ...EventBackupInformation
      }
      ... on EventRefCountInformation {
        ...EventRefCountInformation
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
  }
`);

export const EventRefCountInformationFragment = graphql(/* GraphQL */ `
  fragment EventRefCountInformation on EventRefCountInformation {
    fix
    count
    error
  }
`);

export const EventPoolInformationFragment = graphql(/* GraphQL */ `
  fragment EventPoolInformation on EventPoolInformation {
    fix
    inUnused
    inRefcnt
    inNothing
    missing
  }
`);

export const EventPoolCleanedInformationFragment = graphql(/* GraphQL */ `
  fragment EventPoolCleanedInformation on EventPoolCleanedInformation {
    size
    count
  }
`);

export const EventHashConversionInformationFragment = graphql(/* GraphQL */ `
  fragment EventHashConversionInformation on EventHashConversionInformation {
    count
    algorithm
  }
`);
