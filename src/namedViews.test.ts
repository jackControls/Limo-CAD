import { translateByPartOffset } from './namedViewOffsets';
import type { ViewPartOffsetDto } from './engine/types';

function same(actual: unknown, expected: unknown, message: string) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: ${JSON.stringify(actual)}`);
  }
}

const offsets: ViewPartOffsetDto[] = [
  { body_id: 2, translation: [0, 14, 0] },
  { body_id: 3, translation: [0, -14, 0] },
];

same(
  translateByPartOffset([10, 20, 30], 2, offsets),
  [10, 34, 30],
  'A recalled explode offset is added in world millimeters',
);
same(
  translateByPartOffset([10, 20, 30], 3, offsets),
  [10, 6, 30],
  'The opposite clip moves the other way',
);
same(
  translateByPartOffset([1, 2, 3], 9, offsets),
  [1, 2, 3],
  'A body without an offset keeps the assembled pose',
);
const original: [number, number, number] = [1, 2, 3];
translateByPartOffset(original, 2, offsets);
same(original, [1, 2, 3], 'Display offsets do not mutate the source translation');

console.log('Named view display offsets passed.');
