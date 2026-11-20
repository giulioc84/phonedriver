---
title: How to use phonedriver with Cursor and other MCP clients
seo: How to let Cursor, Gemini CLI, Zed or Cline control a phone
nav: Cursor and other clients
description: Add phonedriver to Cursor, Gemini CLI, Zed, Cline or any MCP client that starts a stdio server, with the entry for each, a device setting and the tools.
updated: 2026-11-20
---

Any client that starts a stdio MCP server can use phonedriver: the command is `npx` with the arguments `-y phonedriver mcp`. [Codex](/docs/codex/) and [VS Code](/docs/vs-code/) have their own pages.

I test with Claude Code. I have not run a session in Cursor, Gemini CLI, Zed or Cline, so the steps below come from each client's own documentation, read on 14 November 2026, and not from something I watched work.

## The entry

```json
{
  "mcpServers": {
    "phonedriver": {
      "command": "npx",
      "args": ["-y", "phonedriver", "mcp"],
      "env": { "PHONEDRIVER_DEVICE": "android:emulator-5554" }
    }
  }
}
```

To keep the agent in your own app on a phone with real accounts, add `"PHONEDRIVER_APP_SCOPE": "com.example.app"` to `env` ([what it covers](/docs/android-phone/#keep-the-agent-in-your-app)). To let the agent clear, remove or change the permissions of an app you are developing on a physical Android phone (an app Gradle installed, not phonedriver), add `"PHONEDRIVER_APPS_UNDER_TEST": "com.example.app,com.example.other"` to `env`, and run `phonedriver daemon stop` once if a daemon is running.

To keep a phone out of the agent's reach on a shared adb server, add `"PHONEDRIVER_DENY_DEVICES": "android:<id>"` to `env` (or write `deny android:<id>` in `~/.phonedriver/devices`, which every agent on the machine obeys); a denied device is not listed, not probed by `doctor` and never gets the agent.

The first start downloads the binary once. On Windows, some clients cannot start `npx` directly: use `"command": "cmd"` and `"args": ["/c", "npx", "-y", "phonedriver", "mcp"]`. If you installed phonedriver yourself, use `"command": "phonedriver"` with `"args": ["mcp"]`, or the full path. `PHONEDRIVER_DEVICE` is optional when one device is connected.

## Cursor

Put the entry in `~/.cursor/mcp.json` for every project, or in `.cursor/mcp.json` inside one project. Open Cursor's MCP settings and check that `phonedriver` shows its tools. [Cursor's MCP documentation](https://cursor.com/docs/context/mcp) lists elicitation as supported, which is what the approval prompt uses.

## Gemini CLI

Add the entry at the top of this page to `~/.gemini/settings.json`, or to `.gemini/settings.json` inside one project. It uses the same `mcpServers` key. `gemini mcp list`, or `/mcp` inside a session, shows the server. Source: [Gemini CLI's MCP documentation](https://geminicli.com/docs/tools/mcp-server/).

## Zed

Zed calls MCP servers context servers. Add this to Zed's settings file (`zed: open settings file`). Source: [Zed's MCP documentation](https://zed.dev/docs/ai/mcp), which says Zed supports MCP tools and prompts and not yet elicitation, so expect the approval guard to reply that a tap needs your approval instead of showing a prompt.

```json
{
  "context_servers": {
    "phonedriver": {
      "command": "npx",
      "args": ["-y", "phonedriver", "mcp"],
      "env": {}
    }
  }
}
```

## Cline

Open the MCP Servers panel, choose Configure, then Configure MCP Servers. That opens `cline_mcp_settings.json`, which takes the entry at the top of this page as it is. Source: [Cline's MCP documentation](https://docs.cline.bot/mcp/configuring-mcp-servers).

## Other clients

Use the client's own place for stdio servers, with the same `command` and `args`. If the client cannot start a local process, run `phonedriver` where the client can reach it and forward the daemon's socket (see [Android emulator](/docs/android-emulator/#use-a-device-on-another-machine)).

## Tools

The server has eighteen tools: `devices`, `apps`, `snapshot`, `screenshot`, `tap`, `type`, `swipe`, `scroll`, `press`, `launch`, `wait`, `batch`, `clipboard`, `shell`, `setting`, `logs`, `network` and `replay`. `apps` also installs, stops, clears and uninstalls. Each prints the same text as the CLI verb of the same name, so anything you read in these docs about the command line holds for the tools.

In Claude Code's wrapper a tool costs about 51 tokens whatever its text. Measured when the server had fourteen tools, the first turn of a one-word `claude -p` cost 5,606 tokens with phonedriver attached ([token benchmark](/benchmarks/#tokens)).

## Approval prompts need elicitation

The [approval guard](/docs/approval-guard/) asks you through MCP elicitation. A client that does not support elicitation cannot show the question, and the tool then replies that the tap needs the human's approval and taps nothing. Check that your client supports elicitation before you give an agent a phone with a shopping app on it.

## Questions

### Which MCP clients does phonedriver work with?

Any client that can start a local process and speak MCP over stdio. Claude Code is the one I test against. Codex 0.167.0 installed the plugin with its server and skill. For Cursor, VS Code, Gemini CLI, Zed and Cline I wrote the entry from the client's documentation and have not run a session.

### What is the MCP config for phonedriver?

A stdio server with the command `npx` and the arguments `-y`, `phonedriver`, `mcp`. Most clients take it under an `mcpServers` key in a JSON file. VS Code uses `servers`, Zed uses `context_servers`, and Codex uses a `[mcp_servers.phonedriver]` table in `config.toml`.

### Does the approval prompt work in every client?

No. It needs MCP elicitation. Claude Code supports it. Cursor's documentation lists it as supported, and Zed's says it is not supported yet. Where the prompt cannot be shown, phonedriver taps nothing and tells the agent the tap needs your approval.
