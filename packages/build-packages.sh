THIS_FILE_DIR="$(readlink -f "$(dirname $0)")"
REPO_ROOT="$(readlink -f "$THIS_FILE_DIR/..")"
OUT_DIR="$REPO_ROOT/dist/packages"

VERSION=$("$REPO_ROOT/target/x86_64-unknown-linux-musl/release/verner" -c "$REPO_ROOT/.verner.yml" git)

echo "Building packages with Version: $VERSION"

mkdir -p "$OUT_DIR"

# build nuget
echo "Building nuget"
NUGET_OUT_DIR="$OUT_DIR/nuget"
mkdir -p $NUGET_OUT_DIR
nuget pack -BasePath "$REPO_ROOT" -OutputDirectory "$NUGET_OUT_DIR" -Version "$VERSION" "$THIS_FILE_DIR/nuget/package.nuspec"