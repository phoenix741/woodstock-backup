import { ApolloClient, createHttpLink, InMemoryCache, split } from '@apollo/client/core';
import { onError } from '@apollo/client/link/error';
import { WebSocketLink } from '@apollo/client/link/ws';
import { DefaultApolloClient } from '@vue/apollo-composable';
import { getMainDefinition, mergeDeep } from '@apollo/client/utilities';
import generatedIntrospection from '@/generated/introspection.json';
import { useAuth } from '@/composables/useAuth';

// Types
import type { App } from 'vue';
import { scalarTypePolicies } from '@/generated/graphql';

// HTTP connection to the API — `credentials: 'include'` sends the session cookie set by
// the OIDC login flow (see docs/developer_guide/AUTHENTICATION.md); a no-op when
// authentication is disabled.
const httpLink = createHttpLink({
  // You should use an absolute URL here
  uri: `http://${location.host}/graphql`,
  credentials: 'include',
});

// Kicked off here, as early as possible (this module loads before any GraphQL query can
// fire), so `enabled`/`isAdmin` are populated — or at least in flight — by the time
// anything needs them, rather than waiting on whichever component happens to mount first.
void useAuth().ensureLoaded();

// A GraphQL request rejected with 401 means the session cookie is missing/expired —
// send the browser through the login flow rather than surfacing a confusing GraphQL
// error. `redirectToLogin` is itself a no-op when authentication is disabled, and awaits
// the initial auth-state load itself so it never skips the redirect just because that load
// hasn't resolved yet.
const authErrorLink = onError(({ networkError }) => {
  if (networkError && 'statusCode' in networkError && networkError.statusCode === 401) {
    void useAuth().redirectToLogin();
  }
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
const link = authErrorLink.concat(
  split(
    // split based on operation type
    ({ query }) => {
      const definition = getMainDefinition(query);
      return definition.kind === 'OperationDefinition' && definition.operation === 'subscription';
    },
    wsLink,
    httpLink,
  ),
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
