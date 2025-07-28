import { ApolloClient, createHttpLink, InMemoryCache, split } from '@apollo/client/core';
import { WebSocketLink } from '@apollo/client/link/ws';
import { DefaultApolloClient } from '@vue/apollo-composable';
import { getMainDefinition, mergeDeep } from '@apollo/client/utilities';
import generatedIntrospection from '@/generated/introspection.json';

// Types
import type { App } from 'vue';
import { scalarTypePolicies } from '@/generated/graphql';

// HTTP connection to the API
const httpLink = createHttpLink({
  // You should use an absolute URL here
  uri: `http://${location.host}/graphql`,
});

const wsLink = new WebSocketLink({
  uri: `ws://${location.host}/graphql/ws`,
  options: {
    // Do not open the connection until a subscription is active
    lazy: true,
    reconnect: true,
    // Retry limit with exponential backoff capped at 30s
    reconnectionAttempts: 10,
    timeout: 30000,
    connectionCallback: (error) => {
      if (error) console.warn('[WS] Connection failed:', error);
    },
  },
});

// using the ability to split links, you can send data to each link
// depending on what kind of operation is being sent
const link = split(
  // split based on operation type
  ({ query }) => {
    const definition = getMainDefinition(query);
    return definition.kind === 'OperationDefinition' && definition.operation === 'subscription';
  },
  wsLink,
  httpLink,
);

// Cache implementation
const cache = new InMemoryCache({
  possibleTypes: generatedIntrospection.possibleTypes,
  typePolicies: mergeDeep(
    {
      BigIntTimeSerie: {
        keyFields: ['time'],
      },
      NumberTimeSerie: {
        keyFields: ['time'],
      },
      HostStatistics: {
        keyFields: ['host'],
      },
      Host: {
        keyFields: ['name'],
      },
      Job: {
        keyFields: ['jobId'],
      },
    },
    scalarTypePolicies,
  ),
});

// Create the apollo client
const apolloClient = new ApolloClient({
  link,
  cache,
  // Only serialize cache to DevTools in development
  connectToDevTools: import.meta.env.DEV,
});

export default {
  install(app: App) {
    app.provide(DefaultApolloClient, apolloClient);
  },
};
