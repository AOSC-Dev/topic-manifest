# AOSC OS APT Topic Generator

This is the AOSC OS APT topic generator. It generates the topics metadata for the binary packages in JSON format.
The generated format is documented at https://app.swaggerhub.com/apis-docs/liushuyu/RepoTopicManager/1.0.0.

This project is part of the AOSC infrastructures.

## Building

Just run `cargo build --release` and wait.

## Usage

Run `./topic-manifest -d <path/to/debs/root>` to start.
