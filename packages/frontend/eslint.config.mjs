import eslint from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import pluginImport from "eslint-plugin-import";
import pluginVue from "eslint-plugin-vue";
import { defineConfig } from "eslint/config";
import globals from "globals";
import tseslint from "typescript-eslint";
import parser from "vue-eslint-parser";

export default defineConfig(
    { ignores: [".vite/"] },
    {
        extends: [
            eslint.configs.recommended,
            tseslint.configs.eslintRecommended,
            tseslint.configs.recommended,
            pluginImport.flatConfigs.recommended,
            pluginImport.flatConfigs.electron,
            pluginImport.flatConfigs.typescript,
            pluginVue.configs["flat/essential"],
        ],

        // Apply these rules to these types of files
        files: ["**/*.{js,ts,tsx,vue}"],

        languageOptions: {
            globals: {
                ...globals.browser,
                ...globals.es2020,
                ...globals.node,
            },

            parser,
            ecmaVersion: 6,
            sourceType: "module",

            parserOptions: {
                parser: tseslint.parser,
            },
        },

        rules: {
            // too many false positives
            "import/no-unresolved": "off",
            // eslint is correct for this lint, but it's a small webapp who cares
            "vue/multi-word-component-names": "off",
            "@typescript-eslint/no-unused-vars": [
                "error",
                {
                    // ignore any unused variables and arguments starting with _
                    varsIgnorePattern: "^_",
                    argsIgnorePattern: "^_",
                    caughtErrors: "all",
                },
            ],
            // requires `import type` because importing a type without the `type` qualifier
            // can break if you do that with the glue package
            "@typescript-eslint/consistent-type-imports": [
                "error",
                { disallowTypeAnnotations: false },
            ],
            "@typescript-eslint/no-import-type-side-effects": "error",
        },
    },
    eslintConfigPrettier,
);
