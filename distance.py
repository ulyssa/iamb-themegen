#!/usr/bin/env python3
import json
import math
import os
import sys

from colormath2.color_objects import sRGBColor, LabColor
from colormath2.color_conversions import convert_color
from colormath2.color_diff import delta_e_cie2000
from pathlib import Path
from string import Template

if len(sys.argv) != 2:
    print("""
Usage: distance.py [HEX] <<EOF
#aabbcc
#001122
EOF
    """)
    sys.exit(2)

base = sRGBColor.new_from_rgb_hex(sys.argv[1])
base_lab = convert_color(base, LabColor)

for line in sys.stdin:
    if not line.startswith("#"):
        print(line)
        continue

    line = line.rstrip()
    target = sRGBColor.new_from_rgb_hex(line)
    target_lab = convert_color(target, LabColor)
    dist = delta_e_cie2000(base_lab, target_lab)
    print(f"{line}\t{dist}")
