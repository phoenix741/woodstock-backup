export class Backup {
  constructor(
    public number: number,
    public complete: boolean,

    public startDate: Date,
    public endDate: Date,

    public fileCount: number,
    public newFileCount: number,
    public existingFileCount: number,

    public fileSize: number,
    public existingFileSize: number,
    public newFileSize: number,

    public speed: number,
  ) {}
}
