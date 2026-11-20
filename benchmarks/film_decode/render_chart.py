"""Render baseline and current film-decode ratio charts from their CSV files."""

import csv
from html import escape
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LEFT, RIGHT, TOP, ROW_HEIGHT = 300, 880, 88, 36
TARGET = 4 / 3


def render(source: str, destination: str, heading: str, max_ratio: float) -> None:
    with (ROOT / source).open(newline="") as file:
        rows = list(csv.DictReader(file))
    height = TOP + len(rows) * ROW_HEIGHT + 64

    def x(ratio: float) -> float:
        return LEFT + ratio / max_ratio * (RIGHT - LEFT)

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="960" height="{height}" viewBox="0 0 960 {height}" role="img" aria-labelledby="title desc">',
        f'<title id="title">{escape(heading)}</title>',
        '<desc id="desc">Each bar is libfilmshell milliseconds divided by halo_api milliseconds. Values above one mean halo_api is faster; the dashed line marks a 25 percent lead.</desc>',
        '<rect width="960" height="100%" fill="#ffffff"/>',
        f'<text x="24" y="31" font-family="sans-serif" font-size="19" font-weight="bold" fill="#172b4d">{escape(heading)}</text>',
        '<text x="24" y="55" font-family="sans-serif" font-size="13" fill="#42526e">Higher is faster for halo_api · 1.0× is equal time · 1.33× is the 25% lead</text>',
    ]
    for tick in (0.5, 1.0, TARGET, 2.0, 3.0, 4.0):
        if tick > max_ratio:
            continue
        xx = x(tick)
        stroke = "#16804a" if tick == TARGET else "#51657a" if tick == 1 else "#dfe5ec"
        dash = ' stroke-dasharray="4 4"' if tick == TARGET else ""
        parts.append(
            f'<line x1="{xx:.1f}" x2="{xx:.1f}" y1="{TOP - 13}" y2="{height - 43}" stroke="{stroke}" stroke-width="{2 if tick in (1.0, TARGET) else 1}"{dash}/>'
        )
        label = f"{tick:.2f}" if tick == TARGET else f"{tick:g}"
        parts.append(
            f'<text x="{xx:.1f}" y="{height - 20}" text-anchor="middle" font-family="sans-serif" font-size="12" fill="#42526e">{label}×</text>'
        )
    for i, row in enumerate(rows):
        ratio = float(row["libfilmshell_ms"]) / float(row["halo_api_ms"])
        y = TOP + i * ROW_HEIGHT
        color = "#16804a" if ratio >= TARGET else "#b76519"
        parts.append(
            f'<text x="24" y="{y + 8}" font-family="sans-serif" font-size="13" fill="#172b4d">{escape(row["film"])}</text>'
        )
        parts.append(
            f'<rect x="{LEFT}" y="{y - 4}" width="{x(ratio) - LEFT:.1f}" height="17" rx="3" fill="{color}"/>'
        )
        parts.append(
            f'<text x="{x(ratio) + 7:.1f}" y="{y + 9}" font-family="sans-serif" font-size="12" fill="#172b4d">{ratio:.2f}×</text>'
        )
    parts.append("</svg>\n")
    (ROOT / destination).write_text("\n".join(parts))


render("comparison.csv", "comparison.svg", "Initial whole-film decode speed", 1.8)
render("parse_to_return.csv", "parse_to_return.svg", "Current parse-to-return speed", 4.8)
