# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.0   | Yes       |

0.1.0 is the first GitHub release. Older commits are history, not a supported line.

## Reporting a vulnerability

Open a private vulnerability report on this GitHub repository
(Security, then Report a vulnerability). Do not put the details in a
public issue.

There is no separate mailbox for reports. Do not send a home address,
a personal mailbox, or a token path.

Include what you ran, the version or commit, and what you expected.
A save file helps when the bug is in a loaded game. Strip anything
that identifies you if you do not want it in the fix.

### Response

There is no paid desk and no emergency line. We read private reports
and aim to acknowledge one within 7 days. A valid fix ships in a
later release. We do not promise a clock inside that.

## What this program touches

Portlight Bounty is a local game. The simulation does not open a
network connection. The Godot chart does not phone home. There is no
account, no telemetry, and no crash uploader.

Data it writes: a version-12 save, in a folder you choose from the
chart, or in the directory you pass to `portlight script --save-dir`
and `portlight load`. The chart's default folder is `saves/` under
the working directory.

Data it reads: the embedded content catalog, your save, and the script
file you name. It does not read credentials, a mailbox, or a home
directory on purpose.

The Python oracle in CI checks the public Portlight repository out at
a pinned commit. That job is CI, not the game.

## Out of scope

Godot editor bugs, a stolen copy of a third-party model weight, and
the generator account terms on pictures that are not under the MIT
grant. Those stay with their owners.
