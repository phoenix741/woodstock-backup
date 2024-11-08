import { ApplicationEventFragment } from '@/generated/graphql';

export interface MergedApplicationEvent extends Omit<ApplicationEventFragment, 'timestamp' | 'step'> {
  startDate: ApplicationEventFragment['timestamp'];
  endDate: ApplicationEventFragment['timestamp'];
}
