import js from "@eslint/js";
import nextVitals from "eslint-config-next/core-web-vitals";
import prettier from "eslint-config-prettier/flat";
import globals from "globals";

const config = [
  js.configs.recommended,
  ...nextVitals,
  {
    rules: {
      "react-hooks/immutability": "off",
      "react-hooks/preserve-manual-memoization": "off",
      "react-hooks/set-state-in-effect": "off",
    },
  },
  {
    files: ["test/**/*.js"],
    languageOptions: {
      globals: globals.jest,
    },
  },
  prettier,
];

export default config;
