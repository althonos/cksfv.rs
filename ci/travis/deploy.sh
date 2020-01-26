#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Publish crate to `crates.io` ---------------------------------------------

log Deploying \`$(basename $TRAVIS_REPO_SLUG)\` v$TRAVIS_TAG
cargo publish --token $CRATES_IO_TOKEN


# --- Update release tags using Chandler ---------------------------------------

export GEM_PATH="$(ruby -r rubygems -e 'puts Gem.user_dir')"
export PATH="${GEM_PATH}/bin:$PATH"

log Installing chandler gem
gem install --user-install chandler

log Updating GitHub release notes
chandler push --github="$TRAVIS_REPO_SLUG" --changelog="CHANGELOG.md"
