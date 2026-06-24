# Notice

This repository was prepared as part of a SkimmIQ solver collaboration.

It builds on the public fan-made repository:

```text
ashtree74/rubik-kociemba-3d
https://github.com/ashtree74/rubik-kociemba-3d
```

The original idea shared by the author treats SkimmIQ as a state graph:

- each color arrangement is a graph node
- each legal tape shift is a graph edge
- the solver searches from the current color state toward any valid solved
  state

This repository packages the later native solver and web injection work so it
can be inspected and integrated back into that style of web page.

SkimmIQ is a puzzle game by Pawel Faber. This repository is an experimental
solver/integration package, not the full SkimmIQ Android/iOS application.

No private application assets, API keys, keystores, or mobile app source code
are intentionally included.
