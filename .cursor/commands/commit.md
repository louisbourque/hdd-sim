# Create Commit

Create well-structured commits following Conventional Commits convention and project best practices.

## Execution Steps

When this command is invoked, follow these steps:

### 1. Analyze Current Changes

- Run `git status` to see all modified files
- Run `git diff` to review the actual changes
- Identify the scope and nature of the changes

### 2. Validate Code Quality

Before committing, verify:

- Run `cargo check` to ensure TypeScript compiles
- Run `cargo fmt` to check code style
- Confirm tests pass (if applicable): `cargo test`
- No unrelated files are included
- No sensitive data or .env files are staged

### 3. Determine Commit Type and Scope

**Analyze the changes** and select the appropriate commit type:

- **feat**: New functionality or feature
- **fix**: Bug correction
- **docs**: Documentation changes only
- **style**: Code formatting (spaces, semicolons, etc.)
- **refactor**: Code restructuring without changing behavior
- **test**: Adding or modifying tests
- **chore**: Maintenance tasks, configuration, dependencies

### 4. Generate and Create Commit

**IMPORTANT**: This command should automatically create the commit after validation. Do NOT just show the commit message and wait for confirmation - proceed directly to creating the commit.

Steps:

1. Generate a commit message following the structure below
2. Stage the appropriate files using `git add`
3. Create the commit immediately using `git commit`
4. Show the result to the user

## Commit Message Structure

```
<type>(<scope>): <description>

[optional body with context]

[optional footer]
```

**Rules:**

- Description MUST be in English
- Use imperative mood ("add" not "added" or "adds")
- First line max 72 characters
- Description should be clear and concise
- Body should explain "why" not "what" (the diff shows "what")

## Examples

### Feature commit

```
feat(dashboard): add QuickSight iframe integration

Integrates AWS QuickSight dashboard using iframe component
with authentication token handling.
```

### Bug fix commit

```
fix(auth): correct expired token validation

Fixes issue where expired tokens were not properly detected,
causing users to see error messages instead of redirect to login.
```

### Refactor commit

```
refactor(casl): implement dual permission system

Refactors CASL authorization to support both group-based
and specific permissions for more granular access control.
```

### Test commit

```
test(payment-links): add unit tests for validation

Adds comprehensive unit tests for payment link validation logic
covering edge cases and error scenarios.
```

### Chore commit

```
chore(deps): update Next.js to v15.1.0
```

## Pre-Commit Checklist

Verify before committing:

- [ ] All changes are related to a single logical change
- [ ] Message follows Conventional Commits convention
- [ ] Description is clear and in English
- [ ] Code compiles without errors (`cargo check`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Tests pass (if applicable)
- [ ] No sensitive data or credentials included
- [ ] No .env files staged

## Output After Commit

After creating the commit:

1. Show the commit message that was created
2. Run `git log -1` to confirm the commit
3. Provide a summary of what was committed

If any validation fails, ask the user if they want to:

- Fix the issues first (recommended)
- Proceed anyway
- Cancel the commit
