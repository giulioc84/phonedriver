---
title: How to use phonedriver with Claude Code
seo: How to let Claude Code control an Android phone or iPhone
nav: Claude Code
description: Add phonedriver to Claude Code as an MCP server with one command, point it at a device and ask it to operate a phone in plain language.
updated: 2026-11-20
---

To let Claude Code control an Android phone or an iPhone, add phonedriver as an MCP server with `claude mcp add phonedriver -- npx -y phonedriver mcp`, connect a device and ask in plain language. Claude Code is the client phonedriver is tested against: the benchmarks on this site run `claude -p` with Claude Sonnet 5.5.

## Add the server

From any directory, with Node 18 or later and nothing installed first:

```sh
claude mcp add phonedriver -- npx -y phonedriver mcp
claude mcp list    # expect: phonedriver ... Connected
```

The first start downloads the binary once (see [Install](/docs/install/)). If you [installed phonedriver](/docs/install/#permanent-install) instead, use `claude mcp add phonedriver -- phonedriver mcp`, or the full path to the binary after the `--`.

For the first run, boot one simulator or connect one device, so no device setting is needed. With more than one device, set the one to use in the server's environment:

```sh
claude mcp add --env PHONEDRIVER_DEVICE=android:emulator-5554 phonedriver -- npx -y phonedriver mcp
```

`phonedriver devices` lists the ids.

## An app you are developing

On a physical Android phone, `apps clear`, `uninstall`, `grant` and `revoke` touch only apps phonedriver installed, so an app that Gradle or Android Studio put there is refused. Name your own apps in the server's environment, which the agent cannot change:

```sh
claude mcp add --env PHONEDRIVER_APPS_UNDER_TEST=com.example.app,com.example.other phonedriver -- npx -y phonedriver mcp
```

To keep a phone out of the agent's reach on a shared adb server, add `--env PHONEDRIVER_DENY_DEVICES=android:<id>` the same way (or write `deny android:<id>` in `~/.phonedriver/devices`, which every agent on the machine obeys). A denied device is not listed, not probed by `doctor` and never gets the agent.

The daemon reads the list when it starts, so run `phonedriver daemon stop` once if one is already running. Emulators and Simulators need none of this.

## Keep the agent in your app

On a phone that is signed in to real accounts, name the apps the agent may use. It then sees nothing from any other app, and its taps, typing, screenshots and `shell` are refused while another app is on screen. The home screen counts as another app.

```sh
claude mcp add --env PHONEDRIVER_APP_SCOPE=com.example.app phonedriver -- npx -y phonedriver mcp
```

This is not `PHONEDRIVER_APPS_UNDER_TEST`, which only lets the agent clear or uninstall an app of yours. The scope is read when the daemon starts, so run `phonedriver daemon stop` once if one is running, and the agent cannot change it. It is not a sandbox: [what it covers and what it does not](/docs/android-phone/#keep-the-agent-in-your-app).

## Or install the plugin

The plugin adds the same server and the phonedriver skill, so Claude Code has the full instructions without you pasting them. Remove a server you added with `claude mcp add` first (`claude mcp remove phonedriver`), or the tools are listed twice.

```sh
claude plugin marketplace add hridaya423/phonedriver
claude plugin install phonedriver@phonedriver
```

Inside a session the same two steps are `/plugin marketplace add hridaya423/phonedriver` and `/plugin install phonedriver@phonedriver`. The tools are then named `mcp__plugin_phonedriver_phonedriver__tap` and so on, and the skill is `phonedriver:phonedriver`. To choose a device, set `PHONEDRIVER_DEVICE` in the environment you start Claude Code from.

## Ask it

Boot a device, then ask Claude Code in plain language:

> Open Settings and turn on dark mode.

Claude Code calls the tools, for example `launch`, `swipe` and `tap`. Each reply is plain text. The tap that turns the switch on returns this on the iOS Simulator:

```screen
ok tapped @e41 "Dark Appearance"
- @e65 @e66 @e67
~ @e41 switch "Dark Appearance" off → on
+ @e68 button "L4S"
```

`+`, `-` and `~` mark lines that appeared, left and changed. The agent does not poll or sleep: every action waits for the screen to settle.

## What the agent gets

Eighteen tools: `devices`, `apps`, `snapshot`, `screenshot`, `tap`, `type`, `swipe`, `scroll`, `press`, `launch`, `wait`, `batch`, `clipboard`, `shell`, `setting`, `logs`, `network` and `replay`. Each prints the same text as the CLI verb of the same name. `phonedriver skill` prints `SKILL.md`, the instructions an agent reads before it uses the tools; the MCP server carries a short version too.

## When Claude Code asks you something

A tap on a button that spends money or cannot be undone (`Place order`, `Pay`, `Delete account`) is held until you answer. Claude Code shows the question with the app, the button and the amounts on screen. See [the approval guard](/docs/approval-guard/). In headless `claude -p` nobody can answer, so nothing is tapped.

## Secrets

Text typed with `secret: true` over MCP never reaches logs, traces or replies. Plain `type` into a password field or a one-time code field is refused, and the error tells the agent to get the text from you and pass it with `secret`.

## Questions

### Can Claude Code control an iPhone?

Yes, from a Mac with Xcode. The iOS Simulator works once it is booted. For a real iPhone or iPad, plug it in and run `phonedriver ios setup` once, and `phonedriver ios wifi on` lets you unplug the cable. See [the iPhone guide](/docs/iphone/).

### Does Claude Code need screenshots to use the phone?

No. It reads the screen as a short numbered text list and taps by number, and each action returns only the lines that changed. It takes a screenshot only when it needs to see a picture, such as a map or a photo.

### How many tokens does phonedriver add to Claude Code?

The first turn of a one-word `claude -p` costs 5,606 tokens with phonedriver's 14 tools attached. On nine real phone jobs with Claude Sonnet 5.5 on one Android emulator, the per-job medians summed to 657,542 tokens (10 November 2026). See [the benchmarks](/benchmarks/#jobs).

### Does it work in a headless claude -p run?

Yes. The benchmarks on this site ran that way. Nobody can answer an approval prompt in a headless run, so a tap on Pay or Place order gets no answer and nothing is tapped.

## Next

Connect a device: an [Android emulator](/docs/android-emulator/), an [Android phone](/docs/android-phone/), the [iOS Simulator](/docs/ios-simulator/) or an [iPhone](/docs/iphone/). Then try [testing your own app](/docs/test-a-mobile-app/), or read how it compares with [mobile-mcp](/compare/mobile-mcp/) and [adb](/compare/adb/).
