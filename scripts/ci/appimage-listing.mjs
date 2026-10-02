// unsquashfs -lln appends a link target to symlink rows. Keep that target out
// of the stored path so AppRun and library links can be audited by name.
export function parseAppImageListing(listing) {
  return listing.split('\n')
    .map(line => line.match(/^([-dl])([-rwxsStT]{9})\s+\S+\s+\d+\s+\S+\s+\S+\s+squashfs-root\/(.+)$/))
    .filter(Boolean)
    .map(([, type, permissions, path]) => ({
      type,
      permissions,
      path: type === 'l' ? path.replace(/ -> .*$/, '') : path,
    }));
}
