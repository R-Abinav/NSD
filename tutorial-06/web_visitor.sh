#!/bin/bash

sites=(
    "https://www.reddit.com"
    "https://www.google.com"
    "https://www.github.com"
    "https://www.youtube.com"
    "https://www.wikipedia.org"
    "https://www.amazon.com"
    "https://www.twitter.com"
    "https://www.linkedin.com"
    "https://www.netflix.com"
    "https://www.apple.com"
    "https://www.microsoft.com"
    "https://www.stackoverflow.com"
    "https://www.cloudflare.com"
)

for site in "${sites[@]}"; do
    echo "hitting $site"
    curl -s -o /dev/null "$site"
done

echo "done"