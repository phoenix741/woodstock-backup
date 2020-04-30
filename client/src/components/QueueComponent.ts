import { DocumentNode } from 'graphql';
import Vue from 'vue';

import { Job } from '../generated/graphql';

export type DeepPartial<T> = T extends Function ? T : T extends object ? { [P in keyof T]?: DeepPartial<T[P]> } : T;

export type Query = {
  queue: {
    all?: Array<DeepPartial<Job>>;
    active?: Array<DeepPartial<Job>>;
    failed?: Array<DeepPartial<Job>>;
    delayed?: Array<DeepPartial<Job>>;
    waiting?: Array<DeepPartial<Job>>;
    completed?: Array<DeepPartial<Job>>;
  };
};

export type Subscription = {
  jobUpdated: DeepPartial<Job>;
};

export const QueueComponent = <Q extends Query, S extends DeepPartial<Subscription>>(
  query: DocumentNode,
  subscriptionQuery: DocumentNode,
  selection: keyof Query['queue'],
) =>
  Vue.extend({
    data: () => {
      return {
        runningTasks: [] as Q['queue']['all'],
      };
    },
    apollo: {
      runningTasks: {
        query: query,
        update: ({ queue }: Q) => queue[selection],
        subscribeToMore: {
          document: subscriptionQuery,
          updateQuery: (previous: Q, { subscriptionData }: { subscriptionData: { data: S } }) => {
            if (!subscriptionData.data.jobUpdated) return;

            const queue = { ...previous.queue };
            for (const type in queue) {
              if (type === '__typename') continue;
              const key = type as keyof Query['queue'];
              const tasks = [...(queue[key] || [])];
              if (tasks.length) {
                const index = tasks.findIndex(task => task.id === subscriptionData.data.jobUpdated?.id);
                if (subscriptionData.data?.jobUpdated?.state === type) {
                  if (index < 0) {
                    queue[key] = [...tasks, subscriptionData.data.jobUpdated];
                  }
                } else {
                  if (index < 0) {
                    queue[key] = [...tasks].slice(index, 1);
                  }
                }
              }
            }

            return {
              ...previous,
              queue,
            };
          },
        },
      },
    },
  });
