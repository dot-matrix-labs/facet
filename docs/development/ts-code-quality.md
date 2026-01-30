# TypeScript Code Quality Standards

This document outlines the code quality standards for the Robert browser automation app, detailing the tools used, our zero-tolerance policy for warnings, and the specific fixes applied to achieve full compliance.

## Standards Overview

We maintain strict code quality standards to ensure maintainability, accessibility, and type safety:

- **Zero Errors**: All TypeScript, ESLint, and formatting errors must be resolved
- **Zero Warnings**: All accessibility and code quality warnings must be addressed
- **Automated Checks**: All checks must pass in CI/CD pipelines

## Quality Tools

### 1. ESLint (v9.37.0)

**Purpose**: Enforces JavaScript/TypeScript code quality and best practices

**Configuration**: `eslint.config.js`

**Key Rules**:

- `@typescript-eslint/no-unused-vars`: Error for unused variables (with `argsIgnorePattern: '^_'` for prefixed variables)
- `@typescript-eslint/no-explicit-any`: Warning for explicit `any` types
- `no-unused-vars`: Error for unused variables in JavaScript

**Running**:

```bash
bun run lint        # Check for issues
bun run lint:fix    # Auto-fix issues
```

### 2. Prettier (v3.6.2)

**Purpose**: Enforces consistent code formatting

**Configuration**: `.prettierrc.json`

**Running**:

```bash
bun run format      # Check formatting
bun run format:fix  # Auto-fix formatting
```

### 3. svelte-check (v4.3.3)

**Purpose**: TypeScript type checking and Svelte-specific linting, including accessibility checks

**Running**:

```bash
bun run check       # Run type checking
```

### 4. All Checks Combined

```bash
bun run check:all   # Runs lint, format, and check in sequence
```

## Zero Warnings Policy

**All warnings must be resolved**, not just errors. This includes:

- TypeScript type errors
- ESLint code quality issues
- Accessibility (a11y) warnings
- Formatting inconsistencies

Warnings indicate potential bugs, accessibility issues, or code smells that should be addressed proactively.

## Recent Quality Fixes

The following changes were made to achieve zero errors and zero warnings:

### 1. Dependency Version Conflicts

**Issue**: Package version constraints prevented installation

**Files Changed**:

- `package.json`

**Changes**:

```diff
- "eslint-plugin-svelte": "^2.48.0"
+ "eslint-plugin-svelte": "^2"

- "svelte-eslint-parser": "^0.45.1"
+ "svelte-eslint-parser": "^0"
```

**Reason**: Specific patch versions were unavailable in the registry. Relaxed to major version constraints to allow installation.

---

### 2. Unused Imports

**Issue**: `onMount` imported but never used

**Files Changed**:

- `src/components/DebugView.svelte`

**Changes**:

```diff
- import { onMount, afterUpdate } from 'svelte';
+ import { afterUpdate } from 'svelte';
```

**Reason**: Removed unused import to satisfy `no-unused-vars` ESLint rule.

---

### 3. Unused Error Parameters

**Issue**: Caught error variable defined but never used

**Files Changed**:

- `src/components/UrlInput.svelte`

**Changes**:

```diff
  try {
    await handleLaunchBrowser();
- } catch (err) {
+ } catch {
    return; // Error already handled
  }
```

**Reason**: Error is handled in the called function, so the parameter is unnecessary. Removed to satisfy `no-unused-vars` rule.

---

### 4. TypeScript Null Safety Issues

**Issue**: TypeScript couldn't narrow types within Svelte `{#if}` blocks

**Files Changed**:

- `src/components/DeveloperMode.svelte`

**Changes**: Added non-null assertions (`!`) after confirming null checks in template logic

```diff
  {#if systemPaths}
    <code>{systemPaths.installation_dir}</code>
-   <button on:click={() => copyToClipboard(systemPaths.installation_dir)}>
+   <button on:click={() => copyToClipboard(systemPaths!.installation_dir)}>
```

**Locations**: 9 instances across:

