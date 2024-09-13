import { ApplicationEvent } from '@/generated/graphql';

export interface MergedApplicationEvent extends Omit<ApplicationEvent, 'timestamp' | 'step'> {
  startDate: ApplicationEvent['timestamp'];
  endDate: ApplicationEvent['timestamp'];
}
