> [!NOTE]
> Please prefer English for all communication in issues and pull requests.

## Creating an issue

Before creating an issue, make sure the problem is not already reported.

## How to contribute

1. **Fork and clone**

   ```sh
   git clone https://github.com/<you>/<repo>.git
   cd <repo>
   npm install
   ```

2. **Create a branch**

   ```sh
   git checkout -b feature/short-description
   ```

3. **Make your changes**

   Follow the conventions in [AGENTS.md](./AGENTS.md). The pre-commit hook
   (lefthook) runs oxlint, oxfmt, `tsc --noEmit`, Vitest, `cargo fmt`,
   `cargo clippy` and `cargo test` on the files you staged — if it fails, fix
   the cause rather than skipping the hook.

4. **Commit**

   ```sh
   git commit -m "feat: add new super feature"
   ```

5. **Push and open a pull request**

   ```sh
   git push -u origin feature/short-description
   ```

## Commit messages

Commit messages follow the [Conventional Commits](https://conventionalcommits.org)
specification:

```
<type>[optional scope]: <description>
```

### Allowed `<type>`

- `chore`: repository maintenance
- `feat`: a new feature
- `fix`: a bug fix
- `perf`: a performance improvement
- `refactor`: neither a feature nor a fix
- `docs`: documentation only
- `ci`: CI configuration and scripts
- `style`: cosmetic change
- `test`: adds or corrects tests
- `revert`: reverts a previous commit
