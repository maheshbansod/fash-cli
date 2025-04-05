# agent-base

A Rust-based agent for interacting with the Gemini API.

## Overview


This project is a Rust-based agent that leverages the Gemini API to perform various tasks. It's designed as a command-line application, allowing users to specify tasks and interact with the Gemini API through a simple interface. The agent handles configuration and interacts with the Gemini API.


## Functionality

The agent's primary functionality is to execute tasks using the Gemini API. The specific tasks that can be performed depend on the implementation within the `agent` and `gemini` modules. The application takes a task as an argument, and then executes it using the agent.

## Getting Started

### Prerequisites

*   Rust toolchain (including Cargo)
*   A Gemini API key

### Installation

1.  Clone the repository:

    ```bash
    git clone &lt;repository_url&gt;
    cd agent-base
    ```

2.  Build the project:

    ```bash
    cargo build --release
    ```

### Configuration

1.  Set the `GEMINI_API_KEY` environment variable. You can do this by creating a `.env` file in the project root directory with the following content:

    ```
    GEMINI_API_KEY=&lt;your_gemini_api_key&gt;
    ```

    Replace `&lt;your_gemini_api_key&gt;` with your actual Gemini API key.

### Usage

The agent is a command-line application. To run it, use the following command:

```bash
./target/release/agent-base &lt;task&gt;
```

Replace `&lt;task&gt;` with the desired task to execute. You can use the `--help` flag to see the usage instructions.

```bash
./target/release/agent-base --help
```


