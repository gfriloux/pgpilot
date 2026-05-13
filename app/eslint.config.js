import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    plugins: { 'react-hooks': reactHooks },
    rules: {
      ...reactHooks.configs.recommended.rules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'react-hooks/exhaustive-deps': 'warn',
      'no-console': 'warn',
    },
  },
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'storybook-static/**',
      '*.config.ts',
      '*.config.cjs',
      '*.config.js',
      'src/components/*.stories.tsx',
    ],
  },
);
