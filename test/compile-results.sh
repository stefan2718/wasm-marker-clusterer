#!/bin/bash

cd $1

for file in `ls`; do
  awk "/$2/ { sum += \$2; count += 1 } END { print sum / count }" $file
done | tr '\n' ','
echo
