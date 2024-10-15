#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Variables
REPO_NAME="YOUR_REPO_NAME"
GITHUB_USERNAME="YOUR_GITHUB_USERNAME"
GITHUB_TOKEN="YOUR_GITHUB_TOKEN"
GITHUB_REPO_URL="https://$GITHUB_USERNAME:$GITHUB_TOKEN@github.com/$GITHUB_USERNAME/$REPO_NAME.git"
OUTPUT_FOLDER="site/data/"
BENCHMARKS_FOLDER="benchmarks"
SWAY_TEST_FOLDER="YOUR_SWAY_TEST_FOLDER"
SWAY_FORC_LOCATION="YOUR_SWAY_FORC_LOCATION"
DYNO="YOUR_DYNO_LOCATION"

# Function to print error message and exit
function error_exit {
    echo "$1" 1>&2
    exit 1
}

# Run the dyno tool
"$DYNO" -t "$SWAY_TEST_FOLDER" -f "$SWAY_FORC_LOCATION" --flamegraph || error_exit "Failed to run dyno tool"

# Get the item in the benchmarks/runs folder
ITEM_PATH=$(find "$BENCHMARKS_FOLDER/runs" -type f -mindepth 1 -maxdepth 1 | head -n 1) || error_exit "Failed to get item in benchmarks/runs folder"
ITEM_NAME=$(basename "$ITEM_PATH" .json) || error_exit "Failed to get item name"

# Create a new local repository
mkdir "$REPO_NAME" || error_exit "Failed to create directory $REPO_NAME"


# Create the necessary folders
mkdir -p "$REPO_NAME/$OUTPUT_FOLDER/$ITEM_NAME" || error_exit "Failed to create output folder $OUTPUT_FOLDER/$ITEM_NAME"

# Move the benchmarks folder into the newly created folder
for item in "$BENCHMARKS_FOLDER"/*; do
    mv "$item" "$REPO_NAME/$OUTPUT_FOLDER/$ITEM_NAME" || error_exit "Failed to move $item to $OUTPUT_FOLDER/$ITEM_NAME"
done

# Remove the benchmarks folder
rm -rf "$BENCHMARKS_FOLDER" || error_exit "Failed to remove benchmarks folder"

# Enter the repository directory and initialize a git repository
cd "$REPO_NAME" || error_exit "Failed to change directory to $REPO_NAME"
git init || error_exit "Failed to initialize git repository"

# Create an empty README.md file
touch README.md || error_exit "Failed to create README.md file"

# Add all files to the repository
git add . || error_exit "Failed to add files to git repository"

# Commit the changes
git commit -m "Initial commit with dyno output and empty README.md" || error_exit "Failed to commit changes"

# Create the repository on GitHub
curl -u "$GITHUB_USERNAME:$GITHUB_TOKEN" https://api.github.com/user/repos -d "{\"name\":\"$REPO_NAME\"}" || error_exit "Failed to create GitHub repository"

# Push the local repository to GitHub
git remote add origin "$GITHUB_REPO_URL" || error_exit "Failed to add remote repository"
git push -u origin main || error_exit "Failed to push to remote repository"

echo "Repository has been created and pushed to GitHub at $GITHUB_REPO_URL"