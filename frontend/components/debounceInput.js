import React from 'react';

// Keep search typing local; publish the latest value after the user pauses.
export default function DebounceInput({ minLength = 0, debounceTimeout = 300, onChange, ...props }) {
  const timer = React.useRef(null);
  React.useEffect(() => () => clearTimeout(timer.current), []);

  return <input {...props} onChange={(event) => {
    const value = event.target.value;
    clearTimeout(timer.current);
    if (value.length >= minLength || value.length === 0) {
      timer.current = setTimeout(() => onChange({ target: { value } }), debounceTimeout);
    }
  }} />;
}
