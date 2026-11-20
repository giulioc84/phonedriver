---
title: How to use phonedriver with VS Code and GitHub Copilot
seo: How to let GitHub Copilot in VS Code control a phone over MCP
nav: VS Code
description: Add phonedriver to VS Code as an MCP server with one command or a .vscode/mcp.json entry, so GitHub Copilot can operate an Android phone or iPhone.
updated: 2026-11-20
date: 2026-11-14
---

VS Code can start phonedriver as an MCP server, which gives GitHub Copilot's chat a phone to operate. I test phonedriver with Claude Code and have not run a VS Code session myself. The steps below are this project's own command and the file format from [VS Code's MCP configuration reference](https://code.visualstudio.com/docs/copilot/reference/mcp-configuration), read on 14 November 2026.

## Add the server

```sh
code --add-mcp '{"name":"phonedriver","command":"npx","args":["-y","phonedriver","mcp"]}'
```

The first start downloads the binary once (see [Install](/docs/install/)). It needs Node 18 or later.

## Or add it to one workspace

Put this in `.vscode/mcp.json`. In VS Code the top key is `servers`, where most other clients use `mcpServers`.

```json
{
  "servers": {
    "phonedriver": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "phonedriver", "mcp"],
      "env": { "PHONEDRIVER_DEVICE": "android:emulator-5554" }
    }
  }
}
```

To keep the agent in your own app on a phone with real accounts, add `"PHONEDRIVER_APP_SCOPE": "com.example.app"` to `env` ([what it covers](/docs/android-phone/#keep-the-agent-in-your-app)). To let the agent clear, remove or change the permissions of an app you are developing on a physical Android phone (an app Gradle installed, not phonedriver), add `"PHONEDRIVER_APPS_UNDER_TEST": "com.example.app"` to `env`, and run `phonedriver daemon stop` once if a daemon is running.

To keep a phone out of the agent's reach on a shared adb server, add `"PHONEDRIVER_DENY_DEVICES": "android:<id>"` to `env` (or write `deny android:<id>` in `~/.phonedriver/devices`, which every agent on the machine obeys); a denied device is not listed, not probed by `doctor` and never gets the agent.

`PHONEDRIVER_DEVICE` is optional when one device is connected. `phonedriver devices` lists the ids. On Windows, if VS Code cannot start `npx` directly, use `"command": "cmd"` with `"args": ["/c", "npx", "-y", "phonedriver", "mcp"]`.

## Connect a device and ask

Start an [Android emulator](/docs/android-emulator/), plug in an [Android phone](/docs/android-phone/) with USB debugging on, or boot the [iOS Simulator](/docs/ios-simulator/). Run `npx -y phonedriver doctor` in the terminal: each line says `ok`, `warn` or `fail`, and a fix follows every line that is not `ok`. Then ask Copilot's chat:

> Open Settings and turn on dark mode.

The server has eighteen tools: `devices`, `apps`, `snapshot`, `screenshot`, `tap`, `type`, `swipe`, `scroll`, `press`, `launch`, `wait`, `batch`, `clipboard`, `shell`, `setting`, `logs`, `network` and `replay`. `snapshot`, `screenshot`, `wait`, `logs` and `network` are marked read-only.

## Approval prompts

The [approval guard](/docs/approval-guard/) asks you before a tap on Pay, Place order or Delete account, through MCP elicitation. VS Code's documentation describes servers asking the user for input, and I have not seen phonedriver's prompt in VS Code. If the question cannot be shown, the tool replies `Needs the human's approval; nothing was tapped`. Check which of the two you get before you give an agent a phone with a shopping app on it.

## Questions

### Does phonedriver work with GitHub Copilot?

It works with any client that starts a stdio MCP server, and VS Code is one. Copilot's chat in VS Code picks from the tools of the MCP servers you have added. I have not run that session myself, so [open an issue]({{REPO}}/issues) if a step here is wrong.

### Where does the entry live?

A `.vscode/mcp.json` file belongs to one workspace and can be committed with the project. VS Code's `MCP: Add Server` command walks you through adding a server for yourself or for the workspace.

### Can I use the phonedriver CLI from the VS Code terminal instead?

Yes. `npx -y phonedriver snapshot` and `npx -y phonedriver tap @e4` print the same text as the MCP tools, so an agent with a terminal and no MCP can use them.
