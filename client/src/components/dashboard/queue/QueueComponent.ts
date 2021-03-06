import { DocumentNode } from 'graphql';
import { Component, Vue } from 'vue-property-decorator';
import { Job } from '@/generated/graphql';

export type Query = {
  queue: Array<Pick<Job, 'id' | 'state'>>;
};

export type Subscription = {
  jobUpdated: Pick<Job, 'id' | 'state'>;
};

export const QueueComponent = <Q extends Query, S extends Subscription>(
  query: DocumentNode,
  subscriptionQuery: DocumentNode,
) => {
  @Component({
    apollo: {
      runningTasks: {
        query,
        update: ({ queue }: Q) => queue,
        variables() {
          return {
            state: this.queryState,
          };
        },
        subscribeToMore: {
          document: subscriptionQuery,
          updateQuery: function (previous: Q, { subscriptionData }: { subscriptionData: { data: S } }) {
            if (!subscriptionData.data.jobUpdated) return;

            const queue = [...(previous?.queue || [])];
            const index = queue.findIndex((task) => task.id === subscriptionData.data.jobUpdated?.id);
            if (
              this.queryState.includes(subscriptionData.data?.jobUpdated?.state || '') ||
              this.queryState.length === 0
            ) {
              if (index < 0) {
                queue.push(subscriptionData.data.jobUpdated);
              }
            } else if (index >= 0) {
              queue.splice(index, 1);
            }

            return {
              ...previous,
              queue,
            };
          },
        },
      },
    },
  })
  class QueueComponent extends Vue {
    state?: string | string[];
    runningTasks: Q['queue'] = [];

    get queryState() {
      return this.state instanceof Array ? this.state : this.state ? [this.state] : [];
    }
  }

  return QueueComponent;
};
