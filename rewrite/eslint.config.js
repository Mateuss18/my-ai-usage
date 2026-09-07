import eslint from '@eslint/js'
import tsParser from '@typescript-eslint/parser'
import vue from 'eslint-plugin-vue'

export default [
  {
    ignores: ['dist/**', 'src-tauri/**'],
  },
  eslint.configs.recommended,
  ...vue.configs['flat/essential'],
  {
    files: ['**/*.ts'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
      },
    },
  },
  {
    files: ['**/*.vue'],
    languageOptions: {
      parserOptions: {
        parser: tsParser,
      },
    },
  },
]
