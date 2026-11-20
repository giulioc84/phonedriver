---
title: How to have an AI agent test your mobile app
seo: How to have Claude Code test an Android or iOS app on a device
nav: Test your app
description: Install a build on an emulator, the iOS Simulator or a phone, and ask Claude Code or Codex to use the app and report bugs with steps. What was measured, and notes for React Native, Expo and Flutter.
meta: Install a build on an emulator, simulator or phone and ask Claude Code or Codex to use your app and report bugs with steps. What was measured and what was not.
updated: 2026-11-20
date: 2026-11-14
---

An agent with phonedriver can install your build, open it, use it the way a person would and write up what breaks, with steps. On my benchmark, Claude Sonnet 5.5 found both planted bugs in a testbed app's sign-up form in 3 of 3 runs, and both in its shop screen in 3 of 3, on one Android emulator on 10 November 2026. That is two jobs on an app I wrote, so read it as what happened and not as a rate.

## Install the build

Connect a device first: an [Android emulator](/docs/android-emulator/), an [Android phone](/docs/android-phone/) or the [iOS Simulator](/docs/ios-simulator/). Then install and open your app:

```sh
phonedriver apps install app-debug.apk
phonedriver launch "My App" --restart
```

On Android the file is an APK. On the iOS Simulator it is the `.app` folder your build produces. Installing on a physical iPhone is not implemented, so install there with Xcode and let the agent open the app. The path is read on the machine that runs the daemon. Over MCP the same actions are the `apps` and `launch` tools, so the agent can do this step itself.

On a phone with real accounts on it, set `PHONEDRIVER_APP_SCOPE=com.example.app` in the MCP server's `env` so the agent reads and touches only your app ([what it covers](/docs/android-phone/#keep-the-agent-in-your-app)).

Three options that help a test run:

