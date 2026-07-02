// replace spaces with '_' and urlencode the remainder
function name_to_path(input) {
  let output = input.replace(/ /g, '_');
  console.log('name_to_path()');
  console.log(output);
  return encodeURI(output); // don't encode '/'
}

// url-decode and replace '_' with spaces
function path_to_name(input) {
  let output = decodeURIComponent(input); // seek to decode ALL (including %2f->'/')
  return output.replace(/_/g, ' ');
}

function reference_url(input) {
  if (!input) {
    return null;
  }

  const match = input.trim().match(/https?:\/\/\S+/i);
  if (!match) {
    return null;
  }

  return match[0].replace(/[),.;]+$/, '');
}

module.exports = { name_to_path, path_to_name, reference_url };
