#!/bin/sh

# Check if there are any staged changes
if [[ -n $(git diff --cached --exit-code) ]]; then
    echo "Staged changes detected. Commit before release."
	exit 1
fi

# Check if there are any unstaged changes
if [[ -n $(git status --porcelain) ]]; then
    echo "Unstaged changes detected. Commit before release."
	exit 1
fi

git cliff > CHANGELOG.md
git commit -m "chore(release) prepare for release"
cargo release --tag-name 'v{{version}}' -v -x $@
