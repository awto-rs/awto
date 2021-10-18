#!/bin/bash
set -e

echo "awto-macros"
cd awto-macros
sed -i '' -e 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "chore(awto-macros): version $1"
cargo publish
cd ..
sleep 10

echo "awto"
cd awto
sed -i '' -e 's/^version.*$/'"version = \"$1\"/" Cargo.toml
sed -i '' -e 's/^awto-macros [^,]*,/awto-macros = { version = "'$1'",/' Cargo.toml
git commit -am "chore(awto): version $1"
cargo publish
cd ..
sleep 10

echo "awto-compile"
cd awto-compile
sed -i '' -e 's/^version.*$/'"version = \"$1\"/" Cargo.toml
sed -i '' -e 's/^awto [^,]*,/awto = { version = "'$1'",/' Cargo.toml
git commit -am "chore(awto-compile): version $1"
cargo publish
cd ..
sleep 10

echo "awto-cli"
cd awto-cli
sed -i '' -e 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "chore(awto-cli): version $1"
cargo publish
cd ..
sleep 10
