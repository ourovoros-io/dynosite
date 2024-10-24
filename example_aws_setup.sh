#!/bin/bash

# Exit immediately if a command exits with a non-zero status
set -e

# Variables
# The name of the S3 bucket
S3_BUCKET="YOUR_S3_BUCKET_NAME"
# The output folder where the dyno tool stores the data
OUTPUT_FOLDER="site/data/"
# The folder where the benchmarks are stored by dyno
BENCHMARKS_FOLDER="benchmarks"
# The folder where the sway tests are located
SWAY_TEST_FOLDER="YOUR_SWAY_TEST_FOLDER"
# The location of the sway forc file
SWAY_FORC_LOCATION="YOUR_SWAY_FORC_LOCATION"
# The location of the dyno file
DYNO="YOUR_DYNO_LOCATION"

# Function to print error message and exit
function error_exit {
    echo "$1" 1>&2
    exit 1
}

# Run the dyno tool
"$DYNO" -t "$SWAY_TEST_FOLDER" -f "$SWAY_FORC_LOCATION" --flamegraph || error_exit "Failed to run dyno tool"

# Wait for 5 seconds
sleep 5

# Run it twice to get the benchmarks folder structure with some initial data for stats
"$DYNO" -t "$SWAY_TEST_FOLDER" -f "$SWAY_FORC_LOCATION" --flamegraph || error_exit "Failed to run dyno tool"

# Get the item in the benchmarks/stats folder
ITEM_PATH=$(find "$BENCHMARKS_FOLDER/stats" -type f -mindepth 1 -maxdepth 1 | head -n 1) || error_exit "Failed to get item in benchmarks/stats folder"
ITEM_NAME=$(basename "$ITEM_PATH" .json) || error_exit "Failed to get item name"

# Create the necessary folders
mkdir -p "$OUTPUT_FOLDER/$ITEM_NAME" || error_exit "Failed to create output folder $OUTPUT_FOLDER/$ITEM_NAME"

# Move the benchmarks folder into the newly created folder
for item in "$BENCHMARKS_FOLDER"/*; do
    mv "$item" "$OUTPUT_FOLDER/$ITEM_NAME" || error_exit "Failed to move $item to $OUTPUT_FOLDER/$ITEM_NAME"
done

# Remove the benchmarks folder
rm -rf "$BENCHMARKS_FOLDER" || error_exit "Failed to remove benchmarks folder"

# Install AWS CLI if not already installed
if ! command -v aws &> /dev/null; then
    echo "AWS CLI not found, installing..."
    curl "https://awscli.amazonaws.com/AWSCLIV2.pkg" -o "AWSCLIV2.pkg"
    sudo installer -pkg AWSCLIV2.pkg -target /
fi

# Configure AWS CLI (ensure AWS credentials are set in the environment or use aws configure)
aws configure set aws_access_key_id YOUR_AWS_ACCESS_KEY_ID
aws configure set aws_secret_access_key YOUR_AWS_SECRET_ACCESS_KEY
aws configure set region YOUR_AWS_REGION

# Upload the data to S3
aws s3 sync "$OUTPUT_FOLDER" "s3://$S3_BUCKET/$OUTPUT_FOLDER" || error_exit "Failed to upload data to S3"

echo "Data has been uploaded to S3 bucket at s3://$S3_BUCKET/$OUTPUT_FOLDER"