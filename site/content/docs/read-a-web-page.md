---
title: How to read a web page on Android as text with phonedriver page
nav: Read a web page
meta: phonedriver page prints the whole web page in front on an Android phone as text in one call, from Chrome or a debug build's web view, with no scrolling.
description: phonedriver page prints the web page in front on an Android phone as text, whole and in one call, from Chrome or a debug build's web view. How to narrow it, how it works in MCP, and what it cannot read.
updated: 2026-11-20
date: 2026-11-18
---

`phonedriver page` prints the text of the web page in front on an Android phone, whole, in one call. An article that is twelve screens tall is one reply, not twelve scrolls and twelve reads.

```console
$ phonedriver page --grep "born|died"
page "Ada Lovelace - Wikipedia" https://en.wikipedia.org/wiki/Ada_Lovelace
Born
Augusta Ada Byron
10 December 1815
Died
27 November 1852 (aged 36)
...
```

The reply is the title, the address and the text a person can read on the page, in reading order, with blank lines dropped.

## Narrow a long page

A long page is cut at a line end before 8,000 characters, and the reply says how much is left.

- `--grep "a|b"` keeps the lines that have one of the words and the two lines after each. A page often puts a label and its value on separate lines, which is why the two lines after are kept.
- `--all` prints everything.

## In MCP

It is the `snapshot` tool with `page: true`. `find` takes the words that `--grep` takes. MCP has no `all`, so an agent narrows a long page with `find`.

## How long it takes

On an Android 17 emulator, the mobile Wikipedia article on Ada Lovelace (83 lines, 6,182 characters) came back in 0.4 to 0.6 s. The GNU GPL's text in Chrome, 35,738 characters, came back whole in 0.54 s with `--all`.

## What it reads, and what it does not

- **Chrome**, and **a web view inside an app that is a debug build** or that turned web debugging on. phonedriver reads through the debugging socket those apps open on the phone, over the adb connection it already has. It opens no port on your computer.
- Any other app answers that it has no page that can be read. Use `phonedriver snapshot` there.
- It only reads. Taps and typing still go through the screen, so the [approval guard](/docs/approval-guard/) sees every button an agent presses on a web page.
- A page's fixed bottom bar, such as a site's tab bar in Chrome, counts as covering what is under it. A tap on something partly under the bar goes to the part that shows, and one fully under it fails with `covered_by: "page's bar"` instead of tapping the bar.
- iOS has no equivalent yet.

The design is decision D71 in [the repository]({{REPO}}/blob/main/docs/decisions.md).
