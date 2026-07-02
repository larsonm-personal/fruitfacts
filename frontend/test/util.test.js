const { name_to_path, path_to_name, reference_url } = require('../components/util');

test('name_to_path', () => {
  expect(name_to_path("Oregon/Willamette Valley U-picks/Brosi's Sugartree Farms")).toBe(
    "Oregon/Willamette_Valley_U-picks/Brosi's_Sugartree_Farms"
  );
});

test('path_to_name', () => {
  expect(path_to_name("Oregon/Willamette%20Valley%20U-picks%2FBrosi's%20Sugartree%20Farms")).toBe(
    "Oregon/Willamette Valley U-picks/Brosi's Sugartree Farms"
  );
  expect(path_to_name("Oregon/Willamette_Valley_U-picks/Brosi's_Sugartree_Farms")).toBe(
    "Oregon/Willamette Valley U-picks/Brosi's Sugartree Farms"
  );
});

test('reference_url', () => {
  expect(
    reference_url(
      'Fruit Notes, Volume 85, Spring, 2020 http://www.umassfruitnotes.com/v85n2/a6.pdf'
    )
  ).toBe('http://www.umassfruitnotes.com/v85n2/a6.pdf');
  expect(reference_url('no external link here')).toBeNull();
});
