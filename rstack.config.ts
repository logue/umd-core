import { pluginSass } from '@rsbuild/plugin-sass';
import { define } from 'rstack';

import { readFileSync } from 'node:fs';

import markdownPlugin from '@eslint/markdown';
import prettierPlugin from 'eslint-config-prettier';
import { configs as yamlConfigs } from 'eslint-plugin-yml';

/** Parse package.json content */
const pkg = JSON.parse(readFileSync('./package.json', 'utf-8')) as {
  name: string;
  description: string;
  author: {
    name: string;
    email: string;
  };
  license: string;
  version: string;
  homepage: string;
};

const buildDate = new Date().toISOString();
const _bannerText = `/**
* ${pkg.name}
*
* @description ${pkg.description}
* @author ${pkg.author.name} <${pkg.author.email}>
* @license ${pkg.license}
* @version ${pkg.version}
* @see {@link ${pkg.homepage}}
*/
`;

const createBuildDefines = () => ({
  'import.meta.env.APP_VERSION': JSON.stringify(pkg.version),
  'import.meta.env.BUILD_DATE': JSON.stringify(buildDate)
});

const IGNORE_PATTERNS = [
  // AI agents skill docs.
  '**/.agent/**',
  '**/.agents/**',
  '**/.claude/**',
  '**/.cursor/**',
  '**/.windsurf/**',
  // Build intermediate artifacts.
  '**/.cache/**',
  '**/.data/**',
  '**/.mf/**',
  '**/.nitro/**',
  '**/.nuxt/**',
  '**/.pnpm/**',
  '**/.rstack/**',
  // Test coverage reports.
  '**/coverage/**',
  '**/e2e/**',
  '**/playwright-report/**',
  '**/reports/**',
  '**/test-results/**',
  // Build artifacts.
  '**/demo/**',
  '**/dist/**',
  '**/docs/**',
  '**/storybook-static/**',
  '**/target/**',
  // Node modules
  '**/node_modules/**'
];

const APP_FILES = ['**/*.{ts,mts,tsx,js,mjs,jsx,json,jsonc,vue,astro,svelte}'];
const TEST_FILES = ['**/*.{test,spec}.{ts,mts,tsx,js,mjs,jsx}'];
const DOC_FILES = ['**/*.{md,mdx}'];
const YAML_FILES = ['**/*.{yml,yaml}'];

/**
 * Rsbuild (Demo application) config
 * @see {@link https://rsbuild.rs/config | Rsbuild Config}
 */
define.app({
  plugins: [pluginSass()],
  source: {
    define: createBuildDefines(),
    entry: {
      'umd-reference': './scss/umd-reference.scss'
    },
    include: ['./src'],
    tsconfigPath: './tsconfig.rsbuild.json'
  },
  output: {
    target: 'web',
    distPath: {
      root: './dist'
    },
    cleanDistPath: false,
    sourceMap: false,
    filename: {
      css: '[name].css',
      font: '[name][ext]'
    }
  }
});

/**
 * Rstest Config
 * @see {@link https://rstest.rs/config/ | Rstest Config}
 */
define.test({
  source: {
    tsconfigPath: './tsconfig.rstest.json'
  }
});

/**
 * Rslint Config
 * @see {@link https://rslint.rs/config/ | Rslint Config}
 */
