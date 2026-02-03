set -euo pipefail

VERSION="${1:-}"
SHA256_X64="${2:-}"
SHA256_ARM64="${3:-}"
OUTPUT_FILE="${4:-steam-cloud-file-manager.rb}"

if [ -z "$VERSION" ] || [ -z "$SHA256_X64" ] || [ -z "$SHA256_ARM64" ]; then
	echo "用法: $0 <版本号> <x64_sha256> <arm64_sha256> [输出文件]"
	echo "示例: $0 0.9.6 abc123... def456..."
	exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_FILE="$SCRIPT_DIR/formula-template.rb"

if [ ! -f "$TEMPLATE_FILE" ]; then
	echo "错误: 模板文件不存在: $TEMPLATE_FILE"
	exit 1
fi

sed -e "s/{{VERSION}}/$VERSION/g" \
	-e "s/{{SHA256_X64}}/$SHA256_X64/g" \
	-e "s/{{SHA256_ARM64}}/$SHA256_ARM64/g" \
	"$TEMPLATE_FILE" >"$OUTPUT_FILE"

echo "已生成: $OUTPUT_FILE"
echo "  版本: $VERSION"
echo "  SHA256 x64: $SHA256_X64"
echo "  SHA256 arm64: $SHA256_ARM64"
