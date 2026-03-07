import { RetentionCategoryDto } from '@/generated/graphql';

export function getRetentionCategoryColor(
  cat?: RetentionCategoryDto | null,
): string {
  switch (cat) {
    case RetentionCategoryDto.Surplus:
      return 'error';
    case RetentionCategoryDto.LastBackup:
      return 'warning';
    case RetentionCategoryDto.Hourly:
      return 'default';
    case RetentionCategoryDto.Daily:
      return 'warning';
    case RetentionCategoryDto.Weekly:
      return 'success';
    case RetentionCategoryDto.Monthly:
      return 'info';
    case RetentionCategoryDto.Yearly:
      return 'primary';
    default:
      return 'default';
  }
}

export function getRetentionCategoryLabel(
  cat?: RetentionCategoryDto | null,
): string {
  switch (cat) {
    case RetentionCategoryDto.Surplus:
      return 'À supprimer';
    case RetentionCategoryDto.LastBackup:
      return 'Conservée';
    case RetentionCategoryDto.Hourly:
      return 'Horaire';
    case RetentionCategoryDto.Daily:
      return 'Quotidien';
    case RetentionCategoryDto.Weekly:
      return 'Hebdo';
    case RetentionCategoryDto.Monthly:
      return 'Mensuel';
    case RetentionCategoryDto.Yearly:
      return 'Annuel';
    default:
      return '';
  }
}
