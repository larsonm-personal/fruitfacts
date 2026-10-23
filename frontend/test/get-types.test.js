import { getTypes, getTypesForAutocomplete } from '../components/getTypes';

test('search autocomplete reads the tracked type data and sorts display names', () => {
  const types = getTypesForAutocomplete();
  expect(types.length).toBe(getTypes().flatMap((group) => group.types).length);
  expect(types.some((type) => type.name === 'Apple')).toBe(true);
  const names = types.map((type) => type.name_alphabetical || type.name);
  expect(names).toEqual([...names].sort((a, b) => a.localeCompare(b)));
});
