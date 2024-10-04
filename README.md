# Dyno Site

## Description

Dyno site is a static html page generator that handles the creation on demand of a fresh version with the latest data from the `dyno` tool that is running in the CI in Github.

## Usage

```bash
Fuel Dyno Profiler Site Generator

Usage: dynosite --benchmarks-folder <BENCHMARKS_FOLDER> --pr-hash <PR_HASH> --pr-title <PR_TITLE> --pr-link <PR_LINK>

Options:
  -b, --benchmarks-folder <BENCHMARKS_FOLDER>  The folder containing the benchmarks
  -p, --pr-hash <PR_HASH>                      The PR hash
  -t, --pr-title <PR_TITLE>                    The pr title
  -l, --pr-link <PR_LINK>                      The pr link
  -h, --help                                   Print help
  -V, --version                                Print version
```
