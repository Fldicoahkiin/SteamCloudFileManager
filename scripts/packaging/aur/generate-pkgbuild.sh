set -euo pipefail

VERSION="${1:-}"
SHA256="${2:-}"
OUTPUT_DIR="${3:-.}"

if [ -z "$VERSION" ] || [ -z "$SHA256" ]; then
	echo "用法: $0 <版本号> <sha256> [输出目录]"
	echo "示例: $0 0.9.6 abc123def456..."
	exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKGBUILD_TEMPLATE="$SCRIPT_DIR/PKGBUILD.template"
SRCINFO_TEMPLATE="$SCRIPT_DIR/.SRCINFO.template"

if [ ! -f "$PKGBUILD_TEMPLATE" ]; then
	echo "错误: PKGBUILD 模板文件不存在: $PKGBUILD_TEMPLATE"
	exit 1
fi

if [ ! -f "$SRCINFO_TEMPLATE" ]; then
	echo "错误: .SRCINFO 模板文件不存在: $SRCINFO_TEMPLATE"
	exit 1
fi

mkdir -p "$OUTPUT_DIR"

sed -e "s/{{VERSION}}/$VERSION/g" \
	-e "s/{{SHA256}}/$SHA256/g" \
	"$PKGBUILD_TEMPLATE" >"$OUTPUT_DIR/PKGBUILD"

sed -e "s/{{VERSION}}/$VERSION/g" \
	-e "s/{{SHA256}}/$SHA256/g" \
	"$SRCINFO_TEMPLATE" >"$OUTPUT_DIR/.SRCINFO"

echo "已生成 AUR 文件到: $OUTPUT_DIR"
echo "  版本: $VERSION"
echo "  SHA256: $SHA256"
echo ""
echo "生成的文件:"
ls -la "$OUTPUT_DIR/PKGBUILD" "$OUTPUT_DIR/.SRCINFO"
