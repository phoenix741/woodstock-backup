export interface RefcntJobData {
  host?: string;
  number?: number;

  fix?: boolean;
  refcnt?: boolean;
  unused?: boolean;

  target?: string;
}
