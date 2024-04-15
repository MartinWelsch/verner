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


# begin setup releaseflow repos
RELEASEFLOW_ROOT="$GIT_REPOS_DIR/releaseflow"
mkdir -p "$RELEASEFLOW_ROOT"

REPO="$RELEASEFLOW_ROOT/0.1.0-SNAPSHOT.2"
git_init
git_commit "initial commit"
git_commit "second commit"


REPO="$RELEASEFLOW_ROOT/1.1.0-SNAPSHOT.1"
git_init
git_commit "initial commit"
git_commit "second commit"
_git branch "release/1.0.x"
git_commit "vNext commit"


REPO="$RELEASEFLOW_ROOT/1.0.0-rc"
git_init
git_commit "initial commit"
git_commit "second commit"
_git checkout -b "release/1.0.x"


REPO="$RELEASEFLOW_ROOT/1.0.0-rc.1"
git_init
git_commit "initial commit"
git_commit "second commit"
_git checkout -b "release/1.0.x"
git_commit "fix the rc"


REPO="$RELEASEFLOW_ROOT/1.0.0"
git_init
git_commit "initial commit"
git_commit "second commit"
_git checkout -b "release/1.0.x"
git_commit "fix the rc"
_git tag v1.0.0


REPO="$RELEASEFLOW_ROOT/1.0.1-rc.1"
git_init
git_commit "initial commit"
git_commit "second commit"
_git checkout -b "release/1.0.x"
git_commit "fix the rc"
_git tag v1.0.0
git_commit "patch after release"

# end setup releaseflow repos