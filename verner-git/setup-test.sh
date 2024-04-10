ROOT_DIR="$(readlink -f "$(dirname $0)")"
GIT_REPOS_DIR="$(readlink -f "$ROOT_DIR/../test_data")"

rm -rf "$GIT_REPOS_DIR"
mkdir "$GIT_REPOS_DIR"

function git_init
{
    git init -b main --separate-git-dir "$REPO" "$REPO"
}

function _git
{
    git --git-dir "$REPO" "$@"
}

function git_commit
{
    _git commit --allow-empty -m "$1"
}


# begin setup semver repos
SEMVER_ROOT="$GIT_REPOS_DIR/semver"
mkdir -p "$SEMVER_ROOT"

REPO="$SEMVER_ROOT/0.0.1-SNAPSHOT.2"
git_init
git_commit "initial commit"
git_commit "second commit"


REPO="$SEMVER_ROOT/1.0.1-SNAPSHOT.1"
git_init
git_commit "initial commit"
git_commit "second commit"
_git branch "release/1.0.x"
git_commit "vNext commit"

# end setup semver repos