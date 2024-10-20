# Dyno Site

## Description

`Dynosite` is a static html page generator. It works together with [`dyno`](https://github.com/ourovoros-io/dyno.git) tool to create on demand a static html page with the execution data.

> [!TIP]
>
> The tool is designed to work with `CI/Github` but it can also work locally generating the website under `/site/index.html`.

## Usage

```bash
Fuel Dynosite Profiler Site Generator

Usage: dynosite [OPTIONS] --benchmarks-folder <BENCHMARKS_FOLDER>

Options:
  -b, --benchmarks-folder <BENCHMARKS_FOLDER>  The target folder containing the benchmarks
  -d, --data-only                              Data only mode
  -s, --site-name <SITE_NAME>                  The site name (Optional)
  -p, --pr-hash <PR_HASH>                      The PR hash (Optional)
  -t, --pr-title <PR_TITLE>                    The PR title (Optional)
  -l, --pr-link <PR_LINK>                      The PR link (Optional)
  -h, --help                                   Print help
  -V, --version                                Print version
```

### Data Only

The data only mode will skip the generation of flamegraphs and plots for the benchmarking data.

### CI

Steps to setup the CI:

- Rename [example_github_setup.sh](./example_github_setup.sh) to `github_setup.sh`.
- Fill up the needed details in [example_github_setup.sh](./example_github_setup.sh).
- Run the `github_setup.sh`. This will create a new repository for the site and the data.
- Rename [example_ci_setup.yml](./example_ci_setup.yml) to `ci_setup.yml` or any other name you want.
- Setup the needed github environment variables.
- Copy the `ci_setup.yml` to the sway repository under the .github workflow folder.
- Create a PR in the sway repository.
- Allow the action to execute.
- Open the README.md in the repository that was created to store the site data.

### Standalone

- Clone the `sway` repository.

```bash
git clone https://github.com/FuelLabs/sway.git
```

```bash
git clone https://github.com/FuelLabs/sway.git sway_modified
```

- Add changes to sway modified.
- Compile original and modified version of `sway` with the `profiler` feature.

```bash
cargo b --release --features profiler
```

- Run the `dyno` pointing to the original version of `sway`.

```bash
cargo r -- -t ../sway/test/src/sdk-harness/test_projects/hashing -f ../sway/target/release/forc --flamegraph --print-output
```

- Run the `dyno` pointing to the modified version of `sway`.

> [!TIP]
>
> The folder sway_modified is the modified version.

```bash
cargo r -- -t ../sway/test/src/sdk-harness/test_projects/hashing -f ../sway_modified/target/release/forc --flamegraph --print-output
```

- Run `dynosite` pointing at the benchmarks folder of `dyno`.

> [!TIP]
> In standalone mode all options are optional. Only the -b is required to point to the benchmarks folder of dyno.

```bash
cargo r -- -b ../dyno/benchmarks -s MODIFIED
```

- Open the `index.html` under `site/` to check the results.

## Use with AWS

Step 1: Create an S3 Bucket

- Sign in to the AWS Management Console.

- Open the S3 Console: From the AWS Management Console, navigate to the S3 service.

Create a Bucket:

- Click on the "Create bucket" button.

- Enter a unique bucket name.

- Choose the AWS region where you want to create the bucket.

- Click "Create bucket" at the bottom of the page.

Step 2: Configure the Bucket for Static Website Hosting

- Open the Bucket: Click on the bucket name you just created.

- Enable Static Website Hosting:

- Go to the "Properties" tab.

- Scroll down to the "Static website hosting" section.

- Click "Edit".

- Select "Enable".

- For "Index document", enter index.html.

- For "Error document", enter error.html (optional).

- Click "Save changes".

Step 3: Set Bucket Policy for Public Access

- Go to the "Permissions" tab of your bucket.

- Edit Bucket Policy:

- Click on "Bucket policy".

Add the following JSON policy to allow public read access to your bucket:

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "PublicReadGetObject",
            "Effect": "Allow",
            "Principal": "*",
            "Action": "s3:GetObject",
            "Resource": "arn:aws:s3:::dynosite/*"
        }
    ]
}
```

- Click "Save changes".

Step 4: Get your public AWS website url.

Get the Website URL:

- Go back to the "Properties" tab.

- Scroll down to the "Static website hosting" section.

- You will see the "Bucket website endpoint" URL. This is the URL where your static website is hosted.

Step 5: Set the action secrets in github.

```json
{
  AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
  AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
  AWS_REGION: ${{ secrets.AWS_REGION }}
  S3_BUCKET: ${{ secrets.S3_BUCKET }}
}
```
