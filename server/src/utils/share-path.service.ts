import { Injectable } from '@nestjs/common';

@Injectable()
export class SharePathService {
  mangle(path: string): string {
    return encodeURIComponent(path || '');
  }

  unmangle(path: string) {
    return decodeURIComponent(path);
  }
}
