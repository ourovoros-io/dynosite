#!/bin/bash

# Variables
REPO_NAME="YOUR_REPO_NAME"
GITHUB_USERNAME="YOUR_GITHUB_USERNAME"
GITHUB_TOKEN="YOUR_GITHUB_TOKEN"
GITHUB_REPO_URL="https://github.com/$GITHUB_USERNAME/$REPO_NAME.git"
OUTPUT_FOLDER="site/data/runs"
BENCHMARKS_FOLDER="../dyno/benchmarks"
SWAY_TEST="../sway/test/src/sdk-harness/test_projects/"
SWAY_FORC="../sway/target/release/forc"


# Create a new local repository
mkdir "$REPO_NAME"
cd "$REPO_NAME" || exit
git init

# Create the necessary folders
mkdir -p "$OUTPUT_FOLDER"

# Create an empty README.md file
touch README.md

# Run the dynosite tool
dyno cargo r -- -t "$SWAY_TEST" -f "$SWAY_FORC" --flamegraph 

# Move the output to the specified output folder
mv "$BENCHMARKS_FOLDER"/* "$OUTPUT_FOLDER"

# Add all files to the repository
git add .

# Commit the changes
git commit -m "Initial commit with dyno output and empty README.md"

# Create the repository on GitHub
curl -u "$GITHUB_USERNAME:$GITHUB_TOKEN" https://api.github.com/user/repos -d "{\"name\":\"$REPO_NAME\"}"

# Push the local repository to GitHub
git remote add origin "$GITHUB_REPO_URL"
git push -u origin master

echo "Repository has been created and pushed to GitHub at $GITHUB_REPO_URL"