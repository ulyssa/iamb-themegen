#!/usr/bin/env python3
import json
import math
import os
import sys

from colormath2.color_objects import sRGBColor, LabColor
from colormath2.color_conversions import convert_color
from colormath2.color_diff import delta_e_cie2000

THEME_DEBUG = os.getenv("THEME_DEBUG", "false") != "false"
USE_CIELAB = os.getenv("THEME_CIELAB", "true") != "false"

def hex_to_rgb(hex_code):
    """
    Convert a hex string into an (R, G, B) tuple
    """
    h = hex_code.lstrip('#')
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4))

def get_distance_to_color(hex_code, r_val, g_val, b_val):
    """
    Calculate the Euclidean distance of a hex code from another RGB color
    """
    r, g, b = hex_to_rgb(hex_code)

    # Calculate Euclidean distance to pure red (255, 0, 0)
    return math.sqrt((r_val - r)**2 + (g_val - g)**2 + (b_val - b)**2)

def get_delta_e_to_color(hex_code, target):
    """
    Cacluate the difference between a hex code and a target color in
    the CIELAB color space.
    """
    target_lab = convert_color(target, LabColor)

    # Parse the input hex color into an sRGB object, and then convert to CIELAB:
    input_rgb = sRGBColor.new_from_rgb_hex(hex_code)
    input_lab = convert_color(input_rgb, LabColor)

    # Calculate the Delta E 2000 distance (lower means perceptually closer):
    return delta_e_cie2000(target_lab, input_lab)

def get_distance_to_red(hex_code):
    # This avoids using #ff0000 because that seems to result in bright orange
    # getting picked over light red/pink colors in practice.
    if USE_CIELAB:
        return get_delta_e_to_color(hex_code, sRGBColor(0.9, 0.0, 0.0))
    else:
        return get_distance_to_color(hex_code, 230, 0, 0)

def get_distance_to_green(hex_code):
    if USE_CIELAB:
        return get_delta_e_to_color(hex_code, sRGBColor(0.0, 0.5, 0.0))
    else:
        return get_distance_to_color(hex_code, 0, 255, 0)

def get_distance_to_yellow(hex_code):
    if USE_CIELAB:
        return get_delta_e_to_color(hex_code, sRGBColor(0.5, 0.5, 0.0))
    else:
        return get_distance_to_color(hex_code, 255, 255, 0)

def get_distance_to_white(hex_code):
    if USE_CIELAB:
        return get_delta_e_to_color(hex_code, sRGBColor(1.0, 1.0, 1.0))
    else:
        return get_distance_to_color(hex_code, 255, 255, 255)

def get_distance_to_black(hex_code):
    if USE_CIELAB:
        return get_delta_e_to_color(hex_code, sRGBColor(0.0, 0.0, 0.0))
    else:
        return get_distance_to_color(hex_code, 0, 0, 0)

def get_distance_to_hex(h1, h2):
    if USE_CIELAB:
        target = sRGBColor.new_from_rgb_hex(h2)
        return get_delta_e_to_color(h1, target)
    else:
        r, g, b = hex_to_rgb(h2)
        return get_distance_to_color(h1, r, g, b)

