# Dyno Site

## Description

`Dynosite` is a static html page generator. It works together with [`dyno`](https://github.com/ourovoros-io/dyno.git) tool to create on demand a static html page.

> [!TIP]
>
> The tool is designed to work with `CI/Github` but it can also work locally generating the website under `/site/index.html`.

## Usage

```bash
Fuel Dynosite Profiler Site Generator

Usage: dynosite --benchmarks-folder <BENCHMARKS_FOLDER> --pr-hash <PR_HASH> --pr-title <PR_TITLE> --pr-link <PR_LINK>

Options:
  -b, --benchmarks-folder <BENCHMARKS_FOLDER>  The folder containing the benchmarks
  -p, --pr-hash <PR_HASH>                      The PR hash
  -t, --pr-title <PR_TITLE>                    The pr title
  -l, --pr-link <PR_LINK>                      The pr link
  -h, --help                                   Print help
  -V, --version                                Print version
```
