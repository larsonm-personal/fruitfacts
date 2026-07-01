// get types from the backend-generated types file. in a library file so we can use it a few places
export function getTypes() {
  const json5 = require('json5');
  const fs = require('fs');
  const path = require('path');

  let typesFile = fs.readFileSync(path.join(process.cwd(), '../plant_database/types.json5'));
  const types = json5.parse(typesFile);

  return types;
}

export function getTypesForAutocomplete() {
  const types = getTypes();

  let typesFlattened = [];

  types.forEach((group) => {
    typesFlattened = typesFlattened.concat(group.types);
  });

  // we now have an array of these: (see types.json5 input file):
  //  {
  //    name: "Euro Plum",
  //    name_alphabetical: "Plum, Euro",
  //    latin_name: "Prunus domestica"
  //  }

  typesFlattened.sort(function compareFn(a, b) {
    const a_name = a.name_alphabetical || a.name;
    const b_name = b.name_alphabetical || b.name;
    return a_name.localeCompare(b_name);
  });

  return typesFlattened;
}