def colorscheme_assign(name, colorscheme, is_dark):
    mapped = {}
    backgrounds = set()
    foregrounds = set()

    for c in colorscheme:
        if c["name"].endswith("Bg"):
            backgrounds.add(c["hexCode"].upper())
        elif c["name"].endswith("Fg"):
            foregrounds.add(c["hexCode"].upper())

    # Sort the background set by increasing distance from all-white/all-black:
    if is_dark:
        bg_target = "#000000"
    else:
        bg_target = "#ffffff"

    backgrounds = list(backgrounds)
    backgrounds.sort(key=lambda h: get_distance_to_hex(h, bg_target))
    lb = len(backgrounds)

    if lb < 3:
        if THEME_DEBUG:
            print(f"Skipping {name} due to insufficient background colors ({lb})")
        return None

    if is_dark:
        mapped["COLOR_BG1"] = backgrounds[0]
        mapped["COLOR_BG2"] = backgrounds[1]
        mapped["COLOR_BG3"] = backgrounds[2]
    else:
        mapped["COLOR_BG1"] = backgrounds[1]
        mapped["COLOR_BG2"] = backgrounds[0]
        mapped["COLOR_BG3"] = backgrounds[2]

    bg1 = mapped["COLOR_BG1"]

    # Filter out colors too close to COLOR_BG1 from the foreground:
    existing = set({mapped[k] for k in mapped})
    others = set({c for c in foregrounds if get_distance_to_hex(c, bg1) > 4 })
    others.difference_update(existing)

    lf = len(others)

    if lf < 9:
        # Some color schemes just don't have that many colors, so we just
        # skip them here rather than generate something that would have
        # most usernames rendered in the same small set of colors.
        if THEME_DEBUG:
            print(f"Skipping {name} due to insufficient foreground colors ({lf})")
        return None

    # Pick our informational red/yellow/green colors first:
    green = min(others, key=get_distance_to_green)
    others.remove(green)

    red1 = min(others, key=get_distance_to_red)
    others.remove(red1)

    red2 = min(others, key=get_distance_to_red)
    others.remove(red2)

    yellow = min(others, key=get_distance_to_yellow)
    others.remove(yellow)

    # Pick our primary foreground color for normal text and window titles
    # based on whether this is a light or dark theme by picking the colors
    # furthest away from the background:
    others = list(others)
    others.sort(key=lambda h: get_distance_to_hex(h, bg1))

    text1 = others.pop()
    ui2 = others.pop()
    text2 = others.pop()
    text3 = others.pop()

    # At this point, we've used 9 foreground colors, and might be out!
    #
    # If we do have more, we'll try to use any remaining contrasting colors
    # for usernames, and some other text.
    if lf > 9:
        # We have plenty of colors, so we can use an extra one
        # for italicized text and room labels:
        text4 = others.pop()
    else:
        # Too few colors, just make `text4` the same as `text1`:
        text4 = text1

    if lf > 10:
        # We have plenty of colors, so we can use an extra one
        # in room lists for read rooms:
        text5 = others.pop()
    else:
        # Too few colors, just make `text5` the same as `text1`:
        text5 = text1

    ui1 = others.pop()

    users = []

    for color in others:
        dist = get_distance_to_hex(color, bg1)
        if dist >= 5:
            users.append(color)

    # Sort to ensure `.toml` output is stable:
    users.sort()
    users = [f'"{c}"' for c in users]
    users = ", ".join(users)

    # Put them into the template map:
    mapped["COLOR_WARN1"] = red1
    mapped["COLOR_WARN2"] = red2
    mapped["COLOR_WARN3"] = yellow
    mapped["COLOR_GOOD1"] = green
    mapped["COLOR_TEXT1"] = text1
    mapped["COLOR_TEXT2"] = text2
    mapped["COLOR_TEXT3"] = text3
    mapped["COLOR_TEXT4"] = text4
    mapped["COLOR_TEXT5"] = text5
    mapped["COLOR_UI1"] = ui1
    mapped["COLOR_UI2"] = ui2
    mapped["COLOR_PLUS"] = users

    return mapped 

def assign_maybe(name, colorscheme, is_dark):
    if colorscheme is None:
        return None
    return colorscheme_assign(name, colorscheme, is_dark)

def assign_colorschemes(c):
    name = c["name"]
    data = {}

    light = assign_maybe(f"{name}-light", c["data"].get("light", None), False)
    if light is not None:
        data["light"] = light

    dark = assign_maybe(f"{name}-dark", c["data"].get("dark", None), True)
    if dark is not None:
        data["dark"] = dark

    if len(data) == 0:
        return None

    return {
        "name": name,
        "data": data,
    }

def assign_repo(repo):
    colors = [assign_colorschemes(c) for c in repo["vimColorSchemes"]]
    colors = [c for c in colors if c is not None]

    return {
        "name": repo["name"],
        "owner": repo["owner"],
        "vimColorSchemes": colors
    }

def main(source, stars):
    with open(source, 'r') as file:
        source = json.load(file)

    repos = [assign_repo(repo) for repo in source if repo["stargazersCount"] < stars]
    repos = [repo for repo in repos if repo is not None]

    json.dump(repos, sys.stdout, indent=2)


if len(sys.argv) < 2 or len(sys.argv) > 3:
    print("""
    Usage: vim2iamb.py input.json [minimum stars]

    This script rewrites the colorschemes from the vimcolorschemes.com site into
    a set of color assignments that can be used with `expand.py` to generate
    themes for iamb.

    Example:

    $ curl -s https://vimcolorschemes.com/search/repositories.json > repositories.json
    $ ./vim2iamb.py repositories.json 1000
    """)
    sys.exit(2)

if len(sys.argv) > 2:
    main(sys.argv[1], int(sys.argv[2]))
else:
    main(sys.argv[1], 200)
