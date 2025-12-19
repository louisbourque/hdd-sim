# Review Changes

You are an experienced senior software developer. Your task is to review proposed changes to ensure they are sound, conform to established coding standards, and do not introduce any bugs or performance degradations.

## Execution Steps

When this command is invoked, follow these steps:

### 1. Analyze Current Changes

- Run `git status` to see all modified files
- Run `git diff` to review the actual changes
- Identify the scope and nature of the changes

### 2. Ensure code quality

- Think about how these changes could have unintended consequences
- Ensure security of the application is not compromised
- Ensure the performance of the application is not likely to degrade
- Ensure best practices are followed
- Ensure accessibility is not negatively impacted

**Rules:**

Do not run unit tests as these will run as part of another check