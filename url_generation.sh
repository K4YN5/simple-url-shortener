#!/bin/bash
count=100000 # Number of unique URLs to generate
output_file="urls.txt"
base_url="http://example.com/page?id="

rm -f "$output_file" # Clear file if exists
for (( i=1; i<=count; i++ ))
do
  echo "\"${base_url}${i}\"" >> "$output_file"
done
echo "Generated $count unique URLs in $output_file"
