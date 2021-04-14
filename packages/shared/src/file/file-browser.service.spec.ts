import * as Long from 'long';
import { reduce } from 'rxjs/operators';

import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';

const service = new FileBrowserService();

console.time('service');
const second$ = service
  .getFiles(
    Buffer.from('/home/'),
    [],
    [
      globStringToRegex('lost+found'),
      globStringToRegex('phoenix/tensorflow'),
      globStringToRegex('phoenix/tmp'),
      globStringToRegex('phoenix/.composer'),
      globStringToRegex('*node_modules'),
      globStringToRegex('*mongodb/db'),
      globStringToRegex('phoenix/.ccache'),
      globStringToRegex('*mongodb/dump'),
      globStringToRegex('phoenix/usr/android-sdk'),
      globStringToRegex('phoenix/.cache'),
      globStringToRegex('phoenix/.CloudStation'),
      globStringToRegex('phoenix/.android'),
      globStringToRegex('phoenix/.AndroidStudio*'),
      globStringToRegex('phoenix/usr/android-studio'),
      globStringToRegex('*.vmdk'),
      globStringToRegex('phoenix/.nvm'),
      globStringToRegex('*.vdi'),
      globStringToRegex('phoenix/.local/share/Trash'),
      globStringToRegex('phoenix/VirtualBox VMs'),
      globStringToRegex('*mongodb/configdb'),
      globStringToRegex('phoenix/.thumbnails'),
      globStringToRegex('phoenix/.VirtualBox'),
      globStringToRegex('phoenix/.vagrant.d'),
      globStringToRegex('phoenix/vagrant'),
      globStringToRegex('phoenix/.npm'),
      globStringToRegex('phoenix/Pictures'),
      globStringToRegex('phoenix/Documents synhronisés'),
      globStringToRegex('phoenix/dwhelper'),
      globStringToRegex('phoenix/snap'),
      globStringToRegex('phoenix/.local/share/flatpak'),
      globStringToRegex('phoenix/usr/AndroidSdk'),
      globStringToRegex('public/kg/gallery'),
      globStringToRegex('*vcpkg'),
    ],
  )
  .pipe(
    reduce((acc, file) => {
      return acc + file.stats?.size.toNumber();
    }, 0),
  )
  .subscribe({
    next: (cnt) => {
      console.log(cnt);
    },
    complete: () => {
      console.timeEnd('service');
    },
  });

/*
count 2 140 350
first: 5:14.524 (m:ss.mmm)
*/

/*
count 2 140 440
second: 1:27.043 (m:ss.mmm)
*/
