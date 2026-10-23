import nextVitals from 'eslint-config-next/core-web-vitals';
import prettier from 'eslint-config-prettier/flat';
// Legacy Pages Router effects predate React Compiler; retain diagnostics as warnings.
export default [...nextVitals, prettier, { rules: {
  "react-hooks/set-state-in-effect": "warn",
  "react-hooks/immutability": "warn"
} }];
