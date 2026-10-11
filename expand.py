#!/usr/bin/env python3
import json
import os
import sys

from pathlib import Path
from string import Template

template = Template("""# ${NAME} theme for iamb
#
# You can place this directly in your `~/.config/iamb/themes` directory to make it
# discoverable, or copy its contents directly into your `config.toml`.
#
# If you are looking to remix this file to make a derived theme, the colors were
# picked and assigned as:
#
# COLOR_BG1 => Primary background (${COLOR_BG1})
# COLOR_BG2 => Alternate background (${COLOR_BG2})
# COLOR_BG3 => Completion menu selected item background (${COLOR_BG3})
#
# COLOR_UI1 => Window border color, text completion menu (${COLOR_UI1})
# COLOR_UI2 => Window title color (${COLOR_UI2})
#
# COLOR_WARN1 => Errors, unencrypted, etc. - typically a red (${COLOR_WARN1})
# COLOR_WARN2 => Redacted messages - typically a red or light orange (${COLOR_WARN2})
# COLOR_WARN3 => Unknown encryption status - typically a yellow (${COLOR_WARN3})
#
# COLOR_GOOD1 => Encrypted indicator - typically a green (${COLOR_GOOD1})
#
# COLOR_TEXT1 => Primary text color (${COLOR_TEXT1})
# COLOR_TEXT2 => Contrast text color, should differ from COLOR_TEXT1 (${COLOR_TEXT2})
# COLOR_TEXT3 => Alternate text color (${COLOR_TEXT3})
# COLOR_TEXT4 => Alternate text color (${COLOR_TEXT4})
# COLOR_TEXT5 => Alternate text color (${COLOR_TEXT5})
#
#${COLOR_PLUS_DOC}
[theme.users]
colors = [
    "${COLOR_WARN1}",
    "${COLOR_WARN2}",
    "${COLOR_WARN3}",
    "${COLOR_GOOD1}",
    "${COLOR_TEXT2}",
    "${COLOR_TEXT3}",
    "${COLOR_TEXT4}",
    "${COLOR_TEXT5}",${COLOR_PLUS_QUOTED}
]

[theme.default]
color = "${COLOR_TEXT1}"
background = "${COLOR_BG1}"

[theme.completion]
default.background = "${COLOR_BG3}"
default.color = "${COLOR_UI1}"
selected.background = "${COLOR_BG2}"
selected.color = "${COLOR_TEXT1}"

[theme.timeline]
redacted.color = "${COLOR_WARN2}"

[theme.messages]
code.color = "${COLOR_TEXT2}"
code.background = "${COLOR_BG2}"
emphasis.color = "${COLOR_TEXT4}"
strong.color = "${COLOR_TEXT3}"

[theme.rooms]
default.color = "${COLOR_TEXT5}"
labels.color = "${COLOR_TEXT4}"
unread.name.color = "${COLOR_TEXT2}"
unread.number.color = "${COLOR_TEXT4}"
mention.number.color = "${COLOR_WARN1}"
notification.number.color = "${COLOR_WARN3}"
marked_unread.number.color = "${COLOR_GOOD1}"

[theme.windows]
border.color = "${COLOR_UI1}"
title.color = "${COLOR_UI2}"

[theme.encryption]
icon_encrypted.color = "${COLOR_GOOD1}"
icon_unencrypted.color = "${COLOR_WARN1}"
icon_unknown.color = "${COLOR_WARN3}"

[theme.cmdbar]
error.color = "${COLOR_WARN1}"
""")

def generate_maybe(directory, name, mapped):
    if mapped is None:
        return

    users = mapped.get("COLOR_PLUS", [])
    users.sort()

    if len(users) > 0:
        mapped["COLOR_PLUS_DOC"] = " COLOR_PLUS => Extra colors used for user names: " +  " ".join(users)
    else:
        mapped["COLOR_PLUS_DOC"] = ""

    mapped["COLOR_PLUS_QUOTED"] = "".join([f'\n    "{u}",' for u in users])
    mapped["NAME"] = name

    out = template.substitute(mapped)

    Path(directory).mkdir(parents=True, exist_ok=True)

    with open(f"{directory}/{name}.toml", "w", encoding="utf-8") as file:
        file.write(out)

def generate_colorschemes(d, c):
    name = c["name"]
    generate_maybe(d, f"{name}-light", c["data"].get("light", None))
    generate_maybe(d, f"{name}-dark", c["data"].get("dark", None))

def generate_repo(repo):
    # Set up the directory structure to mirror GitHub owner/repo, since
    # the source JSON contains multiple repositories that contain themes
    # of the same name, which would cause us to overwrite output with
    # different, slight variations of the colorscheme:
    owner = repo["owner"]["name"]
    name = repo["name"]
    directory = f"target/{owner}/{name}"

    for colorschemes in repo["vimColorSchemes"]:
        generate_colorschemes(directory, colorschemes)

def main(source):
    with open(source, 'r') as file:
        source = json.load(file)

    for repo in source:
        generate_repo(repo)

if len(sys.argv) == 2:
    main(sys.argv[1])
else:
    print("""
    Usage: generate.py iamb.json

    Example:

    $ ./expand.py iamb.json
    """)
    sys.exit(2)
