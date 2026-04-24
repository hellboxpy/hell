Release a new version of hell by running `scripts/release.sh`.

If $ARGUMENTS is provided, map it to a flag:
- "major" → `--major`
- "minor" → `--minor`
- "patch" → `--patch`

If $ARGUMENTS is empty, ask the user which version component to increment before proceeding.

Run the script from the project root with the appropriate flag, then report the version that was released.
