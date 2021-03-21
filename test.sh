#!/bin/bash
# Tests `chacha20`
# 
# Usage: ./test.sh [debug|release]
# The binary must have already been built (use `cargo build [--release]` first)

mkdir -p test || exit -1
cd test || exit -1

PROG=../target/${1:-release}/chacha20

if [[ ! -f $PROG ]]; then
	echo "Couldn't find executable \"$PROG\""
	exit -1
else
	echo "Testing executable \"$PROG\""
	echo ""
fi

INPUT_SIZE=4096

echo ">>> Generating random input ($INPUT_SIZE bytes)"
dd if=/dev/urandom "bs=$INPUT_SIZE" count=1 | base64 > test.txt

echo ">>> Generating key + IV"
KEYS=$($PROG k)
echo "$KEYS"

echo ">>> Encrypting"
time $PROG e $KEYS  < test.txt  > test.cc20    || exit 1
echo ">>> Decrypting"
time $PROG d $KEYS  < test.cc20 > test.out.txt || exit 2

echo ">>> Comparing"
echo "Input (SHA256 sum):	$(sha256sum test.txt)"
echo "Encrypted (SHA256 sum):	$(sha256sum test.cc20)"
echo "Output (SHA256 sum):	$(sha256sum test.out.txt)"

echo "---"
if cmp --silent -- test.txt test.out.txt; then
	echo "Pass!"
else
	echo "Failed"
	exit 3
fi

rm test.*

cd ..
rmdir test || exit -1