- `installation_dir` (line 127)
- `config_dir` (line 137)
- `data_dir` (line 147)
- `cache_dir` (line 157)
- `temp_dir` (line 167)
- `current_dir` (line 177)
- `chrome_path` (display: line 190, button: line 191)
- `serverStatus.url` (line 219)

**Reason**: Svelte's reactive `{#if}` blocks don't narrow TypeScript types in event handlers. Non-null assertions are safe because they're only accessible within null-guarded blocks.

---

### 5. Exhaustive Switch Type Error

**Issue**: `default` case in exhaustive switch had type `never`

**Files Changed**:

- `src/components/DebugView.svelte`

**Changes**:

```diff
  default:
-   return JSON.stringify(event.data);
+   // Exhaustive check - should never reach here
+   return JSON.stringify((event as any).data);
```

**Reason**: TypeScript correctly inferred that all union cases were handled, narrowing to `never`. Cast to `any` for the unreachable fallback case.

---

### 6. Accessibility: Label Without Control

**Issue**: `<label>` elements not associated with form controls

**Files Changed**:

- `src/components/DeveloperMode.svelte`

**Changes**: Replaced semantic `<label>` elements with `<span>` elements

```diff
- <label>Installation Directory:</label>
+ <span class="path-label">Installation Directory:</span>
```

**Locations**: 7 instances for:

- Installation Directory (line 124)
- Config Directory (line 137)
- Data Directory (line 147)
- Cache Directory (line 157)
- Temp Directory (line 167)
- Current Directory (line 177)
- Chrome Path (line 188)

**CSS Update**:

```diff
- .path-item label {
+ .path-label {
    font-weight: 600;
    color: #555;
    font-size: 0.875rem;
  }
```

**Reason**: These are display-only text, not form labels. Using `<span>` is semantically correct and resolves the a11y warning.

---

### 7. Accessibility: Click Handler Without Keyboard Support

**Issue**: Clickable `<div>` missing keyboard event handler and ARIA role

**Files Changed**:

- `src/components/SystemStatus.svelte`

**Changes**: Added `role`, `tabindex`, and keyboard handler

```diff
  <div
    class="status-header"
+   role="button"
+   tabindex="0"
    on:click={() => (expanded = !expanded)}
+   on:keydown={(e) => {
+     if (e.key === 'Enter' || e.key === ' ') {
+       e.preventDefault();
+       expanded = !expanded;
+     }
+   }}
  >
```

**Reason**: Interactive elements must be keyboard-accessible for a11y compliance. Added ARIA role for semantic meaning and keyboard handler for Enter/Space key support.

---

## Maintenance Guidelines

### Before Committing

Always run quality checks before committing:

```bash
bun run check:all
```

All checks must pass with **0 errors and 0 warnings**.

### Common Issues and Solutions

#### Unused Variables

- **Solution**: Remove the variable or prefix with `_` if required by syntax (e.g., `_err`)
- **Note**: For catch blocks, omit the parameter entirely if unused

#### TypeScript Null Safety in Svelte

- **Solution**: Use non-null assertions (`!`) when you've confirmed null checks in template logic
- **Example**: `{#if foo}` → access as `foo!` in event handlers within that block

#### Accessibility Warnings

- **Labels**: Only use `<label>` when associated with form controls (inputs, selects, etc.)
- **Interactive elements**: Add `role="button"`, `tabindex="0"`, and keyboard handlers to clickable non-button elements
- **Alternative**: Consider using actual `<button>` elements when possible

#### Formatting Issues

- **Solution**: Run `bun run format:fix` to auto-format all files
- **IDE Setup**: Configure your editor to format on save using Prettier

### CI/CD Integration

Ensure your CI pipeline runs:

```bash
bun install
bun run check:all
```

The pipeline should fail if any errors or warnings are present.

## Conclusion

Maintaining zero warnings ensures:

- **Type Safety**: Catch bugs at compile-time
- **Accessibility**: Ensure all users can interact with the app
- **Maintainability**: Consistent, clean code is easier to understand and modify
- **Quality**: Professional-grade codebase that follows best practices

When in doubt, fix the warning rather than suppressing it.
