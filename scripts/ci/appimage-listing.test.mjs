import assert from 'node:assert/strict';
import test from 'node:test';
import { parseAppImageListing } from './appimage-listing.mjs';

test('Ubuntu 22.04 AppRun symlinks retain their audited name', () => {
  // Actual listing format from the failed Bevy AppImage package job.
  const entries = parseAppImageListing(`drwxr-xr-x 0/0 106 2026-10-02 05:28 squashfs-root
lrwxrwxrwx 0/0 13 2026-10-02 05:28 squashfs-root/AppRun -> usr/bin/nbcad
lrwxrwxrwx 0/0 46 2026-10-02 05:28 squashfs-root/nbcad.png -> usr/share/icons/hicolor/256x256/apps/nbcad.png
-rwxr-xr-x 0/0 235589672 2026-10-02 05:27 squashfs-root/usr/bin/nbcad`);
  assert.deepEqual(entries, [
    { type: 'l', permissions: 'rwxrwxrwx', path: 'AppRun' },
    { type: 'l', permissions: 'rwxrwxrwx', path: 'nbcad.png' },
    { type: '-', permissions: 'rwxr-xr-x', path: 'usr/bin/nbcad' },
  ]);
});

test('library links retain paths used by the host-only library audit', () => {
  const entries = parseAppImageListing('lrwxrwxrwx 1001/1001 23 2026-10-02 05:28 squashfs-root/usr/lib/libwayland-client.so.0 -> libwayland-client.so.0.3');
  assert.equal(entries[0].path, 'usr/lib/libwayland-client.so.0');
});

test('regular paths with spaces preserve their name and stored permissions', () => {
  const entries = parseAppImageListing('-rwx------ 1001/1001 42 2026-10-02 05:28 squashfs-root/usr/share/Example -> notice.txt');
  assert.deepEqual(entries[0], { type: '-', permissions: 'rwx------', path: 'usr/share/Example -> notice.txt' });
});