- `phonedriver launch "My App" --grant notifications,camera` grants runtime permissions first, so their dialogs never appear.
- `phonedriver launch "My App" --restart` stops the app first, so it opens on its first screen.
- `phonedriver apps clear "My App"` wipes the app's data, for a first-run test. On a physical device this works only for apps phonedriver installed, and for the packages you list in `PHONEDRIVER_APPS_UNDER_TEST` (for an app Gradle or Android Studio installed; set it in the MCP server's `env`, [the Claude Code page](/docs/claude-code/#an-app-you-are-developing) shows how). The agent cannot add to that list.

## Look at what the agent will see

Run `phonedriver snapshot` on your app's main screen before you hand it to an agent. This is what it prints for the Settings app on an Android emulator:

```screen
$ phonedriver snapshot
app com.android.settings (Settings)
@e1 button "Search Settings"
@e2 group scrollable↓
  @e3 button "Google" "Services & preferences"
  @e4 button "Network & internet" "Mobile, Wi‑Fi, hotspot"
```

Each line is an element the agent can act on by its number. If your buttons and fields are listed with their labels, the agent can work from text. If the list is short or empty, the view has no accessibility nodes. phonedriver then falls back to OCR (`text "Web form" (ocr)`), and the agent works from that and from screenshots.

## Ask for the test

Add the server to your agent ([Claude Code](/docs/claude-code/), [Codex](/docs/codex/), [VS Code](/docs/vs-code/) or [another client](/docs/cursor-and-mcp-clients/)), then ask in plain language:

> Open my app, sign up with a new account, and tell me every bug you find on the way. Give me steps to reproduce each one.

> Go through my app's onboarding in dark mode and in light mode, and show me every screen where the text is hard to read.

Every action waits for the screen to settle and returns the lines that changed, so the agent sees what its own tap did without listing the screen again. The agent decides each step from the screen. It does not need test IDs or a script.

## Passwords and test accounts

A plain `type` into a password field or a one-time code field is refused. The error tells the agent to pass the text with `secret`, which keeps it out of logs and traces. For a sign-up test the agent can make up a test password and send it that way without asking you. For a real account, give the password yourself when the agent asks.

## Dark mode and other settings

A setting is one call through [the shell](/docs/shell/), with no screens to tap through:

```sh
phonedriver shell "cmd uimode night yes"        # Android
phonedriver shell "simctl ui appearance dark"   # iOS Simulator
```

## Keep the run as a test

When the agent has walked a flow that works, `phonedriver save signup.steps` writes what it did, one step a line, and `phonedriver replay signup.steps` does it again with no model in a few seconds. A replay stops at the first step that fails and shows the screen, so the agent (or you) can fix that one step and go on. The same files replay on every commit with [the GitHub Action](/docs/replay-in-ci/).

While the agent works, `phonedriver record start` and `record stop` save a video of the screen with a list of the actions and when each began, and `phonedriver compare before.png after.png` says in text what changed between two screenshots.

## Permissions, crashes and the screen

`phonedriver apps grant <app> camera` and `apps revoke <app> location` change a permission without opening the app. `phonedriver apps notify <app> '{"aps":{"alert":"Hi"}}'` (or `--file payload.json`) sends the app a push notification, so you can test what it does with one. It works on the iOS Simulator, which delivers it the way Apple's push service would; the app must have asked for notification permission, and a payload without an `aps` object is refused. A physical iPhone and an Android phone cannot be sent one this way, and the reply says why. On Android an action during which the app crashed says so in its reply, with the exception and the line that threw it, and `phonedriver logs` prints the app's last lines. `press landscape` and `press portrait` turn the screen, and on an emulator or a Simulator `press fingerprint` and `press wrong-fingerprint` answer a fingerprint or Face ID prompt.

## Accessibility

`phonedriver snapshot --audit` lists what a screen reader or a thumb would trip on: controls with no name a screen reader can say, and tap targets under 44 by 44 points on iOS or 48 by 48 on Android. Ask the agent to audit every screen it passes and you get a list per screen, with the refs to look at:

```
audit: 2 findings
@e15 searchbox "Search": 336x38pt, under 44x44pt
@e16 button "Dictate": 17x22pt, under 44x44pt
```

It reads the element list only, so it does not check colour contrast or text that is cut short.

## Run it with nobody watching

The benchmarks ran headless: `claude -p` against an emulator started with `-no-window` (the flags are on [the emulator page](/docs/android-emulator/)). In a headless run nobody can answer the [approval prompt](/docs/approval-guard/), so a tap on Pay or Place order gets no answer and nothing is tapped.

## What was measured

Both jobs ran on a native Android testbed app with bugs planted in it, three runs each, with Claude Sonnet 5.5 through `claude -p` on one emulator on 10 November 2026. A script matched the agent's written report against the planted bugs and read the app's event log to confirm the agent had walked the buggy path. No model judged the result.

| Job | The planted bugs | Passed | Median tokens |
|---|---|---|---|
| Sign-up form | The welcome line drops the last character of the email. The Create button ignores the tap that follows a rejected submit. | 3 of 3 | 121,854 |
| Shop screen | The total stays stale after Decrease. The code SAVE10 takes a flat 10.00 instead of 10%. | 3 of 3 | 92,570 |

mobile-mcp passed both jobs in 2 of 2 runs with 549,354 and 473,095 tokens. phone-harness passed the sign-up job in 1 of 2 runs and the shop job in 2 of 2. The method and every run are on [the benchmarks page](/benchmarks/#jobs).

## React Native, Expo and Flutter

I have not tested an app built with React Native, Expo or Flutter. phonedriver does not look at how an app is built. It reads the accessibility tree that Android or iOS exposes for whatever is on screen, so the check is the one above: run `phonedriver snapshot` on your app and see whether your buttons and fields are listed. If they are not, [open an issue]({{REPO}}/issues) with the snapshot and I will look at it.

## Limits

- WebViews and custom-drawn views can have an empty accessibility tree. The agent then works from OCR and screenshots.
- A tap alone takes 450 to 530 ms on a real iPhone, against 29 to 39 ms on an Android emulator, so a long flow on an iPhone is slower.
- Screens that block screenshots come out black in a screenshot, though their text can still be read.
- A saved run is the steps one agent took, found again by their text. It is not a suite your team writes and reviews as code; for that, on a device farm, Appium is the usual choice ([phonedriver vs Appium](/compare/appium/)).

## Questions

### Can Claude Code test an app on the iOS Simulator?

Yes. Boot a Simulator on a Mac with Xcode, install the `.app` folder with `phonedriver apps install`, and ask Claude Code to use the app. The 12-scenario iOS suite passes on iOS 26.5. The two bug-finding jobs above were measured on Android only.

### Can the agent test on a real phone?

Yes. A real Android phone works over USB or Wi-Fi debugging with nothing installed on it, and an iPhone works from a Mac after `phonedriver ios setup`. For unattended runs, use an emulator or a spare phone: the agent can do anything on the phone that you could do with your finger.

### How many tokens does an agent test run use?

On my two bug-finding jobs the median was 121,854 tokens for the sign-up form and 92,570 for the shop screen, with Claude Sonnet 5.5 on 10 November 2026. Your app and your prompt will give other numbers.
