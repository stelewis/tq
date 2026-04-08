#!/usr/bin/env bash

set -euo pipefail

dist_dir="${1:-dist}"

if [ ! -d "$dist_dir" ]; then
  echo "Distribution directory '$dist_dir' does not exist"
  exit 1
fi

shopt -s nullglob

wheels=("$dist_dir"/*.whl)
sdists=("$dist_dir"/*.tar.gz)

if [ "${#wheels[@]}" -ne 4 ]; then
  echo "Expected exactly 4 wheel artifacts in $dist_dir, found ${#wheels[@]}"
  exit 1
fi

if [ "${#sdists[@]}" -ne 1 ]; then
  echo "Expected exactly 1 source distribution artifact in $dist_dir, found ${#sdists[@]}"
  exit 1
fi

linux_count=0
macos_x86_64_count=0
macos_arm64_count=0
windows_x86_64_count=0

for wheel_path in "${wheels[@]}"; do
  wheel_name="$(basename "$wheel_path")"

  case "$wheel_name" in
    *manylinux*.whl|*musllinux*.whl)
      linux_count=$((linux_count + 1))
      ;;
    *linux*.whl)
      echo "Wheel '$wheel_name' uses a native linux platform tag that PyPI will reject"
      exit 1
      ;;
  esac

  case "$wheel_name" in
    *macosx*x86_64.whl)
      macos_x86_64_count=$((macos_x86_64_count + 1))
      ;;
    *macosx*arm64.whl)
      macos_arm64_count=$((macos_arm64_count + 1))
      ;;
    *win_amd64.whl)
      windows_x86_64_count=$((windows_x86_64_count + 1))
      ;;
  esac
done

if [ "$linux_count" -ne 1 ]; then
  echo "Expected exactly 1 manylinux or musllinux wheel in $dist_dir, found $linux_count"
  exit 1
fi

if [ "$macos_x86_64_count" -ne 1 ]; then
  echo "Expected exactly 1 macOS x86_64 wheel in $dist_dir, found $macos_x86_64_count"
  exit 1
fi

if [ "$macos_arm64_count" -ne 1 ]; then
  echo "Expected exactly 1 macOS arm64 wheel in $dist_dir, found $macos_arm64_count"
  exit 1
fi

if [ "$windows_x86_64_count" -ne 1 ]; then
  echo "Expected exactly 1 Windows x86_64 wheel in $dist_dir, found $windows_x86_64_count"
  exit 1
fi
