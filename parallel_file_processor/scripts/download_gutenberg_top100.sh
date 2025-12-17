\
#!/usr/bin/env bash
set -euo pipefail

# Download at least 100 plain-text books from Project Gutenberg.
# It scrapes book IDs from the "Top 100" list page and downloads each book's UTF-8 plain text.
#
# Source list page: https://www.gutenberg.org/browse/scores/top

OUT_DIR="${1:-books}"
mkdir -p "$OUT_DIR"

TOP_URL="https://www.gutenberg.org/browse/scores/top"

echo "Fetching top list from: $TOP_URL"
html="$(curl -fsSL "$TOP_URL")"

# Extract ebook IDs like /ebooks/1342, unique, take first 120 to be safe.
ids="$(printf "%s" "$html" | grep -oE '/ebooks/[0-9]+' | cut -d/ -f3 | sort -u | head -n 120)"

count=0
for id in $ids; do
  # Plain text UTF-8 download (works for a large number of books).
  url="https://www.gutenberg.org/ebooks/${id}.txt.utf-8"
  out="${OUT_DIR}/${id}.txt"
  if [[ -f "$out" ]]; then
    continue
  fi

  echo "Downloading $id ..."
  if curl -fsSL "$url" -o "$out"; then
    count=$((count+1))
  else
    echo "  failed: $url" >&2
    rm -f "$out" || true
  fi

  # Stop once we have at least 100
  if [[ "$count" -ge 100 ]]; then
    break
  fi

  # Be polite
  sleep 0.2
done

echo "Downloaded $count books into $OUT_DIR"
if [[ "$count" -lt 100 ]]; then
  echo "WARNING: fewer than 100 downloaded. Re-run the script, or use top1000 list for more IDs." >&2
fi