define.lint(
  ({
    globals,
    // See https://rslint.rs/config/rules-and-presets
    js,
    ts,
    reactPlugin,
    reactHooksPlugin,
    importPlugin,
    // nodePlugin,
    promisePlugin,
    // jestPlugin,
    rstestPlugin,
    unicornPlugin,
    jsxA11yPlugin
  }) => [
    {
      ignores: IGNORE_PATTERNS,
      languageOptions: {
        lobals: {
          ...globals.browser,
          ...globals.nodeBuiltin
        },
        parserOptions: {
          project: [
            './tsconfig.node.json',
            './tsconfig.rsbuild.json',
            './tsconfig.rstest.json'
          ]
        }
      }
    },

    // Base TypeScript recommended sets.
    js.configs.recommended,
    ts.configs.strictTypeChecked,
    ts.configs.stylisticTypeChecked,
    importPlugin.configs.recommended,
    promisePlugin.configs.recommended,
    unicornPlugin.configs.recommended,
    // nodePlugin.configs.recommended,
    reactPlugin.configs.recommended,
    reactHooksPlugin.configs.recommended, // Comment out this line when using Vue.
    jsxA11yPlugin.configs.recommended,

    {
      files: APP_FILES,
      plugins: ['@typescript-eslint', 'import', 'promise', 'unicorn'],
      rules: {
        '@typescript-eslint/array-type': [
          'error',
          {
            default: 'array'
          }
        ],

        // Keep project lint behavior aligned with the previous baseline.
        '@typescript-eslint/ban-ts-comment': 'off',
        '@typescript-eslint/consistent-generic-constructors': [
          'error',
          'type-annotation'
        ],
        '@typescript-eslint/explicit-function-return-type': 'off',

        // Ignore intentionally unused identifiers with underscore prefix.
        '@typescript-eslint/no-unused-vars': [
          'error',
          {
            args: 'all',
            argsIgnorePattern: '^_',
            caughtErrors: 'all',
            caughtErrorsIgnorePattern: '^_',
            destructuredArrayIgnorePattern: '^_',
            ignoreRestSiblings: true,
            varsIgnorePattern: '^_'
          }
        ],
        '@typescript-eslint/triple-slash-reference': 'off',
        // Use them according to the context (see AGENTS.md).
        '@typescript-eslint/consistent-type-definitions': 'off',

        // Using parent traversal is prohibited in app code. Use @/ alias instead.
        'import/no-relative-parent-imports': [
          'error',
          {
            ignore: ['^@/']
          }
        ],
        'import/order': [
          'error',
          {
            alphabetize: {
              order: 'asc'
            },
            groups: [
              'builtin',
              'external',
              'parent',
              'sibling',
              'index',
              'object',
              'type'
            ],
            'newlines-between': 'always',
            pathGroups: [
              {
                group: 'builtin',
                pattern:
                  '{@rsbuild/**,@rsdoctor/**,@rslint/**,@rslib/*,@rspack/**,@rstest/**,rstack,rstack/**}',
                position: 'before'
              },
              {
                group: 'internal',
                pattern: '{@/**}',
                position: 'before'
              }
            ],
            pathGroupsExcludedImportTypes: ['builtin']
          }
        ],
        // File names should, in principle, be in PascalCase, with some exceptions.
        'unicorn/filename-case': 'off'
      },
      settings: {
        'import/resolver': {
          typescript: {
            project: [
              './tsconfig.node.json',
              './tsconfig.rsbuild.json',
              './tsconfig.rspress.json',
              './tsconfig.rslib.json',
              './tsconfig.rstest.json'
            ]
          }
        }
      }
    },

    {
      // Test files intentionally import from parent directories.
      files: TEST_FILES,
      ...rstestPlugin.configs.recommended,
      rules: {
        // The demo data for the test code should preferably be of type `any`.
        '@typescript-eslint/no-explicit-any': 'warn',
        'import/no-relative-parent-imports': 'off'
      }
    },

    ...markdownPlugin.configs.recommended.map(config => ({
      ...config,
      language: 'markdown/gfm',
      files: DOC_FILES
    })),

    ...yamlConfigs['flat/prettier'].map(config => ({
      ...config,
      files: YAML_FILES,
      plugins: ['yml']
    })),
    prettierPlugin.rules
  ]
);

/**
 * Prettier Configs
 * @see {@link https://prettier.io/docs/options | Prettier Options}
 */
define.fmt({
  printWidth: 80,
  tabWidth: 2,
  useTabs: false,
  semi: true,
  singleQuote: true,
  trailingComma: 'none',
  bracketSpacing: true,
  bracketSameLine: false,
  arrowParens: 'avoid',
  endOfLine: 'lf',
  // Bellow is rstack options
  // see https://rstack.rs/guide/formatting
  ignorePatterns: IGNORE_PATTERNS,
  sortPackageJson: true
});

/**
 * Git hooks
 * @see {@link https://rstack.rs/guide/cli/hooks | Git Hooks}
 */
define.staged({
  '*.{js,jsx,ts,tsx,mjs,cjs,mts,cts}': ['rs lint --fix', 'rs fmt'],
  '*.{json,md,mdx,css,scss,less,html,yml,yaml}': 'rs fmt'
});
