#!/bin/sh
# Install git hooks for this repository

HOOKS_DIR="$(git rev-parse --show-toplevel)/.git/hooks"
SCRIPT_DIR="$(dirname "$0")"

echo "Installing git hooks..."

# Pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/sh
# Pre-commit hook: format check and clippy

echo "Running cargo fmt check..."
cargo fmt --all --check
if [ $? -ne 0 ]; then
    echo "❌ cargo fmt failed. Run 'cargo fmt' to fix."
    exit 1
fi

echo "Running cargo clippy..."
cargo clippy --workspace --all-targets -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ cargo clippy failed. Fix warnings before committing."
    exit 1
fi

echo "✅ Pre-commit checks passed"
EOF

# Pre-push hook
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/sh
# Pre-push hook: run tests and doc build

echo "Running cargo test..."
cargo test --workspace
if [ $? -ne 0 ]; then
    echo "❌ Tests failed. Fix before pushing."
    exit 1
fi

echo "Building docs..."
cargo doc --workspace --no-deps
if [ $? -ne 0 ]; then
    echo "❌ Doc build failed."
    exit 1
fi

echo "✅ Pre-push checks passed"
EOF

chmod +x "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Git hooks installed successfully"
