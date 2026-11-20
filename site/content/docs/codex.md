---
title: How to use phonedriver with Codex
seo: How to let Codex control an Android phone or iPhone
nav: Codex
description: Add phonedriver to OpenAI Codex as an MCP server with one command, a config.toml entry or a plugin, pick a device, and ask Codex to operate a phone.
updated: 2026-11-20
date: 2026-11-14
---

Codex can operate an Android phone or an iPhone through phonedriver. Run `codex mcp add phonedriver -- npx -y phonedriver mcp`, connect a device, and ask in plain language.

phonedriver is tested with Claude Code. In Codex 0.167.0 I installed the plugin and saw its MCP server and its skill load. The benchmarks on this site were not run in Codex.

## Add the server

With Node 18 or later and nothing installed first:

```sh
codex mcp add phonedriver -- npx -y phonedriver mcp
codex mcp list
```

Inside a Codex session, `/mcp` shows the active servers. The first start downloads the binary once (see [Install](/docs/install/)). If you [installed phonedriver](/docs/install/#permanent-install), use `codex mcp add phonedriver -- phonedriver mcp`.

With more than one device connected, name the one to use. `phonedriver devices` lists the ids.

```sh
codex mcp add phonedriver --env PHONEDRIVER_DEVICE=android:emulator-5554 -- npx -y phonedriver mcp
```

To keep the agent in your own app on a phone with real accounts, add `--env PHONEDRIVER_APP_SCOPE=com.example.app` ([what it covers](/docs/android-phone/#keep-the-agent-in-your-app)). To let the agent clear, remove or change the permissions of an app you are developing on a physical Android phone (the apps Gradle installed, not phonedriver), add `--env PHONEDRIVER_APPS_UNDER_TEST=com.example.app` the same way, and run `phonedriver daemon stop` once if a daemon is running.

To keep a phone out of the agent's reach on a shared adb server, add `--env PHONEDRIVER_DENY_DEVICES=android:<id>` the same way, or write `deny android:<id>` in `~/.phonedriver/devices`, which every agent on the machine obeys. A denied device is not listed, not probed by `doctor` and never gets the agent.

## Or edit config.toml

Codex keeps its MCP servers in `~/.codex/config.toml`, or in `.codex/config.toml` inside a trusted project. The same server as a table:

```toml
[mcp_servers.phonedriver]
command = "npx"
args = ["-y", "phonedriver", "mcp"]

[mcp_servers.phonedriver.env]
PHONEDRIVER_DEVICE = "android:emulator-5554"
```

Leave the `env` table out when one device is connected. The command and the file format are from [Codex's MCP documentation](https://developers.openai.com/codex/mcp), read on 14 November 2026.

## Or install the plugin

The plugin adds the same server and the phonedriver skill, so Codex has the instructions without you pasting them. Use the plugin or `mcp add`, not both, or the tools are listed twice.

```sh
codex plugin marketplace add hridaya423/phonedriver
codex plugin add phonedriver@phonedriver
```

## Ask it

Start an [Android emulator](/docs/android-emulator/), plug in an [Android phone](/docs/android-phone/) with USB debugging on, or boot the [iOS Simulator](/docs/ios-simulator/). Then ask:

> Open Settings and turn on dark mode.

Codex gets the screen as a short numbered list and taps by number. Each action waits for the screen to settle and returns only the lines that changed:

```screen
ok tapped @e10 "Dark theme"
~ @e10 switch "Dark theme" off → on
```

## Approval prompts in Codex

The [approval guard](/docs/approval-guard/) holds a tap on Pay, Place order or Delete account and asks you through MCP elicitation. Codex's MCP documentation does not say whether Codex supports elicitation, and I have not tried the prompt there. In a client that cannot show the question, the tool replies `Needs the human's approval; nothing was tapped`, and the agent has to ask you in chat. Read [what the guard does not catch](/docs/approval-guard/#what-it-does-not-catch) before you give an agent a phone with a payment app on it.

## Without MCP

An agent with a shell can run the same commands as a CLI, and the text is the same as the MCP tools return:

```sh
npx -y phonedriver snapshot
npx -y phonedriver tap @e4
```

`npx -y phonedriver skill` prints the instructions an agent should read first.

## Questions

### Can Codex control a real iPhone?

Yes, from a Mac with Xcode. Plug the iPhone in, run `phonedriver ios setup` once, and Codex uses it through the same MCP server. `phonedriver ios wifi on` lets you unplug the cable afterwards. See [the iPhone guide](/docs/iphone/).

### Does Codex need the phonedriver skill?

It helps. The MCP server carries a short version of the instructions, and the skill holds the full strategy, such as sending a path of taps in one call. The plugin installs the skill. With `codex mcp add`, `npx -y phonedriver skill` prints it.

### How many tokens do the tools cost?

In Claude Code, the first turn of a one-word `claude -p` costs 5,606 tokens with phonedriver's 14 tools attached. I have not measured the same turn in Codex. See [the token benchmark](/benchmarks/#tokens).
