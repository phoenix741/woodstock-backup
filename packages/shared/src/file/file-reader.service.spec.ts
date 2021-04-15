import { toArray } from 'rxjs/operators';

import { IndexManifest } from '../manifest/index-manifest.model';
import { globStringToRegex } from '../utils/global-regexp.utils';
import { FileBrowserService } from './file-browser.service';
import { FileReader } from './file-reader.service';

// test('Calculate hash of the files', async () => {
//   const index = new IndexManifest();
//   const service = new FileReader(new FileBrowserService());

//   await new Promise<void>((resolve, reject) => {
//     service
//       .getFiles(index, Buffer.from(__dirname), [], [globStringToRegex('*.spec.ts')])
//       .pipe(toArray())
//       .subscribe({
//         next: (value) => {
//           expect(value.length).toEqual(3);

//           expect(value[0].sha256?.toString('hex')).toMatchSnapshot();
//           expect(value[0].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();

//           expect(value[1].sha256?.toString('hex')).toMatchSnapshot();
//           expect(value[1].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();

//           expect(value[2].sha256?.toString('hex')).toMatchSnapshot();
//           expect(value[2].chunks?.map((c) => c.toString('hex'))).toMatchSnapshot();
//         },
//         complete: () => resolve(),
//         error: (err) => reject(err),
//       });
//   });
// });

// test('Test', async () => {
const index = new IndexManifest();
const service = new FileReader(new FileBrowserService());

// console.time('test');
service
  .getFiles(
    index,
    Buffer.from('/home'),
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
  .subscribe({
    complete: () => console.log('fin'),
    error: (err) => console.log(err),
  });
// console.timeEnd('test');
// }, 1000000000);
