"""Generate the API reference pages automatically."""

from pathlib import Path
import re

import mkdocs_gen_files

# Objects to document from the kaspa module
stub_file = Path("kaspa.pyi")

nav = mkdocs_gen_files.Nav()

CATEGORY_ORDER = [
    "Classes",
    "Enums",
    "Functions",
    "TypedDicts",
]


def parse_stub_file(content: str) -> dict:
    """
    Parse the .pyi stub file and extract objects with their categories.

    Returns dict mapping object names to their info:
        {name: {"type": "class"|"function"|"enum"|"typeddict", "category": str}}
    """
    objects = {}

    # Pattern to match class definitions with their docstrings
    # Matches: class Name: or class Name(bases):
    # followed by optional docstring
    class_pattern = re.compile(
        r'^(?:@typing\.final\s*\n)?class\s+(\w+)(?:\(([^)]*)\))?:\s*\n'
        r'(?:\s+r?"""(.*?)""")?',
        re.MULTILINE | re.DOTALL
    )

    for match in class_pattern.finditer(content):
        name = match.group(1)
        bases = match.group(2) or ""

        # Determine type
        if 'enum.Enum' in bases or 'Enum' in bases:
            obj_type = "enum"
            category = "Enums"
        elif 'TypedDict' in bases:
            obj_type = "typeddict"
            category = "TypedDicts"
        else:
            obj_type = "class"
            category = "Classes"

        objects[name] = {"type": obj_type, "category": category}

    # Find standalone functions with their docstrings
    # Pattern: def name(...): on single line, followed by optional docstring
    func_pattern = re.compile(
        r'^def\s+(\w+)[^\n]+:\n(?:\s+r?"""(.*?)""")?',
        re.MULTILINE | re.DOTALL
    )
    for match in func_pattern.finditer(content):
        name = match.group(1)
        objects[name] = {"type": "function", "category": "Functions"}

    return objects


def extract_category(docstring: str, default: str) -> str:
    """Extract Category: value from docstring."""
    if not docstring:
        return default

    # Look for "Category: Something" or "Category: Something/Subcategory"
    match = re.search(r'Category:\s*([^\n]+)', docstring, re.IGNORECASE)
    if match:
        return match.group(1).strip()

    return default


def category_sort_key(category: str) -> tuple:
    """Sort key for categories based on CATEGORY_ORDER."""
    try:
        return (0, CATEGORY_ORDER.index(category))
    except ValueError:
        return (1, category)


if stub_file.exists():
    content = stub_file.read_text()
    objects = parse_stub_file(content)

    # Group objects by category
    by_category: dict[str, list[str]] = {}
    for name, info in objects.items():
        cat = info["category"]
        if cat not in by_category:
            by_category[cat] = []
        by_category[cat].append(name)

    # Sort objects within each category
    for cat in by_category:
        by_category[cat].sort()

    # Sort categories
    sorted_categories = sorted(by_category.keys(), key=category_sort_key)

    # Generate index page
    index_path = Path("reference", "index.md")
    with mkdocs_gen_files.open(index_path, "w") as f:
        f.write((
            "# API Reference\n\n"
            "Complete reference for the Kaspa Python SDK."
        ))

    nav[("index",)] = "index.md"

    def get_type_label(name: str) -> str:
        """Get the display label for an object's type."""
        obj_type = objects[name]["type"]
        if obj_type == "enum":
            return "Enum"
        elif obj_type == "typeddict":
            return "TypedDict"
        elif obj_type == "class":
            return "Class"
        else:
            return "Func"

    def nav_label(name: str) -> str:
        """Generate navigation label."""
        return f'{name}'

    def category_to_nav_path(category: str) -> tuple:
        """Convert category string to nav path tuple."""
        return tuple(category.split("/"))

    # Generate category index pages and item pages
    for category in sorted_categories:
        nav_path = category_to_nav_path(category)

        # Generate pages for each item in category subdirectory
        for name in by_category[category]:
            doc_path = Path("reference", category, f"{name}.md")
            type_label = get_type_label(name)

            with mkdocs_gen_files.open(doc_path, "w") as f:
                f.write(f'# `{name}` ({type_label})\n\n')
                f.write(f"::: kaspa.{name}\n")
                f.write("    options:\n")
                f.write("      show_root_heading: false\n")
                f.write("      show_root_full_path: false\n")

            # Add to nav with category hierarchy
            nav[(*nav_path, nav_label(name))] = f"{category}/{name}.md"

    # Generate the navigation file
    with mkdocs_gen_files.open("reference/SUMMARY.md", "w") as nav_file:
        nav_file.writelines(nav.build_literate_nav())
